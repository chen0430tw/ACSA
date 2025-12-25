// Shadow Mode - 影子模式数据保护系统
// 用户数据安全的最后一道防线
//
// 核心功能：
// 1. PII（个人身份信息）自动检测
// 2. 敏感数据脱敏/匿名化
// 3. 数据访问审计
// 4. 与SOSA加密集成
// 5. 可逆/不可逆脱敏策略

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::sosa_crypto::{EncryptedData, KeyPurpose, SosaCryptoEngine};

/// PII类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PiiType {
    /// 电子邮件
    Email,
    /// 电话号码
    Phone,
    /// 身份证号
    IdCard,
    /// 信用卡号
    CreditCard,
    /// IP地址
    IpAddress,
    /// 姓名
    Name,
    /// 地址
    Address,
    /// 银行账户
    BankAccount,
    /// 密码/密钥
    Credential,
    /// 其他敏感信息
    Other,
}

/// 脱敏策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskingStrategy {
    /// 完全隐藏 (xxx...xxx)
    FullMask,
    /// 部分隐藏 (保留前后部分)
    PartialMask,
    /// 哈希替换 (SHA256)
    Hash,
    /// 加密存储 (可逆，使用SOSA加密)
    Encrypt,
    /// 随机化 (保持格式但替换内容)
    Randomize,
    /// 泛化 (例如：具体年龄 -> 年龄段)
    Generalize,
}

/// 脱敏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingConfig {
    /// PII类型
    pub pii_type: PiiType,
    /// 脱敏策略
    pub strategy: MaskingStrategy,
    /// 是否可逆
    pub reversible: bool,
    /// 保留前N个字符（用于PartialMask）
    pub keep_prefix: usize,
    /// 保留后N个字符（用于PartialMask）
    pub keep_suffix: usize,
}

impl Default for MaskingConfig {
    fn default() -> Self {
        Self {
            pii_type: PiiType::Other,
            strategy: MaskingStrategy::PartialMask,
            reversible: false,
            keep_prefix: 2,
            keep_suffix: 2,
        }
    }
}

/// 脱敏结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedData {
    /// 原始数据指纹（用于审计）
    pub fingerprint: String,
    /// 脱敏后的数据
    pub masked_value: String,
    /// 脱敏策略
    pub strategy: MaskingStrategy,
    /// 是否可逆
    pub reversible: bool,
    /// 加密数据（如果使用加密策略）
    pub encrypted: Option<EncryptedData>,
    /// 脱敏时间
    pub masked_at: DateTime<Utc>,
}

/// PII检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiDetection {
    /// 检测到的PII类型
    pub pii_type: PiiType,
    /// 匹配的文本
    pub matched_text: String,
    /// 匹配位置（起始索引）
    pub start_pos: usize,
    /// 匹配位置（结束索引）
    pub end_pos: usize,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
}

/// 访问审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessAudit {
    /// 审计ID
    pub audit_id: String,
    /// 访问者
    pub accessor: String,
    /// 访问的数据指纹
    pub data_fingerprint: String,
    /// PII类型
    pub pii_type: PiiType,
    /// 访问操作（read/decrypt/unmask）
    pub operation: String,
    /// 是否被允许
    pub allowed: bool,
    /// 拒绝原因
    pub deny_reason: Option<String>,
    /// 访问时间
    pub accessed_at: DateTime<Utc>,
}

/// 影子模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowModeConfig {
    /// 是否启用自动PII检测
    pub auto_detect_pii: bool,
    /// 是否自动脱敏
    pub auto_mask: bool,
    /// 默认脱敏策略
    pub default_masking: HashMap<PiiType, MaskingConfig>,
    /// 审计日志保留天数
    pub audit_retention_days: i64,
}

impl Default for ShadowModeConfig {
    fn default() -> Self {
        let mut default_masking = HashMap::new();

        // 邮箱：部分隐藏
        default_masking.insert(
            PiiType::Email,
            MaskingConfig {
                pii_type: PiiType::Email,
                strategy: MaskingStrategy::PartialMask,
                reversible: false,
                keep_prefix: 3,
                keep_suffix: 0,
            },
        );

        // 电话：部分隐藏
        default_masking.insert(
            PiiType::Phone,
            MaskingConfig {
                pii_type: PiiType::Phone,
                strategy: MaskingStrategy::PartialMask,
                reversible: false,
                keep_prefix: 3,
                keep_suffix: 4,
            },
        );

        // 身份证：加密存储
        default_masking.insert(
            PiiType::IdCard,
            MaskingConfig {
                pii_type: PiiType::IdCard,
                strategy: MaskingStrategy::Encrypt,
                reversible: true,
                keep_prefix: 0,
                keep_suffix: 0,
            },
        );

        // 信用卡：部分隐藏
        default_masking.insert(
            PiiType::CreditCard,
            MaskingConfig {
                pii_type: PiiType::CreditCard,
                strategy: MaskingStrategy::PartialMask,
                reversible: false,
                keep_prefix: 0,
                keep_suffix: 4,
            },
        );

        // 密码/凭证：完全隐藏
        default_masking.insert(
            PiiType::Credential,
            MaskingConfig {
                pii_type: PiiType::Credential,
                strategy: MaskingStrategy::FullMask,
                reversible: false,
                keep_prefix: 0,
                keep_suffix: 0,
            },
        );

        Self {
            auto_detect_pii: true,
            auto_mask: true,
            default_masking,
            audit_retention_days: 90,
        }
    }
}

/// 影子模式引擎
pub struct ShadowModeEngine {
    config: ShadowModeConfig,
    /// SOSA加密引擎
    crypto: Arc<SosaCryptoEngine>,
    /// PII检测器（正则表达式）
    detectors: HashMap<PiiType, Regex>,
    /// 脱敏缓存（指纹 -> 脱敏数据）
    masked_cache: Arc<RwLock<HashMap<String, MaskedData>>>,
    /// 审计日志
    audit_log: Arc<RwLock<Vec<AccessAudit>>>,
}

impl ShadowModeEngine {
    /// 创建新的影子模式引擎
    pub fn new(config: ShadowModeConfig, crypto: Arc<SosaCryptoEngine>) -> Self {
        info!("👤 Initializing Shadow Mode Engine");
        info!("    Auto-detect PII: {}", config.auto_detect_pii);
        info!("    Auto-mask: {}", config.auto_mask);

        let mut detectors = HashMap::new();

        // Email检测器
        detectors.insert(
            PiiType::Email,
            Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
        );

        // 电话检测器（中国手机号）
        detectors.insert(
            PiiType::Phone,
            Regex::new(r"1[3-9]\d{9}").unwrap(),
        );

        // 身份证检测器（中国18位身份证）
        detectors.insert(
            PiiType::IdCard,
            Regex::new(r"\d{17}[\dXx]").unwrap(),
        );

        // 信用卡检测器（13-19位数字）
        detectors.insert(
            PiiType::CreditCard,
            Regex::new(r"\b\d{13,19}\b").unwrap(),
        );

        // IP地址检测器
        detectors.insert(
            PiiType::IpAddress,
            Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
        );

        Self {
            config,
            crypto,
            detectors,
            masked_cache: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 检测文本中的PII
    pub fn detect_pii(&self, text: &str) -> Vec<PiiDetection> {
        if !self.config.auto_detect_pii {
            return Vec::new();
        }

        let mut detections = Vec::new();

        for (pii_type, regex) in &self.detectors {
            for mat in regex.find_iter(text) {
                detections.push(PiiDetection {
                    pii_type: *pii_type,
                    matched_text: mat.as_str().to_string(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    confidence: 0.9, // 基于正则的检测置信度较高
                });
            }
        }

        if !detections.is_empty() {
            info!("🔍 Detected {} PII instances", detections.len());
        }

        detections
    }

    /// 脱敏数据
    pub async fn mask_data(
        &self,
        data: &str,
        pii_type: PiiType,
        config: Option<MaskingConfig>,
    ) -> Result<MaskedData> {
        let config = config.unwrap_or_else(|| {
            self.config
                .default_masking
                .get(&pii_type)
                .cloned()
                .unwrap_or_default()
        });

        let fingerprint = self.calculate_fingerprint(data);

        // 检查缓存
        let cache = self.masked_cache.read().await;
        if let Some(cached) = cache.get(&fingerprint) {
            debug!("📦 Using cached masked data");
            return Ok(cached.clone());
        }
        drop(cache);

        let masked_value = match config.strategy {
            MaskingStrategy::FullMask => "*".repeat(data.len()),

            MaskingStrategy::PartialMask => {
                let len = data.len();
                if len <= config.keep_prefix + config.keep_suffix {
                    "*".repeat(len)
                } else {
                    let prefix = &data[..config.keep_prefix];
                    let suffix = &data[len - config.keep_suffix..];
                    let mask_len = len - config.keep_prefix - config.keep_suffix;
                    format!("{}{}{}", prefix, "*".repeat(mask_len), suffix)
                }
            }

            MaskingStrategy::Hash => {
                // TODO: 使用 sha2 crate
                format!("hash_{}", fingerprint)
            }

            MaskingStrategy::Encrypt => {
                // 使用SOSA加密
                let encrypted = self.crypto.encrypt(data.as_bytes(), None).await?;
                format!("encrypted_{}", encrypted.key_id)
            }

            MaskingStrategy::Randomize => {
                // TODO: 生成同格式的随机数据
                "*".repeat(data.len())
            }

            MaskingStrategy::Generalize => {
                // TODO: 实现泛化逻辑
                "[REDACTED]".to_string()
            }
        };

        let encrypted = if config.strategy == MaskingStrategy::Encrypt {
            Some(self.crypto.encrypt(data.as_bytes(), None).await?)
        } else {
            None
        };

        let masked = MaskedData {
            fingerprint: fingerprint.clone(),
            masked_value,
            strategy: config.strategy,
            reversible: config.reversible,
            encrypted,
            masked_at: Utc::now(),
        };

        // 缓存
        let mut cache = self.masked_cache.write().await;
        cache.insert(fingerprint, masked.clone());

        info!("🎭 Masked data using {:?} strategy", config.strategy);

        Ok(masked)
    }

    /// 解除脱敏（仅限可逆策略）
    pub async fn unmask_data(
        &self,
        masked: &MaskedData,
        accessor: &str,
    ) -> Result<String> {
        // 审计检查
        let allowed = self.check_access_permission(accessor, &masked.fingerprint).await?;

        let audit = AccessAudit {
            audit_id: format!("audit_{}", Utc::now().timestamp_millis()),
            accessor: accessor.to_string(),
            data_fingerprint: masked.fingerprint.clone(),
            pii_type: PiiType::Other, // TODO: 从缓存中获取类型
            operation: "unmask".to_string(),
            allowed,
            deny_reason: if !allowed {
                Some("Access denied by policy".to_string())
            } else {
                None
            },
            accessed_at: Utc::now(),
        };

        // 记录审计
        let mut log = self.audit_log.write().await;
        log.push(audit);
        drop(log);

        if !allowed {
            return Err(anyhow!("Access denied: insufficient permissions"));
        }

        if !masked.reversible {
            return Err(anyhow!("Data is not reversible"));
        }

        match masked.strategy {
            MaskingStrategy::Encrypt => {
                if let Some(encrypted) = &masked.encrypted {
                    let decrypted = self.crypto.decrypt(encrypted).await?;
                    Ok(String::from_utf8(decrypted)?)
                } else {
                    Err(anyhow!("Encrypted data not found"))
                }
            }
            _ => Err(anyhow!("Strategy {:?} is not reversible", masked.strategy)),
        }
    }

    /// 自动处理文本中的PII
    pub async fn auto_process_text(&self, text: &str) -> Result<String> {
        if !self.config.auto_mask {
            return Ok(text.to_string());
        }

        let detections = self.detect_pii(text);
        if detections.is_empty() {
            return Ok(text.to_string());
        }

        let mut processed = text.to_string();
        // 从后向前替换，避免索引偏移
        for detection in detections.iter().rev() {
            let masked = self
                .mask_data(&detection.matched_text, detection.pii_type, None)
                .await?;

            processed.replace_range(
                detection.start_pos..detection.end_pos,
                &masked.masked_value,
            );
        }

        info!("🛡️  Auto-processed {} PII instances", detections.len());
        Ok(processed)
    }

    /// 获取审计日志
    pub async fn get_audit_log(&self, limit: Option<usize>) -> Vec<AccessAudit> {
        let log = self.audit_log.read().await;
        let limit = limit.unwrap_or(100);
        log.iter().rev().take(limit).cloned().collect()
    }

    // ===== 内部辅助方法 =====

    fn calculate_fingerprint(&self, data: &str) -> String {
        // TODO: 使用 sha2 crate 计算真实SHA256
        format!("fp_{}", data.len())
    }

    async fn check_access_permission(&self, _accessor: &str, _fingerprint: &str) -> Result<bool> {
        // TODO: 实现实际的权限检查逻辑
        // 可以集成 auth_system.rs 的权限系统
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sosa_crypto::{SosaCryptoConfig, SosaCryptoEngine};

    #[tokio::test]
    async fn test_pii_detection() {
        let crypto = Arc::new(SosaCryptoEngine::new(SosaCryptoConfig::default()));
        let engine = ShadowModeEngine::new(ShadowModeConfig::default(), crypto);

        let text = "My email is test@example.com and phone is 13812345678";
        let detections = engine.detect_pii(text);

        assert_eq!(detections.len(), 2);
        assert_eq!(detections[0].pii_type, PiiType::Email);
        assert_eq!(detections[1].pii_type, PiiType::Phone);
    }

    #[tokio::test]
    async fn test_masking() {
        let crypto = Arc::new(SosaCryptoEngine::new(SosaCryptoConfig::default()));
        let engine = ShadowModeEngine::new(ShadowModeConfig::default(), crypto);

        let masked = engine
            .mask_data("test@example.com", PiiType::Email, None)
            .await
            .unwrap();

        // 应该保留前3个字符
        assert!(masked.masked_value.starts_with("tes"));
        assert!(masked.masked_value.contains('*'));
    }
}
