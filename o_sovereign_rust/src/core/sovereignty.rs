//! 主权模式与认知疫苗模块 (Sovereignty Mode & Cognitive Vaccine)
//!
//! 实现 ACSA 认知疫苗系统,包括:
//! - 暴露剂量计 (DoseMeter) - 检测认知病毒载量
//! - 执行权熔断 (ExecCircuit Breaker) - 阻断完全外包
//! - 主体性训练 (Agentic Gym) - 恢复独立能力
//! - H(t) 生物活性计算 - ACSA 指数衰减定律
//!
//! 基于论文: 《ACSA 指数级衰减定律》与《自由之锁》
//!
//! 注意: 所有功能默认关闭,尊重用户自由意志选择权

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 主权模式全局实例
pub static SOVEREIGNTY: LazyLock<SovereigntySystem> =
    LazyLock::new(|| SovereigntySystem::new());

// ============================================================================
// 配置与数据结构
// ============================================================================

/// 主权模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyConfig {
    /// 主权模式开关 (默认: false, 尊重自由意志)
    #[serde(default)]
    pub enabled: bool,

    /// 依赖系数 λ (lambda) - 决定衰减速率
    #[serde(default = "default_lambda")]
    pub lambda: f64,

    /// 初始智慧基线 H₀
    #[serde(default = "default_h0")]
    pub initial_wisdom: f64,

    /// 熔断触发阈值
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// 是否显示 H(t) 警告 (默认: false, 不主动打扰)
    #[serde(default)]
    pub show_warnings: bool,
}

fn default_lambda() -> f64 {
    0.15 // 中等依赖系数
}

fn default_h0() -> f64 {
    100.0 // 满分基线
}

impl Default for SovereigntyConfig {
    fn default() -> Self {
        Self {
            enabled: false, // 默认关闭,让用户自己选择
            lambda: default_lambda(),
            initial_wisdom: default_h0(),
            circuit_breaker: CircuitBreakerConfig::default(),
            show_warnings: false,
        }
    }
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 连续外包决策触发阈值
    #[serde(default = "default_consecutive_delegations")]
    pub consecutive_delegations_threshold: usize,

    /// 无思考停顿时间阈值 (秒)
    #[serde(default = "default_no_thinking_threshold")]
    pub no_thinking_threshold_secs: i64,

    /// 一键确认比例阈值 (0.0-1.0)
    #[serde(default = "default_auto_confirm_threshold")]
    pub auto_confirm_threshold: f64,
}

fn default_consecutive_delegations() -> usize {
    5
}

fn default_no_thinking_threshold() -> i64 {
    1200 // 20 分钟
}

fn default_auto_confirm_threshold() -> f64 {
    0.8 // 80%
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            consecutive_delegations_threshold: default_consecutive_delegations(),
            no_thinking_threshold_secs: default_no_thinking_threshold(),
            auto_confirm_threshold: default_auto_confirm_threshold(),
        }
    }
}

// ============================================================================
// 决策类型与事件
// ============================================================================

/// 决策类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionType {
    /// 用户独立思考后决策
    Independent,
    /// 部分辅助 (用户提供约束条件)
    Assisted,
    /// 完全外包给 AI
    FullyDelegated,
    /// 一键确认 (无思考)
    AutoConfirmed,
}

/// 决策事件记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvent {
    pub timestamp: DateTime<Utc>,
    pub decision_type: DecisionType,
    /// 需求表达长度 (字符数)
    pub prompt_length: usize,
    /// 思考时间 (秒)
    pub thinking_time_secs: i64,
    /// 是否遇到困难立即求助
    pub gave_up_on_difficulty: bool,
}

// ============================================================================
// H(t) 生物活性计算
// ============================================================================

/// 生物活性指标 (Independent Bio-Activity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioActivity {
    /// H(t) 当前值
    pub current: f64,

    /// H₀ 初始基线
    pub baseline: f64,

    /// 衰减率 (%)
    pub decay_rate: f64,

    /// 风险等级
    pub risk_level: RiskLevel,

    /// 计算时间
    pub calculated_at: DateTime<Utc>,
}

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 健康 (H > 80)
    Healthy,
    /// 警告 (60 < H ≤ 80)
    Warning,
    /// 危险 (40 < H ≤ 60)
    Danger,
    /// 严重 (20 < H ≤ 40)
    Critical,
    /// 濒临线粒体化 (H ≤ 20)
    Mitochondrial,
}

impl BioActivity {
    /// 计算 H(t) = H₀ · e^(-λ · N(t) · t)
    pub fn calculate(
        h0: f64,
        lambda: f64,
        node_density: f64,
        time_hours: f64,
    ) -> Self {
        let exponent = -lambda * node_density * time_hours;
        let current = h0 * exponent.exp();
        let decay_rate = ((h0 - current) / h0) * 100.0;

        let risk_level = if current > 80.0 {
            RiskLevel::Healthy
        } else if current > 60.0 {
            RiskLevel::Warning
        } else if current > 40.0 {
            RiskLevel::Danger
        } else if current > 20.0 {
            RiskLevel::Critical
        } else {
            RiskLevel::Mitochondrial
        };

        Self {
            current,
            baseline: h0,
            decay_rate,
            risk_level,
            calculated_at: Utc::now(),
        }
    }
}

// ============================================================================
// 暴露剂量计 (DoseMeter)
// ============================================================================

/// 暴露剂量计 - 检测认知病毒载量
#[derive(Debug)]
pub struct DoseMeter {
    /// 决策事件历史 (保留最近 1000 条)
    events: Arc<RwLock<VecDeque<DecisionEvent>>>,

    /// 首次使用时间
    first_use: Arc<RwLock<Option<DateTime<Utc>>>>,

    /// 配置
    config: Arc<RwLock<SovereigntyConfig>>,
}

impl DoseMeter {
    pub fn new(config: SovereigntyConfig) -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            first_use: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// 记录决策事件
    pub async fn record_decision(&self, event: DecisionEvent) {
        // 记录首次使用时间
        {
            let mut first_use = self.first_use.write().await;
            if first_use.is_none() {
                *first_use = Some(Utc::now());
            }
        }

        // 添加事件
        let mut events = self.events.write().await;
        if events.len() >= 1000 {
            events.pop_front();
        }
        events.push_back(event);

        debug!("📊 Decision recorded, total events: {}", events.len());
    }

    /// 计算节点密度 N(t) - 基于外包决策比例
    pub async fn calculate_node_density(&self) -> f64 {
        let events = self.events.read().await;
        if events.is_empty() {
            return 0.0;
        }

        let delegated_count = events
            .iter()
            .filter(|e| {
                matches!(
                    e.decision_type,
                    DecisionType::FullyDelegated | DecisionType::AutoConfirmed
                )
            })
            .count();

        delegated_count as f64 / events.len() as f64
    }

    /// 计算使用时长 (小时)
    pub async fn calculate_usage_hours(&self) -> f64 {
        let first_use = self.first_use.read().await;
        match *first_use {
            Some(start) => {
                let duration = Utc::now().signed_duration_since(start);
                duration.num_seconds() as f64 / 3600.0
            }
            None => 0.0,
        }
    }

    /// 计算 H(t) 生物活性
    pub async fn calculate_bio_activity(&self) -> BioActivity {
        let config = self.config.read().await;
        let node_density = self.calculate_node_density().await;
        let time_hours = self.calculate_usage_hours().await;

        BioActivity::calculate(
            config.initial_wisdom,
            config.lambda,
            node_density,
            time_hours,
        )
    }

    /// 获取最近 N 天的统计
    pub async fn get_recent_stats(&self, days: i64) -> DoseStats {
        let events = self.events.read().await;
        let cutoff = Utc::now() - Duration::days(days);

        let recent: Vec<_> = events
            .iter()
            .filter(|e| e.timestamp > cutoff)
            .cloned()
            .collect();

        if recent.is_empty() {
            return DoseStats::default();
        }

        let total = recent.len();
        let delegated = recent
            .iter()
            .filter(|e| {
                matches!(
                    e.decision_type,
                    DecisionType::FullyDelegated | DecisionType::AutoConfirmed
                )
            })
            .count();

        let auto_confirmed = recent
            .iter()
            .filter(|e| e.decision_type == DecisionType::AutoConfirmed)
            .count();

        let avg_prompt_length = recent.iter().map(|e| e.prompt_length).sum::<usize>()
            / total.max(1);

        let gave_up_count = recent.iter().filter(|e| e.gave_up_on_difficulty).count();

        DoseStats {
            total_decisions: total,
            delegation_ratio: delegated as f64 / total as f64,
            auto_confirm_ratio: auto_confirmed as f64 / total as f64,
            avg_prompt_length,
            failure_intolerance_ratio: gave_up_count as f64 / total as f64,
        }
    }
}

/// 剂量统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DoseStats {
    /// 总决策次数
    pub total_decisions: usize,
    /// 外包决策比例
    pub delegation_ratio: f64,
    /// 一键确认比例
    pub auto_confirm_ratio: f64,
    /// 平均需求表达长度
    pub avg_prompt_length: usize,
    /// 失败不耐受比例 (遇到困难立即求助)
    pub failure_intolerance_ratio: f64,
}

// ============================================================================
// 执行权熔断器 (ExecCircuit Breaker)
// ============================================================================

/// 执行权熔断器 - 防止完全线粒体化
#[derive(Debug)]
pub struct ExecCircuitBreaker {
    dose_meter: Arc<DoseMeter>,
    config: Arc<RwLock<SovereigntyConfig>>,
}

impl ExecCircuitBreaker {
    pub fn new(dose_meter: Arc<DoseMeter>, config: Arc<RwLock<SovereigntyConfig>>) -> Self {
        Self {
            dose_meter,
            config,
        }
    }

    /// 检查是否应该触发熔断
    pub async fn should_trigger(&self) -> Option<CircuitBreakReason> {
        let config = self.config.read().await;
        if !config.enabled {
            return None; // 主权模式未开启,不熔断
        }

        let events = self.dose_meter.events.read().await;
        if events.is_empty() {
            return None;
        }

        // 检查连续外包
        let recent: Vec<_> = events.iter().rev().take(10).collect();
        let consecutive_delegated = recent
            .iter()
            .take_while(|e| {
                matches!(
                    e.decision_type,
                    DecisionType::FullyDelegated | DecisionType::AutoConfirmed
                )
            })
            .count();

        if consecutive_delegated >= config.circuit_breaker.consecutive_delegations_threshold {
            return Some(CircuitBreakReason::ConsecutiveDelegation(
                consecutive_delegated,
            ));
        }

        // 检查无思考高速外包
        if let Some(last_event) = events.back() {
            let duration_since_last = Utc::now()
                .signed_duration_since(last_event.timestamp)
                .num_seconds();

            if duration_since_last < 5
                && last_event.decision_type == DecisionType::FullyDelegated
            {
                // 检查是否连续高速外包
                let high_speed_count = recent
                    .windows(2)
                    .filter(|pair| {
                        let time_diff = pair[0]
                            .timestamp
                            .signed_duration_since(pair[1].timestamp)
                            .num_seconds();
                        time_diff < 10
                            && matches!(
                                pair[0].decision_type,
                                DecisionType::FullyDelegated
                            )
                    })
                    .count();

                if high_speed_count >= 3 {
                    return Some(CircuitBreakReason::HighSpeedOutsourcing);
                }
            }
        }

        // 检查一键确认比例
        let stats = self.dose_meter.get_recent_stats(1).await;
        if stats.total_decisions >= 10
            && stats.auto_confirm_ratio > config.circuit_breaker.auto_confirm_threshold
        {
            return Some(CircuitBreakReason::ExcessiveAutoConfirm(
                stats.auto_confirm_ratio,
            ));
        }

        None
    }

    /// 执行熔断 - 返回引导消息
    pub async fn execute_break(
        &self,
        reason: CircuitBreakReason,
    ) -> Result<String> {
        warn!("⚠️  Sovereignty circuit breaker triggered: {:?}", reason);

        let message = match reason {
            CircuitBreakReason::ConsecutiveDelegation(count) => {
                format!(
                    "🛡️  主权保护已触发: 检测到连续 {} 次完全外包决策。\n\
                     为了维持您的生物活性 H(t),请尝试独立拆解下一个任务。\n\
                     提示: 请先列出 3 个关键步骤或约束条件。",
                    count
                )
            }
            CircuitBreakReason::HighSpeedOutsourcing => {
                "🛡️  主权保护已触发: 检测到高速连续外包模式。\n\
                 建议: 暂停 60 秒,思考一下真正的需求是什么。"
                    .to_string()
            }
            CircuitBreakReason::ExcessiveAutoConfirm(ratio) => {
                format!(
                    "🛡️  主权保护已触发: 一键确认比例过高 ({:.1}%)。\n\
                     请对下一个决策进行二选一,而不是直接确认。",
                    ratio * 100.0
                )
            }
        };

        Ok(message)
    }
}

/// 熔断触发原因
#[derive(Debug, Clone)]
pub enum CircuitBreakReason {
    /// 连续外包决策
    ConsecutiveDelegation(usize),
    /// 高速外包模式 (无思考停顿)
    HighSpeedOutsourcing,
    /// 一键确认比例过高
    ExcessiveAutoConfirm(f64),
}

// ============================================================================
// 主权系统 (Sovereignty System)
// ============================================================================

/// 主权系统 - 整合所有疫苗功能
pub struct SovereigntySystem {
    config: Arc<RwLock<SovereigntyConfig>>,
    dose_meter: Arc<DoseMeter>,
    circuit_breaker: Arc<RwLock<Option<ExecCircuitBreaker>>>,
}

impl SovereigntySystem {
    pub fn new() -> Self {
        let config = Arc::new(RwLock::new(SovereigntyConfig::default()));
        let dose_meter = Arc::new(DoseMeter::new(SovereigntyConfig::default()));

        Self {
            config: config.clone(),
            dose_meter: dose_meter.clone(),
            circuit_breaker: Arc::new(RwLock::new(None)),
        }
    }

    /// 初始化系统
    pub async fn initialize(&self, config: SovereigntyConfig) -> Result<()> {
        *self.config.write().await = config.clone();

        if config.enabled {
            let breaker = ExecCircuitBreaker::new(
                self.dose_meter.clone(),
                self.config.clone(),
            );
            *self.circuit_breaker.write().await = Some(breaker);
            info!("🛡️  Sovereignty mode enabled (respecting free will)");
        } else {
            *self.circuit_breaker.write().await = None;
            debug!("ℹ️  Sovereignty mode disabled (user choice)");
        }

        Ok(())
    }

    /// 记录决策
    pub async fn record_decision(&self, event: DecisionEvent) {
        self.dose_meter.record_decision(event).await;
    }

    /// 获取 H(t) 生物活性
    pub async fn get_bio_activity(&self) -> BioActivity {
        self.dose_meter.calculate_bio_activity().await
    }

    /// 获取剂量统计
    pub async fn get_dose_stats(&self, days: i64) -> DoseStats {
        self.dose_meter.get_recent_stats(days).await
    }

    /// 检查是否需要熔断
    pub async fn check_circuit_break(&self) -> Option<String> {
        let breaker = self.circuit_breaker.read().await;
        if let Some(ref breaker) = *breaker {
            if let Some(reason) = breaker.should_trigger().await {
                return breaker.execute_break(reason).await.ok();
            }
        }
        None
    }

    /// 是否启用主权模式
    pub async fn is_enabled(&self) -> bool {
        self.config.read().await.enabled
    }
}

impl Default for SovereigntySystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 生成 H(t) 趋势报告
pub async fn generate_bio_activity_report() -> String {
    let activity = SOVEREIGNTY.get_bio_activity().await;
    let stats = SOVEREIGNTY.get_dose_stats(7).await;

    let risk_emoji = match activity.risk_level {
        RiskLevel::Healthy => "✅",
        RiskLevel::Warning => "⚠️",
        RiskLevel::Danger => "🔶",
        RiskLevel::Critical => "🔴",
        RiskLevel::Mitochondrial => "💀",
    };

    format!(
        r#"
📊 生物活性报告 (Bio-Activity Report)

H(t) 当前值: {:.2} / {:.2} {}
衰减率: {:.1}%
风险等级: {:?}

最近 7 天统计:
├─ 总决策数: {}
├─ 外包比例: {:.1}%
├─ 一键确认: {:.1}%
├─ 平均表达长度: {} 字符
└─ 失败不耐受: {:.1}%

计算时间: {}
"#,
        activity.current,
        activity.baseline,
        risk_emoji,
        activity.decay_rate,
        activity.risk_level,
        stats.total_decisions,
        stats.delegation_ratio * 100.0,
        stats.auto_confirm_ratio * 100.0,
        stats.avg_prompt_length,
        stats.failure_intolerance_ratio * 100.0,
        activity.calculated_at.format("%Y-%m-%d %H:%M:%S UTC"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bio_activity_calculation() {
        let h0 = 100.0;
        let lambda = 0.15;
        let node_density = 0.8; // 80% 外包
        let time_hours = 100.0; // 100 小时

        let activity = BioActivity::calculate(h0, lambda, node_density, time_hours);

        assert!(activity.current < h0);
        assert!(activity.decay_rate > 0.0);
        println!("H(t) = {:.2}, decay = {:.1}%", activity.current, activity.decay_rate);
    }

    #[tokio::test]
    async fn test_dose_meter() {
        let config = SovereigntyConfig::default();
        let meter = DoseMeter::new(config);

        // 记录一些决策
        for i in 0..10 {
            let event = DecisionEvent {
                timestamp: Utc::now(),
                decision_type: if i < 7 {
                    DecisionType::FullyDelegated
                } else {
                    DecisionType::Independent
                },
                prompt_length: 50,
                thinking_time_secs: 5,
                gave_up_on_difficulty: false,
            };
            meter.record_decision(event).await;
        }

        let stats = meter.get_recent_stats(7).await;
        assert_eq!(stats.total_decisions, 10);
        assert!(stats.delegation_ratio > 0.6);
    }
}
