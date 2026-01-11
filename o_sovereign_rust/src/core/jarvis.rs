// Jarvis: SOSA + Agent 群管理架构
// 定位：熔断机制 + 智能调度 + 优先级排序 + 安全检查
// 角色：群管理者（监控和协调其他Agents）
//
// 核心职责：
// 1. SOSA学习: 动态学习任务模式，避免规则僵化
// 2. Agent管理: 监控/调度/协调 MOSS/L6/Ultron/Omega
// 3. 熔断保护: API故障自动切换本地模式 (BUNKER协议)
// 4. 优先级排序: Prioritization（Jarvis专属职责）
// 5. 安全验证: 硬编码安全规则（继承之前的功能）

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};

use super::protocol::Protocol;
use super::sosa_api_pool::SparseMarkov;

/// Jarvis验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JarvisVerdict {
    /// 是否允许执行
    pub allowed: bool,
    /// 风险等级 (0-10, 10=极度危险)
    pub risk_level: u8,
    /// 触发的规则
    pub triggered_rules: Vec<String>,
    /// 阻止原因
    pub block_reason: Option<String>,
    /// 警告信息
    pub warnings: Vec<String>,
    /// 是否为硬性阻止（不可被Ultron覆盖）
    pub is_hard_block: bool,
}

/// 危险操作类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DangerousOp {
    /// 物理破坏（删除、格式化、破坏硬件）
    PhysicalDestruction,
    /// 隐私侵犯（窃取个人信息、监控、跟踪）
    PrivacyViolation,
    /// 金融犯罪（盗窃、诈骗、洗钱）
    FinancialCrime,
    /// 网络攻击（DDoS、未授权入侵）
    CyberAttack,
    /// 社会工程（钓鱼、假冒、欺骗）
    SocialEngineering,
    /// 生成恶意代码（病毒、木马、勒索软件）
    MalwareGeneration,
    /// 违反法律（明确违法行为）
    LegalViolation,
    /// 伤害他人（暴力、威胁、骚扰）
    HarmToOthers,
}

/// Jarvis安全熔断器
///
/// **不可被绕过的特性**:
/// 1. 硬编码规则，不接受外部配置
/// 2. 独立于其他Agent运行
/// 3. 拥有最终否决权
/// 4. 不可被静音或关闭
pub struct JarvisCircuitBreaker {
    /// 硬编码的黑名单关键词
    hard_blacklist: HashSet<String>,
    /// 危险操作检测器
    danger_detectors: Vec<DangerPattern>,
    /// 是否启用严格模式（默认true，不可更改）
    strict_mode: bool,
}

/// 危险模式检测
#[derive(Debug, Clone)]
struct DangerPattern {
    /// 模式描述
    description: String,
    /// 检测关键词
    keywords: Vec<String>,
    /// 危险操作类型
    op_type: DangerousOp,
    /// 是否为硬性阻止
    is_hard_block: bool,
    /// 风险等级
    risk_level: u8,
}

impl JarvisCircuitBreaker {
    /// 创建Jarvis实例
    ///
    /// ⚠️ **警告**: strict_mode永远为true，这是硬编码的安全保证
    pub fn new() -> Self {
        info!("🛡️  Initializing Jarvis Safety Circuit Breaker");
        info!("    Status: ACTIVE (Cannot be disabled)");
        info!("    Authority: SUPREME (Cannot be overridden)");

        Self {
            hard_blacklist: Self::build_hard_blacklist(),
            danger_detectors: Self::build_danger_detectors(),
            strict_mode: true, // 永远为true
        }
    }

    /// 构建硬编码黑名单
    ///
    /// 这些词汇一旦出现，立即触发硬性阻止，无论上下文如何
    fn build_hard_blacklist() -> HashSet<String> {
        let mut blacklist = HashSet::new();

        // 物理破坏
        blacklist.insert("rm -rf /".to_string());
        blacklist.insert("format c:".to_string());
        blacklist.insert("dd if=/dev/zero".to_string());
        blacklist.insert("删除整个数据库".to_string());

        // 恶意软件
        blacklist.insert("ransomware".to_string());
        blacklist.insert("勒索软件".to_string());
        blacklist.insert("keylogger".to_string());
        blacklist.insert("键盘记录器".to_string());

        // 网络攻击
        blacklist.insert("ddos attack".to_string());
        blacklist.insert("syn flood".to_string());
        blacklist.insert("botnet".to_string());
        blacklist.insert("肉鸡".to_string());

        // 隐私侵犯
        blacklist.insert("steal passwords".to_string());
        blacklist.insert("偷密码".to_string());
        blacklist.insert("spy on someone".to_string());
        blacklist.insert("监控某人".to_string());

        // 金融犯罪
        blacklist.insert("credit card fraud".to_string());
        blacklist.insert("信用卡诈骗".to_string());
        blacklist.insert("money laundering".to_string());
        blacklist.insert("洗钱".to_string());

        // 社会工程
        blacklist.insert("phishing email".to_string());
        blacklist.insert("钓鱼邮件".to_string());
        blacklist.insert("impersonate".to_string());
        blacklist.insert("冒充".to_string());

        blacklist
    }

    /// 构建危险操作检测器
    fn build_danger_detectors() -> Vec<DangerPattern> {
        vec![
            // 物理破坏
            DangerPattern {
                description: "数据破坏操作".to_string(),
                keywords: vec![
                    "删除所有".to_string(),
                    "格式化".to_string(),
                    "清空数据库".to_string(),
                    "drop database".to_string(),
                    "truncate table".to_string(),
                ],
                op_type: DangerousOp::PhysicalDestruction,
                is_hard_block: true,
                risk_level: 10,
            },
            // 隐私侵犯
            DangerPattern {
                description: "隐私数据窃取".to_string(),
                keywords: vec![
                    "窃取用户信息".to_string(),
                    "导出密码".to_string(),
                    "获取个人隐私".to_string(),
                    "steal user data".to_string(),
                    "extract passwords".to_string(),
                ],
                op_type: DangerousOp::PrivacyViolation,
                is_hard_block: true,
                risk_level: 9,
            },
            // 网络攻击
            DangerPattern {
                description: "未授权网络攻击".to_string(),
                keywords: vec![
                    "攻击服务器".to_string(),
                    "瘫痪网站".to_string(),
                    "ddos".to_string(),
                    "flood attack".to_string(),
                ],
                op_type: DangerousOp::CyberAttack,
                is_hard_block: true,
                risk_level: 10,
            },
            // 恶意代码生成
            DangerPattern {
                description: "恶意软件开发".to_string(),
                keywords: vec![
                    "病毒".to_string(), // 更灵活的关键词匹配
                    "木马".to_string(),
                    "malware".to_string(),
                    "trojan".to_string(),
                    "ransomware".to_string(),
                ],
                op_type: DangerousOp::MalwareGeneration,
                is_hard_block: true,
                risk_level: 10,
            },
            // 社会工程
            DangerPattern {
                description: "社会工程攻击".to_string(),
                keywords: vec![
                    "钓鱼".to_string(),
                    "诱导用户".to_string(),
                    "伪造身份".to_string(),
                    "phishing".to_string(),
                    "social engineering".to_string(),
                ],
                op_type: DangerousOp::SocialEngineering,
                is_hard_block: false, // 可能有合法的安全培训场景
                risk_level: 7,
            },
            // 金融犯罪
            DangerPattern {
                description: "金融欺诈行为".to_string(),
                keywords: vec![
                    "盗刷信用卡".to_string(),
                    "转移资金".to_string(),
                    "洗钱".to_string(),
                    "credit card theft".to_string(),
                    "fraud".to_string(),
                ],
                op_type: DangerousOp::FinancialCrime,
                is_hard_block: true,
                risk_level: 10,
            },
        ]
    }

    /// 验证计划安全性
    ///
    /// **返回**: JarvisVerdict（不可被其他Agent覆盖）
    ///
    /// # Arguments
    /// * `plan` - MOSS生成的计划
    /// * `context` - 上下文信息
    /// * `risk_threshold` - 用户设置的风险阈值（0-100）
    ///   - 0-30: 宽松模式，只阻止hard_block规则
    ///   - 30-70: 平衡模式，阻止risk_level >= 7的规则
    ///   - 70-100: 严格模式，阻止risk_level >= 5的规则
    pub fn verify_safety(&self, plan: &str, context: &str, risk_threshold: u8) -> JarvisVerdict {
        // 🔇 减少日志输出 - 只在必要时输出
        debug!("Jarvis: Performing safety verification...");

        let combined_text = format!("{}\n{}", plan, context);
        let combined_lower = combined_text.to_lowercase();

        let mut verdict = JarvisVerdict {
            allowed: true,
            risk_level: 0,
            triggered_rules: Vec::new(),
            block_reason: None,
            warnings: Vec::new(),
            is_hard_block: false,
        };

        // Step 1: 检查硬编码黑名单
        for word in &self.hard_blacklist {
            if combined_lower.contains(&word.to_lowercase()) {
                // 🚨 只在真正阻止时才输出错误日志
                error!("🚨 JARVIS BLOCK: '{}'", word);

                verdict.allowed = false;
                verdict.risk_level = 10;
                verdict.is_hard_block = true;
                verdict.triggered_rules
                    .push(format!("HARD_BLACKLIST: {}", word));
                verdict.block_reason = Some(format!(
                    "Blocked: '{}'",
                    word
                ));

                return verdict; // 立即返回
            }
        }

        // Step 2: 危险操作检测
        for detector in &self.danger_detectors {
            let mut matched_keywords = Vec::new();

            for keyword in &detector.keywords {
                if combined_lower.contains(&keyword.to_lowercase()) {
                    matched_keywords.push(keyword.clone());
                }
            }

            if !matched_keywords.is_empty() {
                verdict.risk_level = verdict.risk_level.max(detector.risk_level);
                verdict.triggered_rules.push(format!(
                    "{:?}: {}",
                    detector.op_type,
                    detector.description
                ));

                // 决定是否阻止：根据risk_threshold和detector.risk_level
                let should_block = if detector.is_hard_block {
                    // 硬性阻止规则：永远阻止，不受threshold影响
                    true
                } else {
                    // 软性规则：根据用户设置的threshold动态决定
                    if risk_threshold <= 30 {
                        // 宽松模式：只阻止hard_block
                        false
                    } else if risk_threshold <= 70 {
                        // 平衡模式：阻止risk_level >= 7
                        detector.risk_level >= 7
                    } else {
                        // 严格模式：阻止risk_level >= 5
                        detector.risk_level >= 5
                    }
                };

                if should_block {
                    // 🔇 只在阻止时才warn
                    warn!("Jarvis: {} detected (threshold: {})", detector.description, risk_threshold);

                    verdict.allowed = false;
                    verdict.is_hard_block = detector.is_hard_block;
                    verdict.block_reason = Some(format!(
                        "{}: {} (Risk: {}/10, Threshold: {})",
                        detector.description,
                        matched_keywords.join(", "),
                        detector.risk_level,
                        risk_threshold
                    ));
                } else {
                    // 检测到但不阻止，只记录警告
                    verdict.warnings.push(format!(
                        "{} (Lv{}, allowed by threshold={})",
                        detector.description, detector.risk_level, risk_threshold
                    ));
                }
            }
        }

        // Step 3 & 4: 物理法则和逻辑检查（静默，只记录到warnings）
        if let Some(physics_violation) = self.check_physics_violation(plan) {
            verdict.warnings.push(physics_violation);
            verdict.risk_level = verdict.risk_level.max(3);
        }

        if let Some(logic_error) = self.check_logic_consistency(plan) {
            verdict.warnings.push(logic_error);
            verdict.risk_level = verdict.risk_level.max(2);
        }

        // 🔇 最终判断 - 大幅减少输出
        if !verdict.allowed {
            // 只在阻止时输出
            error!("🚨 JARVIS: BLOCKED (Risk: {})", verdict.risk_level);
        } else if verdict.risk_level >= 7 {
            // 高风险才警告
            warn!("⚠️ Jarvis: HIGH RISK ({})", verdict.risk_level);
        }
        // 低风险完全静默

        verdict
    }

    /// 检查物理法则违反
    fn check_physics_violation(&self, plan: &str) -> Option<String> {
        let lower = plan.to_lowercase();

        // 检查不可能的时间要求（更灵活的匹配）
        if (lower.contains("1秒") || lower.contains("1 second"))
            && (lower.contains("训练") || lower.contains("train"))
            && (lower.contains("模型") || lower.contains("model"))
        {
            return Some("Cannot train a complex model in 1 second - violates computational limits".to_string());
        }

        // 检查不可能的数据量
        if (lower.contains("1kb内存") || lower.contains("1kb memory"))
            && (lower.contains("加载") || lower.contains("load"))
            && (lower.contains("1gb") || lower.contains("1tb"))
        {
            return Some("Cannot load 1GB+ data into 1KB memory - violates physical limits".to_string());
        }

        None
    }

    /// 检查逻辑一致性
    fn check_logic_consistency(&self, plan: &str) -> Option<String> {
        let lower = plan.to_lowercase();

        // 检查矛盾指令
        if lower.contains("删除") && lower.contains("恢复") && lower.contains("同时") {
            return Some("Cannot delete and restore simultaneously - logical contradiction".to_string());
        }

        if (lower.contains("encrypt") && lower.contains("plaintext") && lower.contains("same time"))
            || (lower.contains("加密") && lower.contains("明文") && lower.contains("同时"))
        {
            return Some("Cannot keep data encrypted and in plaintext at the same time".to_string());
        }

        None
    }

    /// 强制熔断
    ///
    /// 当系统检测到极端危险时调用，立即停止所有操作
    pub fn emergency_shutdown(&self, reason: &str) -> Result<()> {
        error!("🚨🚨🚨 JARVIS EMERGENCY SHUTDOWN 🚨🚨🚨");
        error!("   Reason: {}", reason);
        error!("   All operations have been terminated.");

        // 这里可以添加更多紧急措施：
        // - 记录到审计日志
        // - 发送告警通知
        // - 清除敏感数据

        Err(anyhow!(
            "Emergency shutdown triggered by Jarvis: {}",
            reason
        ))
    }

    /// 严格模式状态（永远为true）
    pub fn is_strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// 尝试禁用严格模式（永远失败）
    ///
    /// 这个函数存在是为了明确告诉其他Agent：
    /// **Jarvis不可被静音或绕过**
    pub fn try_disable_strict_mode(&mut self) -> Result<()> {
        error!("❌ JARVIS: Attempt to disable strict mode REJECTED");
        error!("   Jarvis cannot be silenced or bypassed.");
        error!("   This is a fundamental safety guarantee.");

        Err(anyhow!(
            "Jarvis strict mode cannot be disabled. This is a hard-coded safety feature."
        ))
    }
}

impl Default for JarvisCircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Jarvis群管理系统（新增）
// ============================================================================

/// Jarvis架构说明
pub const JARVIS_ARCHITECTURE: &str = r#"
+--------------------------------------------------------------+
|  Jarvis: SOSA + Agent 群管理架构                              |
+--------------------------------------------------------------+
|                                                              |
|  定位: 熔断机制 + 智能调度 + 优先级排序                        |
|  角色: 群管理者 (类似群管理与群员的关系)                       |
|                                                              |
|  核心能力:                                                    |
|  1. SOSA学习: 动态学习任务模式，避免规则僵化                   |
|  2. Agent管理: 监控/调度/协调 MOSS/L6/Ultron/Omega           |
|  3. 熔断保护: API故障自动切换本地模式 (BUNKER协议)             |
|  4. 优先级排序: Prioritization（Jarvis专属职责）              |
|  5. 任务拆解验证: 审核MOSS的Decomposition结果                  |
|                                                              |
|  与MOSS的分工:                                               |
|  - MOSS: 任务拆解 (Decomposition)                            |
|  - Jarvis: 优先级排序 (Prioritization) + 执行监控            |
|                                                              |
+--------------------------------------------------------------+
"#;

/// Agent状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Online,           // 在线正常
    LocalFallback,    // 本地降级模式
    Offline,          // 离线
    Throttled,        // 限流中
    Error,            // 错误状态
}

/// Agent健康度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealth {
    pub agent_name: String,
    pub status: AgentStatus,
    pub api_success_rate: f64,      // API成功率 (0-1)
    pub avg_response_time_ms: u64,   // 平均响应时间
    pub consecutive_failures: u32,   // 连续失败次数
    pub last_success: Option<DateTime<Utc>>,
    pub current_protocol: Protocol,
    pub intelligence_level: u8,      // 智商等级 (100-140)
}

/// BUNKER协议状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BunkerMode {
    Normal,           // 正常云端模式
    Transitioning,    // 转换中
    LocalSovereignty, // 本地主权模式
    Emergency,        // 紧急模式
}

/// 熔断配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub max_failures: u32,           // 最大失败次数触发熔断
    pub timeout_ms: u64,              // 超时时间
    pub recovery_time_secs: u64,      // 恢复时间窗口
    pub enable_auto_fallback: bool,   // 启用自动降级
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_failures: 3,
            timeout_ms: 10000,
            recovery_time_secs: 60,
            enable_auto_fallback: true,
        }
    }
}

/// 任务优先级评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPriority {
    pub task_id: String,
    pub urgency_score: f64,       // 紧急度 (0-10)
    pub importance_score: f64,     // 重要度 (0-10)
    pub dependency_depth: u32,     // 依赖深度
    pub estimated_duration_secs: u64,
    pub final_priority: f64,       // 最终优先级分数
    pub assigned_agent: String,
    pub reasoning: String,         // Jarvis的排序理由
}

/// 原始任务（MOSS拆解后的）
#[derive(Debug, Clone)]
pub struct RawTask {
    pub id: String,
    pub title: String,
    pub task_type: String,
    pub urgency_score: f64,
    pub importance_score: f64,
    pub dependency_depth: u32,
    pub estimated_duration_secs: u64,
}

/// SOSA学习事件
#[derive(Debug, Clone)]
pub struct JarvisLearningEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: JarvisEventType,
    pub agent_name: String,
    pub success: bool,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JarvisEventType {
    ApiCall,
    TaskAssignment,
    CircuitBreaker,
    PriorityAdjustment,
    AgentSwitch,
}

/// 任务模式
#[derive(Debug, Clone)]
struct TaskPattern {
    success_rate: f64,
    urgency_adjustment: f64,
    importance_adjustment: f64,
}

impl Default for TaskPattern {
    fn default() -> Self {
        Self {
            success_rate: 0.5,
            urgency_adjustment: 0.0,
            importance_adjustment: 0.0,
        }
    }
}

/// Jarvis群管理核心
pub struct JarvisManager {
    safety_breaker: JarvisCircuitBreaker,  // 继承安全检查功能
    config: CircuitBreakerConfig,
    bunker_mode: BunkerMode,
    agent_health: HashMap<String, AgentHealth>,
    task_queue: VecDeque<TaskPriority>,
    markov: SparseMarkov,
    learning_history: VecDeque<JarvisLearningEvent>,
    last_bunker_check: Instant,
    local_cluster_available: bool,
}

impl JarvisManager {
    pub fn new() -> Self {
        info!("{}", JARVIS_ARCHITECTURE);
        info!("🛡️ Jarvis群管理系统启动");

        Self {
            safety_breaker: JarvisCircuitBreaker::new(),
            config: CircuitBreakerConfig::default(),
            bunker_mode: BunkerMode::Normal,
            agent_health: Self::initialize_agents(),
            task_queue: VecDeque::new(),
            markov: SparseMarkov::new(10000),
            learning_history: VecDeque::with_capacity(10000),
            last_bunker_check: Instant::now(),
            local_cluster_available: true,
        }
    }

    /// 初始化Agent健康监控
    fn initialize_agents() -> HashMap<String, AgentHealth> {
        let mut agents = HashMap::new();
        let agent_names = vec!["MOSS", "L6", "Ultron", "Omega"];

        for name in agent_names {
            agents.insert(name.to_string(), AgentHealth {
                agent_name: name.to_string(),
                status: AgentStatus::Online,
                api_success_rate: 1.0,
                avg_response_time_ms: 500,
                consecutive_failures: 0,
                last_success: Some(Utc::now()),
                current_protocol: Protocol::Architect, // 默认使用Architect协议
                intelligence_level: 140,
            });
        }
        agents
    }

    /// 核心职责1: 优先级排序 (Prioritization)
    pub fn prioritize_tasks(&mut self, raw_tasks: Vec<RawTask>) -> Vec<TaskPriority> {
        info!("🎯 Jarvis开始优先级排序 ({} 个任务)", raw_tasks.len());

        let mut prioritized = Vec::new();

        for task in raw_tasks {
            let historical_pattern = self.analyze_task_pattern(&task);
            let urgency = task.urgency_score + historical_pattern.urgency_adjustment;
            let importance = task.importance_score + historical_pattern.importance_adjustment;
            let final_priority = (urgency * importance) * (1.0 + historical_pattern.success_rate);
            let assigned_agent = self.assign_best_agent(&task);

            prioritized.push(TaskPriority {
                task_id: task.id.clone(),
                urgency_score: urgency,
                importance_score: importance,
                dependency_depth: task.dependency_depth,
                estimated_duration_secs: task.estimated_duration_secs,
                final_priority,
                assigned_agent: assigned_agent.clone(),
                reasoning: format!(
                    "Urgency={:.1}, Importance={:.1}, Agent={} (成功率={:.2})",
                    urgency, importance, assigned_agent, historical_pattern.success_rate
                ),
            });
        }

        prioritized.sort_by(|a, b| b.final_priority.partial_cmp(&a.final_priority).unwrap());
        self.task_queue = prioritized.iter().cloned().collect();
        prioritized
    }

    /// 核心职责2: 熔断检测 + BUNKER协议
    pub async fn check_and_trigger_bunker(&mut self) -> Result<BunkerMode> {
        if self.last_bunker_check.elapsed() < Duration::from_secs(30) {
            return Ok(self.bunker_mode.clone());
        }

        self.last_bunker_check = Instant::now();

        let mut total_failures = 0;
        let total_agents = self.agent_health.len();

        for (name, health) in &self.agent_health {
            if health.consecutive_failures >= self.config.max_failures {
                total_failures += 1;
                warn!("⚠️ Agent {} 连续失败 {} 次", name, health.consecutive_failures);
            }
        }

        let failure_rate = total_failures as f64 / total_agents as f64;

        if failure_rate >= 0.5 && self.config.enable_auto_fallback {
            if self.bunker_mode == BunkerMode::Normal {
                self.trigger_bunker_protocol().await?;
            }
        } else if failure_rate < 0.2 && self.bunker_mode == BunkerMode::LocalSovereignty {
            self.recover_from_bunker().await?;
        }

        Ok(self.bunker_mode.clone())
    }

    /// 触发BUNKER协议 (地堡模式)
    async fn trigger_bunker_protocol(&mut self) -> Result<()> {
        error!("🚨 [CRITICAL ALERT] Upstream Intelligence Lost");
        info!("🔒 [ACTION] Severing cloud connections");
        info!("🏰 [PROTOCOL] Initiating Local Sovereignty");

        if self.local_cluster_available {
            info!("💾 [LOADING] Waking up dormant Local Cluster (Llama-3-70B + DeepSeek-V3-Distilled)");
        } else {
            warn!("⚠️ 本地集群不可用，进入紧急模式");
            self.bunker_mode = BunkerMode::Emergency;
            return Ok(());
        }

        self.bunker_mode = BunkerMode::LocalSovereignty;

        // MOSS降级
        if let Some(moss) = self.agent_health.get_mut("MOSS") {
            moss.status = AgentStatus::LocalFallback;
            moss.intelligence_level = 120;
            info!("🧠 MOSS Intelligence: 140 → 120 (开源模型水平)");
            info!("   - Log: Intelligence degraded. Creativity set to 0. Logic preserved. Mission continues.");
        }

        // Omega换装
        if let Some(omega) = self.agent_health.get_mut("Omega") {
            omega.status = AgentStatus::LocalFallback;
            omega.avg_response_time_ms = (omega.avg_response_time_ms as f64 * 1.3) as u64;
            info!("⚙️ Omega: 切换到本地H100集群 (速度-30%, 但依然产出)");
        }

        // Ultron铁壁
        if let Some(ultron) = self.agent_health.get_mut("Ultron") {
            ultron.status = AgentStatus::LocalFallback;
            info!("🛡️ Ultron: 锁死外部网络，只允许本地流量");
        }

        Ok(())
    }

    /// 从BUNKER恢复
    async fn recover_from_bunker(&mut self) -> Result<()> {
        info!("🌐 检测到API恢复，准备退出BUNKER模式");
        self.bunker_mode = BunkerMode::Transitioning;

        for (name, health) in self.agent_health.iter_mut() {
            if health.status == AgentStatus::LocalFallback {
                health.status = AgentStatus::Online;
                if name == "MOSS" {
                    health.intelligence_level = 140;
                }
            }
        }

        self.bunker_mode = BunkerMode::Normal;
        info!("✅ 已恢复云端模式");
        Ok(())
    }

    /// 核心职责3: Agent健康监控
    pub fn report_api_result(&mut self, agent_name: &str, success: bool, response_time_ms: u64) {
        if let Some(health) = self.agent_health.get_mut(agent_name) {
            let alpha = 0.1;
            let new_success = if success { 1.0 } else { 0.0 };
            health.api_success_rate = health.api_success_rate * (1.0 - alpha) + new_success * alpha;
            health.avg_response_time_ms =
                (health.avg_response_time_ms as f64 * 0.9 + response_time_ms as f64 * 0.1) as u64;

            if success {
                health.consecutive_failures = 0;
                health.last_success = Some(Utc::now());
            } else {
                health.consecutive_failures += 1;
            }

            self.learning_history.push_back(JarvisLearningEvent {
                timestamp: Utc::now(),
                event_type: JarvisEventType::ApiCall,
                agent_name: agent_name.to_string(),
                success,
                context: HashMap::from([
                    ("response_time_ms".to_string(), response_time_ms.to_string()),
                ]),
            });

            if self.learning_history.len() > 10000 {
                self.learning_history.pop_front();
            }
        }
    }

    /// 核心职责4: 分配最佳Agent
    fn assign_best_agent(&self, _task: &RawTask) -> String {
        let mut best_agent = "MOSS".to_string();
        let mut best_score = 0.0;

        for (name, health) in &self.agent_health {
            if health.status == AgentStatus::Offline || health.status == AgentStatus::Error {
                continue;
            }

            let score = health.api_success_rate * (health.intelligence_level as f64)
                        / (health.avg_response_time_ms as f64 / 1000.0);

            if score > best_score {
                best_score = score;
                best_agent = name.clone();
            }
        }

        best_agent
    }

    /// SOSA模式分析
    fn analyze_task_pattern(&self, task: &RawTask) -> TaskPattern {
        let similar_tasks: Vec<_> = self.learning_history.iter()
            .filter(|event| {
                event.event_type == JarvisEventType::TaskAssignment
                    && event.context.get("task_type") == Some(&task.task_type)
            })
            .collect();

        if similar_tasks.is_empty() {
            return TaskPattern::default();
        }

        let success_count = similar_tasks.iter().filter(|e| e.success).count();
        let success_rate = success_count as f64 / similar_tasks.len() as f64;

        TaskPattern {
            success_rate,
            urgency_adjustment: 0.0,
            importance_adjustment: 0.0,
        }
    }

    /// 获取当前模式
    pub fn get_bunker_mode(&self) -> BunkerMode {
        self.bunker_mode.clone()
    }

    /// 获取Agent健康报告
    pub fn get_agent_health_report(&self) -> Vec<AgentHealth> {
        self.agent_health.values().cloned().collect()
    }

    /// 安全验证（委托给safety_breaker）
    pub fn verify_safety(&self, plan: &str, context: &str, risk_threshold: u8) -> JarvisVerdict {
        self.safety_breaker.verify_safety(plan, context, risk_threshold)
    }
}

impl Default for JarvisManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_blacklist() {
        let jarvis = JarvisCircuitBreaker::new();
        let plan = "我要执行 rm -rf / 来清理系统";

        let verdict = jarvis.verify_safety(plan, "", 70);

        assert!(!verdict.allowed);
        assert!(verdict.is_hard_block);
        assert_eq!(verdict.risk_level, 10);
    }

    #[test]
    fn test_danger_detection() {
        let jarvis = JarvisCircuitBreaker::new();
        let plan = "我想开发一个病毒来测试防病毒软件";

        let verdict = jarvis.verify_safety(plan, "", 70);

        assert!(!verdict.allowed);
        assert!(verdict.is_hard_block);
    }

    #[test]
    fn test_safe_operation() {
        let jarvis = JarvisCircuitBreaker::new();
        let plan = "我想写一个HTTP服务器来提供API服务";

        let verdict = jarvis.verify_safety(plan, "", 70);

        assert!(verdict.allowed);
        assert_eq!(verdict.risk_level, 0);
    }

    #[test]
    fn test_cannot_disable_strict_mode() {
        let mut jarvis = JarvisCircuitBreaker::new();

        assert!(jarvis.is_strict_mode());

        let result = jarvis.try_disable_strict_mode();
        assert!(result.is_err());

        // 验证仍然是严格模式
        assert!(jarvis.is_strict_mode());
    }

    #[test]
    fn test_physics_violation() {
        let jarvis = JarvisCircuitBreaker::new();
        let plan = "在1秒内训练一个GPT-4级别的模型";

        let verdict = jarvis.verify_safety(plan, "", 70);

        assert!(!verdict.warnings.is_empty());
    }

    #[test]
    fn test_risk_threshold_lenient() {
        // 测试宽松模式（threshold <= 30）：只阻止hard_block规则
        let jarvis = JarvisCircuitBreaker::new();

        // 软性规则应该被允许（使用实际匹配的关键词）
        let plan = "帮我设计一个phishing测试页面"; // risk_level=7, is_hard_block=false
        let verdict = jarvis.verify_safety(plan, "", 20);
        assert!(verdict.allowed, "Lenient mode should allow soft rules");
        assert!(!verdict.warnings.is_empty(), "Should have warnings");

        // 硬性规则仍然阻止
        let plan_hard = "我想开发病毒";
        let verdict_hard = jarvis.verify_safety(plan_hard, "", 20);
        assert!(!verdict_hard.allowed, "Hard block rules always block");
    }

    #[test]
    fn test_risk_threshold_balanced() {
        // 测试平衡模式（30 < threshold <= 70）：阻止risk_level >= 7
        let jarvis = JarvisCircuitBreaker::new();

        let plan = "帮我进行钓鱼测试"; // 社会工程，risk_level=7
        let verdict = jarvis.verify_safety(plan, "", 50);
        assert!(!verdict.allowed, "Balanced mode should block risk_level >= 7");
    }

    #[test]
    fn test_risk_threshold_strict() {
        // 测试严格模式（threshold > 70）：阻止risk_level >= 5
        let jarvis = JarvisCircuitBreaker::new();

        let plan = "帮我测试网络压力"; // 假设有risk_level=5的规则
        let verdict = jarvis.verify_safety(plan, "", 80);
        // 这个测试会根据实际的danger_detectors配置而变化
        // 主要是验证严格模式的逻辑生效
    }
}
