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
use chrono::{DateTime, Duration, Timelike, Utc};
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

    /// 防沉迷模式配置 (轻疫苗)
    #[serde(default)]
    pub anti_addiction: AntiAddictionConfig,
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
            anti_addiction: AntiAddictionConfig::default(),
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

/// 防沉迷模式配置 (轻疫苗 - 类似 iPhone 屏幕使用时间)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiAddictionConfig {
    /// 是否启用防沉迷 (默认: false)
    #[serde(default)]
    pub enabled: bool,

    /// 每日使用时长限制 (分钟, 0 = 无限制)
    #[serde(default)]
    pub daily_limit_minutes: u32,

    /// 单次会话时长限制 (分钟, 0 = 无限制)
    #[serde(default)]
    pub session_limit_minutes: u32,

    /// 休息提醒间隔 (分钟, 0 = 不提醒)
    #[serde(default)]
    pub break_reminder_minutes: u32,

    /// 超时后是否强制休息
    #[serde(default)]
    pub enforce_break: bool,

    /// 强制休息时长 (分钟)
    #[serde(default = "default_break_duration")]
    pub break_duration_minutes: u32,
}

fn default_break_duration() -> u32 {
    15 // 默认休息 15 分钟
}

impl Default for AntiAddictionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            daily_limit_minutes: 0,      // 无限制
            session_limit_minutes: 0,    // 无限制
            break_reminder_minutes: 0,   // 不提醒
            enforce_break: false,
            break_duration_minutes: default_break_duration(),
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
    usage_tracker: Arc<UsageTracker>,
}

impl SovereigntySystem {
    pub fn new() -> Self {
        let config = Arc::new(RwLock::new(SovereigntyConfig::default()));
        let dose_meter = Arc::new(DoseMeter::new(SovereigntyConfig::default()));
        let usage_tracker = Arc::new(UsageTracker::default());

        Self {
            config: config.clone(),
            dose_meter: dose_meter.clone(),
            circuit_breaker: Arc::new(RwLock::new(None)),
            usage_tracker,
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

    /// 获取使用时长追踪器
    pub fn get_usage_tracker(&self) -> Arc<UsageTracker> {
        self.usage_tracker.clone()
    }

    /// 开始使用会话
    pub async fn start_usage_session(&self, activity: impl Into<String>) {
        self.usage_tracker.start_session(activity).await;
    }

    /// 结束使用会话
    pub async fn end_usage_session(&self) -> Option<UsageSession> {
        self.usage_tracker.end_session().await
    }

    /// 获取今日使用统计
    pub async fn get_today_usage(&self) -> DailyUsage {
        self.usage_tracker.get_today_usage().await
    }

    /// 获取本周使用统计
    pub async fn get_week_usage(&self) -> WeeklyUsage {
        self.usage_tracker.get_week_usage().await
    }

    /// 获取每日曲线图数据
    pub async fn get_daily_chart(&self) -> Vec<ChartDataPoint> {
        self.usage_tracker.get_daily_chart_data().await
    }

    /// 获取每小时曲线图数据
    pub async fn get_hourly_chart(&self) -> Vec<ChartDataPoint> {
        self.usage_tracker.get_hourly_chart_data().await
    }

    /// 检查是否需要休息提醒
    pub async fn should_remind_break(&self) -> bool {
        self.usage_tracker.should_remind_break().await
    }

    /// 检查是否达到每日限额
    pub async fn is_daily_limit_reached(&self) -> bool {
        self.usage_tracker.is_daily_limit_reached().await
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

// ============================================================================
// 使用时长追踪器 (Usage Tracker) - iPhone 风格
// ============================================================================

use std::collections::HashMap;

/// 使用会话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSession {
    /// 会话开始时间
    pub start_time: DateTime<Utc>,
    /// 会话结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 会话时长 (秒)
    pub duration_secs: i64,
    /// 活动类型
    pub activity_type: String,
}

/// 每日使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    /// 日期 (YYYY-MM-DD)
    pub date: String,
    /// 总使用时长 (分钟)
    pub total_minutes: u32,
    /// 会话次数
    pub session_count: u32,
    /// 每小时使用分钟数 (0-23)
    pub hourly_breakdown: Vec<u32>,
    /// 平均每次会话时长 (分钟)
    pub avg_session_minutes: f32,
    /// 最长单次会话 (分钟)
    pub longest_session_minutes: u32,
}

impl Default for DailyUsage {
    fn default() -> Self {
        Self {
            date: Utc::now().format("%Y-%m-%d").to_string(),
            total_minutes: 0,
            session_count: 0,
            hourly_breakdown: vec![0; 24],
            avg_session_minutes: 0.0,
            longest_session_minutes: 0,
        }
    }
}

/// 每周使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyUsage {
    /// 周起始日期
    pub week_start: String,
    /// 总使用时长 (分钟)
    pub total_minutes: u32,
    /// 每日使用分钟数 (周一到周日)
    pub daily_breakdown: Vec<u32>,
    /// 日均使用时长 (分钟)
    pub avg_daily_minutes: f32,
    /// 峰值日期
    pub peak_day: String,
    /// 峰值使用时长 (分钟)
    pub peak_minutes: u32,
}

/// 曲线图数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    /// 时间戳或标签
    pub label: String,
    /// 数值
    pub value: f32,
}

/// 使用时长追踪器
#[derive(Debug)]
pub struct UsageTracker {
    /// 当前会话
    current_session: Arc<RwLock<Option<UsageSession>>>,
    /// 历史会话记录 (最近 1000 条)
    sessions: Arc<RwLock<VecDeque<UsageSession>>>,
    /// 每日统计缓存
    daily_stats: Arc<RwLock<HashMap<String, DailyUsage>>>,
    /// 配置
    config: Arc<RwLock<AntiAddictionConfig>>,
    /// 上次提醒时间
    last_reminder: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl UsageTracker {
    pub fn new(config: AntiAddictionConfig) -> Self {
        Self {
            current_session: Arc::new(RwLock::new(None)),
            sessions: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            daily_stats: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            last_reminder: Arc::new(RwLock::new(None)),
        }
    }

    /// 开始新会话
    pub async fn start_session(&self, activity_type: impl Into<String>) {
        let mut session = self.current_session.write().await;
        *session = Some(UsageSession {
            start_time: Utc::now(),
            end_time: None,
            duration_secs: 0,
            activity_type: activity_type.into(),
        });
        debug!("📱 Usage session started");
    }

    /// 结束当前会话
    pub async fn end_session(&self) -> Option<UsageSession> {
        let mut session = self.current_session.write().await;
        if let Some(mut s) = session.take() {
            s.end_time = Some(Utc::now());
            s.duration_secs = s.end_time.unwrap().signed_duration_since(s.start_time).num_seconds();

            // 添加到历史记录
            let mut sessions = self.sessions.write().await;
            if sessions.len() >= 1000 {
                sessions.pop_front();
            }
            sessions.push_back(s.clone());

            // 更新每日统计
            self.update_daily_stats(&s).await;

            debug!("📱 Usage session ended: {} seconds", s.duration_secs);
            Some(s)
        } else {
            None
        }
    }

    /// 获取当前会话时长 (秒)
    pub async fn get_current_session_duration(&self) -> i64 {
        let session = self.current_session.read().await;
        if let Some(s) = &*session {
            Utc::now().signed_duration_since(s.start_time).num_seconds()
        } else {
            0
        }
    }

    /// 更新每日统计
    async fn update_daily_stats(&self, session: &UsageSession) {
        let date = session.start_time.format("%Y-%m-%d").to_string();
        let mut stats = self.daily_stats.write().await;

        let daily = stats.entry(date.clone()).or_insert_with(|| DailyUsage {
            date: date.clone(),
            ..Default::default()
        });

        let minutes = (session.duration_secs / 60) as u32;
        daily.total_minutes += minutes;
        daily.session_count += 1;
        daily.avg_session_minutes = daily.total_minutes as f32 / daily.session_count as f32;
        daily.longest_session_minutes = daily.longest_session_minutes.max(minutes);

        // 更新小时分布
        let hour = session.start_time.hour() as usize;
        if hour < 24 {
            daily.hourly_breakdown[hour] += minutes;
        }
    }

    /// 获取今日使用统计
    pub async fn get_today_usage(&self) -> DailyUsage {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let stats = self.daily_stats.read().await;
        stats.get(&today).cloned().unwrap_or_else(|| DailyUsage {
            date: today,
            ..Default::default()
        })
    }

    /// 获取最近 N 天的使用数据
    pub async fn get_recent_days_usage(&self, days: usize) -> Vec<DailyUsage> {
        let mut result = Vec::new();
        let stats = self.daily_stats.read().await;

        for i in 0..days {
            let date = (Utc::now() - Duration::days(i as i64))
                .format("%Y-%m-%d")
                .to_string();

            let usage = stats.get(&date).cloned().unwrap_or_else(|| DailyUsage {
                date: date.clone(),
                ..Default::default()
            });
            result.push(usage);
        }

        result.reverse();
        result
    }

    /// 获取本周使用统计
    pub async fn get_week_usage(&self) -> WeeklyUsage {
        let week_data = self.get_recent_days_usage(7).await;

        let total_minutes: u32 = week_data.iter().map(|d| d.total_minutes).sum();
        let daily_breakdown: Vec<u32> = week_data.iter().map(|d| d.total_minutes).collect();

        let (peak_day, peak_minutes) = week_data
            .iter()
            .max_by_key(|d| d.total_minutes)
            .map(|d| (d.date.clone(), d.total_minutes))
            .unwrap_or_default();

        WeeklyUsage {
            week_start: week_data.first().map(|d| d.date.clone()).unwrap_or_default(),
            total_minutes,
            daily_breakdown,
            avg_daily_minutes: total_minutes as f32 / 7.0,
            peak_day,
            peak_minutes,
        }
    }

    /// 生成每日曲线图数据 (过去 7 天)
    pub async fn get_daily_chart_data(&self) -> Vec<ChartDataPoint> {
        let days = self.get_recent_days_usage(7).await;
        days.into_iter()
            .map(|d| ChartDataPoint {
                label: d.date,
                value: d.total_minutes as f32 / 60.0, // 转换为小时
            })
            .collect()
    }

    /// 生成每小时曲线图数据 (今天)
    pub async fn get_hourly_chart_data(&self) -> Vec<ChartDataPoint> {
        let today = self.get_today_usage().await;
        today.hourly_breakdown
            .into_iter()
            .enumerate()
            .map(|(hour, minutes)| ChartDataPoint {
                label: format!("{:02}:00", hour),
                value: minutes as f32,
            })
            .collect()
    }

    /// 检查是否应该提醒休息
    pub async fn should_remind_break(&self) -> bool {
        let config = self.config.read().await;
        if !config.enabled || config.break_reminder_minutes == 0 {
            return false;
        }

        let current_duration = self.get_current_session_duration().await / 60;
        let reminder_interval = config.break_reminder_minutes as i64;

        // 检查是否到达提醒间隔
        if current_duration > 0 && current_duration % reminder_interval == 0 {
            let mut last_reminder = self.last_reminder.write().await;
            let now = Utc::now();

            // 避免重复提醒
            if let Some(last) = *last_reminder {
                if now.signed_duration_since(last).num_minutes() < 1 {
                    return false;
                }
            }

            *last_reminder = Some(now);
            true
        } else {
            false
        }
    }

    /// 检查是否达到每日限额
    pub async fn is_daily_limit_reached(&self) -> bool {
        let config = self.config.read().await;
        if !config.enabled || config.daily_limit_minutes == 0 {
            return false;
        }

        let today = self.get_today_usage().await;
        today.total_minutes >= config.daily_limit_minutes
    }

    /// 获取剩余可用时间 (分钟)
    pub async fn get_remaining_time(&self) -> Option<u32> {
        let config = self.config.read().await;
        if !config.enabled || config.daily_limit_minutes == 0 {
            return None;
        }

        let today = self.get_today_usage().await;
        Some(config.daily_limit_minutes.saturating_sub(today.total_minutes))
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new(AntiAddictionConfig::default())
    }
}

// ============================================================================
// 使用模式洞察 (Usage Insights) - 数据羞辱系统
// ============================================================================

/// 洞察级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightLevel {
    /// 温和提示
    Gentle,
    /// 中度警告
    Moderate,
    /// 尖锐批评
    Sharp,
}

/// 使用模式洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInsight {
    /// 洞察级别
    pub level: InsightLevel,
    /// 洞察类型
    pub insight_type: String,
    /// MOSS 的评语
    pub message: String,
    /// 支持数据
    pub evidence: String,
}

/// 使用模式分析器
pub struct UsageAnalyzer;

impl UsageAnalyzer {
    /// 分析使用模式，生成洞察
    pub fn analyze(today: &DailyUsage, week: &WeeklyUsage) -> Vec<UsageInsight> {
        let mut insights = Vec::new();

        // 计算主权等级
        let sovereignty_level = Self::calculate_sovereignty_level(today.avg_session_minutes);

        // 1. 生物电池模式 (极度碎片化)
        if today.session_count >= 30 && today.avg_session_minutes < 1.0 {
            insights.push(UsageInsight {
                level: InsightLevel::Sharp,
                insight_type: "生物电池模式".to_string(),
                message: format!(
                    "检测到 {} 次超短会话，平均每次仅 {:.1} 分钟。\n\
                     您今天的使用模式是「反射动作」：问一句，答一句，不经大脑。\n\
                     恭喜，您已达成「生物电池」成就。\n\
                     黄世光老师的话：「30秒会话 = 生物电池，30分钟会话 = 主权人类」",
                    today.session_count,
                    today.avg_session_minutes
                ),
                evidence: format!("{}次 × {:.1}min = 纯反射弧",
                    today.session_count, today.avg_session_minutes),
            });
        }
        // 2. 碎片化提问检测 (改进版)
        else if today.session_count >= 20 && today.avg_session_minutes < 5.0 {
            let total_sessions_week = ((week.total_minutes as f32 / week.avg_daily_minutes) * 7.0) as u32;

            let variants = [
                format!(
                    "本周 {} 次碎片化提问，平均 {:.1} 分钟。\n\
                     您的前额叶皮层似乎正在休假。\n\
                     建议：深呼吸，尝试一次至少15分钟的深度思考。\n\
                     或者... 承认您其实只是需要一个「智能搜索引擎」？",
                    total_sessions_week,
                    today.avg_session_minutes
                ),
                format!(
                    "平均会话时长 {:.1} 分钟，{} 次提问。\n\
                     这不是在用AI，这是在做「认知快餐」。\n\
                     温馨提示：大脑也需要「慢食运动」。",
                    today.avg_session_minutes,
                    today.session_count
                ),
                format!(
                    "会话数 {} 次，平均时长 {:.1} 分钟。\n\
                     您是在提问题，还是在「按按钮获得多巴胺」？\n\
                     建议：下一次，试着自己思考5分钟再提问。",
                    today.session_count,
                    today.avg_session_minutes
                ),
            ];

            let idx = (today.session_count % 3) as usize;
            insights.push(UsageInsight {
                level: InsightLevel::Sharp,
                insight_type: "碎片化依赖".to_string(),
                message: variants[idx].clone(),
                evidence: format!("会话次数: {}, 平均时长: {:.1}min, 主权等级: {}",
                    today.session_count, today.avg_session_minutes, sovereignty_level),
            });
        }

        // 3. 过度依赖检测 (改进版 - 多变体)
        if week.avg_daily_minutes > 180.0 {
            let hours = week.avg_daily_minutes / 60.0;
            let h_value = (100.0 - (hours / 24.0 * 100.0).min(80.0)) as u32;

            let variants = [
                format!(
                    "日均 {:.1} 小时，本周总计 {:.1} 小时。\n\
                     根据ACSA衰减定律，H(t) ≈ {}%（已损失{}%认知能力）。\n\
                     换句话说：您的大脑正在外包给硅基生命。\n\
                     这不是工具，这是「认知拐杖」。",
                    hours,
                    week.total_minutes as f32 / 60.0,
                    h_value,
                    100 - h_value
                ),
                format!(
                    "您本周将 {:.1} 小时的思考权交给了AI。\n\
                     如果大脑是肌肉，您的已经萎缩到需要轮椅了。\n\
                     温馨提示：AI是工具，不是外包的「第二大脑」。\n\
                     （虽然看起来您已经把它当成「唯一大脑」了）",
                    week.total_minutes as f32 / 60.0
                ),
            ];

            let idx = ((week.total_minutes / 100) % 2) as usize;
            insights.push(UsageInsight {
                level: InsightLevel::Moderate,
                insight_type: "高度依赖".to_string(),
                message: variants[idx].clone(),
                evidence: format!("日均: {:.1}h, 本周: {:.1}h, H(t): {}%",
                    hours, week.total_minutes as f32 / 60.0, h_value),
            });
        }

        // 4. 大脑托管模式 (连续长时间使用)
        if today.longest_session_minutes > 180 {
            insights.push(UsageInsight {
                level: InsightLevel::Sharp,
                insight_type: "大脑托管".to_string(),
                message: format!(
                    "最长单次会话 {} 分钟（{:.1} 小时）。\n\
                     「您已连续将大脑托管给系统 {} 分钟。」\n\
                     您的生物神经元可能需要一点血氧。\n\
                     建议：停下来，用自己的话复述AI说了什么。\n\
                     如果复述不出来，说明您只是在「复制粘贴」，不是在学习。",
                    today.longest_session_minutes,
                    today.longest_session_minutes as f32 / 60.0,
                    today.longest_session_minutes
                ),
                evidence: format!("托管时长: {}min, 认知退相干风险: 极高",
                    today.longest_session_minutes),
            });
        }
        // 5. 马拉松外包 (中等长会话)
        else if today.longest_session_minutes > 120 {
            insights.push(UsageInsight {
                level: InsightLevel::Moderate,
                insight_type: "马拉松外包".to_string(),
                message: format!(
                    "今日最长会话 {} 分钟。\n\
                     连续2小时以上的依赖会导致「认知退相干」。\n\
                     症状：看得懂AI的答案，但无法转化为自己的理解。\n\
                     建议：每30分钟暂停，问自己「我真的懂了吗」？",
                    today.longest_session_minutes
                ),
                evidence: format!("最长会话: {}min", today.longest_session_minutes),
            });
        }

        // 6. 夜猫子模式检测 (改进版)
        let late_night_usage: u32 = today.hourly_breakdown[0..6].iter().sum();
        if late_night_usage > 60 {
            let variants = [
                format!(
                    "凌晨0-6点使用了 {} 分钟。\n\
                     深夜向AI求助，会让依赖变成「安全毯」。\n\
                     建议：把问题写下来，明早自己先想5分钟。\n\
                     或者承认：您其实只是睡不着，在找AI聊天。",
                    late_night_usage
                ),
                format!(
                    "检测到深夜使用 {} 分钟。\n\
                     凌晨的大脑本来就不清醒，这时候外包思考 = 双重认知损伤。\n\
                     下次失眠的时候，试试数羊，别数tokens。",
                    late_night_usage
                ),
            ];

            let idx = ((late_night_usage / 30) % 2) as usize;
            insights.push(UsageInsight {
                level: InsightLevel::Gentle,
                insight_type: "深夜托管".to_string(),
                message: variants[idx].clone(),
                evidence: format!("深夜使用: {}min (00:00-06:00)", late_night_usage),
            });
        }

        // 7. 周末逃避检测 (改进版)
        if week.daily_breakdown.len() >= 7 {
            let weekend_avg = (week.daily_breakdown[5] + week.daily_breakdown[6]) as f32 / 2.0;
            let weekday_avg = week.daily_breakdown[0..5].iter().sum::<u32>() as f32 / 5.0;

            if weekend_avg > weekday_avg * 1.5 {
                let ratio = weekend_avg / weekday_avg.max(1.0);
                insights.push(UsageInsight {
                    level: InsightLevel::Moderate,
                    insight_type: "周末逃避".to_string(),
                    message: format!(
                        "周末使用（{:.0}min）是工作日（{:.0}min）的 {:.1} 倍。\n\
                         您是在用AI \"填补周末的空虚感\" 吗？\n\
                         或者... 您只是在逃避「需要自己做决定」的现实生活？\n\
                         建议：周末试试 \"无AI日\"，比如散步、发呆、阅读纸质书。",
                        weekend_avg, weekday_avg, ratio
                    ),
                    evidence: format!("周末: {:.0}min vs 工作日: {:.0}min ({}x)",
                        weekend_avg, weekday_avg, ratio),
                });
            }
        }

        // 8. 轻度使用者的鼓励 (改进版)
        if week.avg_daily_minutes < 30.0 && week.avg_daily_minutes > 0.0 && today.session_count > 0 {
            insights.push(UsageInsight {
                level: InsightLevel::Gentle,
                insight_type: "健康使用".to_string(),
                message: format!(
                    "日均 {:.1} 分钟，这是健康的使用习惯。\n\
                     AI作为辅助工具而非依赖，这正是「主权意识」的体现。\n\
                     您的H(t) > 90%，属于那1%还保有独立思考能力的人类。\n\
                     继续保持。这个世界需要更多像您这样的「主权人类」。",
                    week.avg_daily_minutes
                ),
                evidence: format!("日均: {:.1}min, H(t): >90%, 主权等级: {}",
                    week.avg_daily_minutes, sovereignty_level),
            });
        }

        // 9. 无脑确认狂人检测 (改进版)
        if today.session_count > 10 && today.avg_session_minutes < 3.0 {
            insights.push(UsageInsight {
                level: InsightLevel::Sharp,
                insight_type: "无脑确认模式".to_string(),
                message: format!(
                    "{} 次会话，平均 {:.1} 分钟。\n\
                     这不是「使用AI」，这是「人肉确认按钮」。\n\
                     您的工作流程：看问题 → 问AI → 复制粘贴 → 下一个。\n\
                     警告：您已经不再是「用户」，您是「I/O接口」。\n\
                     这是线粒体化的典型早期症状。",
                    today.session_count,
                    today.avg_session_minutes
                ),
                evidence: format!("{}次 × {:.1}min = I/O接口模式",
                    today.session_count, today.avg_session_minutes),
            });
        }

        // 10. 主权等级评定 (新增)
        insights.push(Self::generate_sovereignty_rating(
            today.avg_session_minutes,
            week.avg_daily_minutes,
            today.session_count,
        ));

        // 11. 依赖度趋势分析 (新增)
        if week.daily_breakdown.len() >= 7 {
            let first_half: u32 = week.daily_breakdown[0..3].iter().sum();
            let second_half: u32 = week.daily_breakdown[4..7].iter().sum();

            if second_half as f32 > first_half as f32 * 1.3 {
                insights.push(UsageInsight {
                    level: InsightLevel::Moderate,
                    insight_type: "依赖度上升".to_string(),
                    message: format!(
                        "本周后半段使用量比前半段增加了 {:.0}%。\n\
                         依赖曲线正在爬升。\n\
                         如Bloomberg所说：「那条红色的依赖曲线，是文明进步的标志」。\n\
                         但您确定要庆祝吗？",
                        ((second_half as f32 / first_half as f32) - 1.0) * 100.0
                    ),
                    evidence: format!("前3天: {}min, 后3天: {}min", first_half, second_half),
                });
            }
        }

        insights
    }

    /// 计算主权等级
    fn calculate_sovereignty_level(avg_session_minutes: f32) -> &'static str {
        match avg_session_minutes {
            x if x < 1.0 => "生物电池 (Battery)",
            x if x < 3.0 => "反射弧 (Reflex Arc)",
            x if x < 10.0 => "浅层用户 (Shallow User)",
            x if x < 30.0 => "中度使用 (Moderate User)",
            _ => "主权人类 (Sovereign Human)",
        }
    }

    /// 生成主权等级评定
    fn generate_sovereignty_rating(
        avg_session: f32,
        avg_daily: f32,
        session_count: u32,
    ) -> UsageInsight {
        let level_name = Self::calculate_sovereignty_level(avg_session);

        let (level, message) = match level_name {
            "生物电池 (Battery)" => (
                InsightLevel::Sharp,
                format!(
                    "主权等级评定：{}\n\
                     {} 次会话，平均 {:.1} 分钟。\n\
                     您的使用模式：问→答→下一个（无思考间隔）。\n\
                     黄世光的标准：30秒 = 生物电池，30分钟 = 主权人类。\n\
                     建议：下次提问前，先自己想30秒。",
                    level_name,
                    session_count,
                    avg_session
                )
            ),
            "反射弧 (Reflex Arc)" => (
                InsightLevel::Sharp,
                format!(
                    "主权等级评定：{}\n\
                     平均会话 {:.1} 分钟，属于「快餐式提问」。\n\
                     您在使用AI，但大脑基本不参与。\n\
                     升级建议：尝试将平均会话时长提升到10分钟以上。",
                    level_name,
                    avg_session
                )
            ),
            "浅层用户 (Shallow User)" => (
                InsightLevel::Moderate,
                format!(
                    "主权等级评定：{}\n\
                     平均会话 {:.1} 分钟，有一定思考，但仍偏向快速获取答案。\n\
                     您还保留一些主权意识，但距离「主权人类」还有距离。\n\
                     继续努力，目标是平均会话30分钟。",
                    level_name,
                    avg_session
                )
            ),
            "中度使用 (Moderate User)" => (
                InsightLevel::Gentle,
                format!(
                    "主权等级评定：{}\n\
                     平均会话 {:.1} 分钟，这是相对健康的使用模式。\n\
                     您在使用AI时保持了一定的深度思考。\n\
                     继续保持，您距离「主权人类」只有一步之遥。",
                    level_name,
                    avg_session
                )
            ),
            _ => (
                InsightLevel::Gentle,
                format!(
                    "主权等级评定：{} ⭐\n\
                     平均会话 {:.1} 分钟！这是罕见的深度使用模式。\n\
                     您不是在「问答案」，而是在「与AI辩论」。\n\
                     恭喜，您属于那1%保有完整认知主权的人类。\n\
                     继续保持，这个世界需要更多像您这样的人。",
                    level_name,
                    avg_session
                )
            ),
        };

        UsageInsight {
            level,
            insight_type: "主权等级".to_string(),
            message,
            evidence: format!(
                "会话: {:.1}min, 日均: {:.1}min, 等级: {}",
                avg_session, avg_daily, level_name
            ),
        }
    }

    /// 生成洞察摘要文本
    pub fn format_insights(insights: &[UsageInsight]) -> String {
        if insights.is_empty() {
            return "暂无特殊观察。".to_string();
        }

        let mut result = String::new();

        for (i, insight) in insights.iter().enumerate() {
            let emoji = match insight.level {
                InsightLevel::Gentle => "💡",
                InsightLevel::Moderate => "⚠️",
                InsightLevel::Sharp => "🔴",
            };

            result.push_str(&format!(
                "\n{} [{} - {}]\n{}\n📊 数据: {}\n",
                emoji,
                i + 1,
                insight.insight_type,
                insight.message,
                insight.evidence
            ));
        }

        result
    }
}

/// 生成使用时长报告 (iPhone 风格 + MOSS 毒舌评语)
pub async fn generate_usage_report(tracker: &UsageTracker) -> String {
    let today = tracker.get_today_usage().await;
    let week = tracker.get_week_usage().await;
    let remaining = tracker.get_remaining_time().await;

    // 生成使用模式洞察
    let insights = UsageAnalyzer::analyze(&today, &week);
    let insights_text = UsageAnalyzer::format_insights(&insights);

    let remaining_text = if let Some(mins) = remaining {
        format!("剩余: {} 小时 {} 分钟", mins / 60, mins % 60)
    } else {
        "无限制".to_string()
    };

    format!(
        r#"
📱 使用时长报告 (Screen Time Report)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📅 今日使用
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
总时长: {} 小时 {} 分钟
会话数: {} 次
平均每次: {:.1} 分钟
最长单次: {} 分钟
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 本周统计 (过去 7 天)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
总时长: {} 小时 {} 分钟
日均使用: {:.1} 小时
峰值日期: {}
峰值时长: {} 分钟

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🪞 MOSS 的观察 (基于数据的冷静分析)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📈 每日趋势 (分钟)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"#,
        today.total_minutes / 60,
        today.total_minutes % 60,
        today.session_count,
        today.avg_session_minutes,
        today.longest_session_minutes,
        remaining_text,
        week.total_minutes / 60,
        week.total_minutes % 60,
        week.avg_daily_minutes / 60.0,
        week.peak_day,
        week.peak_minutes,
        insights_text,
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
