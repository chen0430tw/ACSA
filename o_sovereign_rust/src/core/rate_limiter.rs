// Rate Limiter - 速率限制系统
// 防止API滥用和DDoS攻击
//
// 核心功能：
// 1. Token Bucket算法
// 2. IP/用户级别限流
// 3. API端点级别限流
// 4. 动态限流规则
// 5. 限流统计和监控

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 限流策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    /// Token Bucket（令牌桶）
    TokenBucket,
    /// Fixed Window（固定窗口）
    FixedWindow,
    /// Sliding Window（滑动窗口）
    SlidingWindow,
}

/// 限流级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RateLimitLevel {
    /// IP地址级别
    IpAddress,
    /// 用户级别
    User,
    /// API端点级别
    Endpoint,
    /// 全局级别
    Global,
}

/// 限流规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    /// 规则ID
    pub rule_id: String,
    /// 限流级别
    pub level: RateLimitLevel,
    /// 策略
    pub strategy: RateLimitStrategy,
    /// 每秒请求数
    pub requests_per_second: f64,
    /// 桶容量（最大突发请求数）
    pub bucket_capacity: u64,
    /// 窗口大小（秒）
    pub window_size_secs: u64,
    /// 是否启用
    pub enabled: bool,
}

impl Default for RateLimitRule {
    fn default() -> Self {
        Self {
            rule_id: "default".to_string(),
            level: RateLimitLevel::Global,
            strategy: RateLimitStrategy::TokenBucket,
            requests_per_second: 10.0,
            bucket_capacity: 20,
            window_size_secs: 60,
            enabled: true,
        }
    }
}

/// Token Bucket状态
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenBucket {
    /// 当前令牌数
    tokens: f64,
    /// 最大容量
    capacity: f64,
    /// 补充速率（每秒）
    refill_rate: f64,
    /// 最后更新时间
    last_update: DateTime<Utc>,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_update: Utc::now(),
        }
    }

    fn refill(&mut self) {
        let now = Utc::now();
        let elapsed = (now - self.last_update).num_milliseconds() as f64 / 1000.0;
        let new_tokens = elapsed * self.refill_rate;

        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_update = now;
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn tokens_available(&self) -> f64 {
        self.tokens
    }
}

/// 限流记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRecord {
    /// 标识符（IP/用户ID/端点）
    pub identifier: String,
    /// 限流级别
    pub level: RateLimitLevel,
    /// 总请求数
    pub total_requests: u64,
    /// 被限流的请求数
    pub throttled_requests: u64,
    /// 第一次请求时间
    pub first_request_at: DateTime<Utc>,
    /// 最后请求时间
    pub last_request_at: DateTime<Utc>,
}

/// 限流结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    /// 是否允许
    pub allowed: bool,
    /// 剩余配额
    pub remaining: u64,
    /// 配额重置时间
    pub reset_at: DateTime<Utc>,
    /// 重试时间（秒）
    pub retry_after_secs: Option<u64>,
}

/// 速率限制器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    /// 全局限流规则
    pub global_rule: RateLimitRule,
    /// IP级别规则
    pub ip_rule: RateLimitRule,
    /// 用户级别规则
    pub user_rule: RateLimitRule,
    /// 端点级别规则
    pub endpoint_rules: HashMap<String, RateLimitRule>,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            global_rule: RateLimitRule {
                rule_id: "global".to_string(),
                level: RateLimitLevel::Global,
                requests_per_second: 1000.0,
                bucket_capacity: 2000,
                ..Default::default()
            },
            ip_rule: RateLimitRule {
                rule_id: "ip".to_string(),
                level: RateLimitLevel::IpAddress,
                requests_per_second: 10.0,
                bucket_capacity: 20,
                ..Default::default()
            },
            user_rule: RateLimitRule {
                rule_id: "user".to_string(),
                level: RateLimitLevel::User,
                requests_per_second: 50.0,
                bucket_capacity: 100,
                ..Default::default()
            },
            endpoint_rules: HashMap::new(),
        }
    }
}

/// 速率限制器
pub struct RateLimiter {
    config: RateLimiterConfig,
    /// IP级别的桶
    ip_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    /// 用户级别的桶
    user_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    /// 端点级别的桶
    endpoint_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    /// 全局桶
    global_bucket: Arc<RwLock<TokenBucket>>,
    /// 限流记录
    records: Arc<RwLock<HashMap<String, RateLimitRecord>>>,
}

impl RateLimiter {
    /// 创建新的速率限制器
    pub fn new(config: RateLimiterConfig) -> Self {
        info!("🚦 Initializing Rate Limiter");
        info!("    Global: {:.1} req/s", config.global_rule.requests_per_second);
        info!("    IP: {:.1} req/s", config.ip_rule.requests_per_second);
        info!("    User: {:.1} req/s", config.user_rule.requests_per_second);

        let global_bucket = TokenBucket::new(
            config.global_rule.bucket_capacity as f64,
            config.global_rule.requests_per_second,
        );

        Self {
            config,
            ip_buckets: Arc::new(RwLock::new(HashMap::new())),
            user_buckets: Arc::new(RwLock::new(HashMap::new())),
            endpoint_buckets: Arc::new(RwLock::new(HashMap::new())),
            global_bucket: Arc::new(RwLock::new(global_bucket)),
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检查IP是否被限流
    pub async fn check_ip(&self, ip: &str) -> Result<RateLimitResult> {
        if !self.config.ip_rule.enabled {
            return Ok(RateLimitResult {
                allowed: true,
                remaining: u64::MAX,
                reset_at: Utc::now() + Duration::hours(1),
                retry_after_secs: None,
            });
        }

        self.check_bucket(
            ip,
            &self.ip_buckets,
            &self.config.ip_rule,
            RateLimitLevel::IpAddress,
        )
        .await
    }

    /// 检查用户是否被限流
    pub async fn check_user(&self, user_id: &str) -> Result<RateLimitResult> {
        if !self.config.user_rule.enabled {
            return Ok(RateLimitResult {
                allowed: true,
                remaining: u64::MAX,
                reset_at: Utc::now() + Duration::hours(1),
                retry_after_secs: None,
            });
        }

        self.check_bucket(
            user_id,
            &self.user_buckets,
            &self.config.user_rule,
            RateLimitLevel::User,
        )
        .await
    }

    /// 检查端点是否被限流
    pub async fn check_endpoint(&self, endpoint: &str, identifier: &str) -> Result<RateLimitResult> {
        if let Some(rule) = self.config.endpoint_rules.get(endpoint) {
            if !rule.enabled {
                return Ok(RateLimitResult {
                    allowed: true,
                    remaining: u64::MAX,
                    reset_at: Utc::now() + Duration::hours(1),
                    retry_after_secs: None,
                });
            }

            let key = format!("{}:{}", endpoint, identifier);
            self.check_bucket(&key, &self.endpoint_buckets, rule, RateLimitLevel::Endpoint)
                .await
        } else {
            // 无特定规则，使用全局规则
            Ok(RateLimitResult {
                allowed: true,
                remaining: u64::MAX,
                reset_at: Utc::now() + Duration::hours(1),
                retry_after_secs: None,
            })
        }
    }

    /// 检查全局限流
    pub async fn check_global(&self) -> Result<RateLimitResult> {
        if !self.config.global_rule.enabled {
            return Ok(RateLimitResult {
                allowed: true,
                remaining: u64::MAX,
                reset_at: Utc::now() + Duration::hours(1),
                retry_after_secs: None,
            });
        }

        let mut bucket = self.global_bucket.write().await;
        let allowed = bucket.try_consume(1.0);

        Ok(RateLimitResult {
            allowed,
            remaining: bucket.tokens_available() as u64,
            reset_at: Utc::now() + Duration::seconds(self.config.global_rule.window_size_secs as i64),
            retry_after_secs: if !allowed { Some(1) } else { None },
        })
    }

    /// 综合检查（检查所有级别）
    pub async fn check_all(
        &self,
        ip: &str,
        user_id: Option<&str>,
        endpoint: Option<&str>,
    ) -> Result<RateLimitResult> {
        // 首先检查全局限流
        let global = self.check_global().await?;
        if !global.allowed {
            warn!("🚫 Global rate limit exceeded");
            return Ok(global);
        }

        // 检查IP限流
        let ip_result = self.check_ip(ip).await?;
        if !ip_result.allowed {
            warn!("🚫 IP rate limit exceeded: {}", ip);
            return Ok(ip_result);
        }

        // 检查用户限流
        if let Some(uid) = user_id {
            let user_result = self.check_user(uid).await?;
            if !user_result.allowed {
                warn!("🚫 User rate limit exceeded: {}", uid);
                return Ok(user_result);
            }
        }

        // 检查端点限流
        if let Some(ep) = endpoint {
            let identifier = user_id.unwrap_or(ip);
            let endpoint_result = self.check_endpoint(ep, identifier).await?;
            if !endpoint_result.allowed {
                warn!("🚫 Endpoint rate limit exceeded: {}", ep);
                return Ok(endpoint_result);
            }
        }

        Ok(RateLimitResult {
            allowed: true,
            remaining: ip_result.remaining,
            reset_at: ip_result.reset_at,
            retry_after_secs: None,
        })
    }

    /// 获取限流统计
    pub async fn get_stats(&self, identifier: &str) -> Option<RateLimitRecord> {
        let records = self.records.read().await;
        records.get(identifier).cloned()
    }

    /// 重置限流计数
    pub async fn reset(&self, identifier: &str, level: RateLimitLevel) -> Result<()> {
        match level {
            RateLimitLevel::IpAddress => {
                let mut buckets = self.ip_buckets.write().await;
                buckets.remove(identifier);
            }
            RateLimitLevel::User => {
                let mut buckets = self.user_buckets.write().await;
                buckets.remove(identifier);
            }
            RateLimitLevel::Endpoint => {
                let mut buckets = self.endpoint_buckets.write().await;
                buckets.remove(identifier);
            }
            RateLimitLevel::Global => {
                let mut bucket = self.global_bucket.write().await;
                *bucket = TokenBucket::new(
                    self.config.global_rule.bucket_capacity as f64,
                    self.config.global_rule.requests_per_second,
                );
            }
        }

        info!("🔄 Rate limit reset: {} ({:?})", identifier, level);
        Ok(())
    }

    // ===== 内部辅助方法 =====

    async fn check_bucket(
        &self,
        identifier: &str,
        buckets: &Arc<RwLock<HashMap<String, TokenBucket>>>,
        rule: &RateLimitRule,
        level: RateLimitLevel,
    ) -> Result<RateLimitResult> {
        let mut buckets_map = buckets.write().await;

        let bucket = buckets_map
            .entry(identifier.to_string())
            .or_insert_with(|| TokenBucket::new(rule.bucket_capacity as f64, rule.requests_per_second));

        let allowed = bucket.try_consume(1.0);

        // 更新记录
        self.update_record(identifier, level, allowed).await;

        Ok(RateLimitResult {
            allowed,
            remaining: bucket.tokens_available() as u64,
            reset_at: Utc::now() + Duration::seconds(rule.window_size_secs as i64),
            retry_after_secs: if !allowed {
                Some((1.0 / rule.requests_per_second) as u64)
            } else {
                None
            },
        })
    }

    async fn update_record(&self, identifier: &str, level: RateLimitLevel, allowed: bool) {
        let mut records = self.records.write().await;

        let record = records
            .entry(identifier.to_string())
            .or_insert_with(|| RateLimitRecord {
                identifier: identifier.to_string(),
                level,
                total_requests: 0,
                throttled_requests: 0,
                first_request_at: Utc::now(),
                last_request_at: Utc::now(),
            });

        record.total_requests += 1;
        if !allowed {
            record.throttled_requests += 1;
        }
        record.last_request_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10.0, 1.0);

        // 消耗5个令牌
        assert!(bucket.try_consume(5.0));
        assert!(bucket.tokens_available() >= 4.9 && bucket.tokens_available() <= 5.1);

        // 尝试消耗10个令牌（应该失败）
        assert!(!bucket.try_consume(10.0));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimiterConfig::default();
        let limiter = RateLimiter::new(config);

        let result = limiter.check_ip("192.168.1.1").await.unwrap();
        assert!(result.allowed);
    }
}
