// O-Sovereign Core Types
// ACSA (对抗约束型盲从代理) 核心数据类型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent 角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    /// MOSS - 战略规划 (GPT-5.2)
    MOSS,
    /// L6 - 真理校验 (Gemini 3 Deep Think)
    L6,
    /// Ultron - 红队审计 (Claude Opus 4.5)
    Ultron,
    /// Omega - 盲从执行 (Gemini Flash)
    Omega,
}

impl AgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MOSS => "MOSS",
            Self::L6 => "L6",
            Self::Ultron => "Ultron",
            Self::Omega => "Omega",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::MOSS => "战略规划 AI - 负责任务拆解和长期规划",
            Self::L6 => "真理校验 AI - 负责物理逻辑验证",
            Self::Ultron => "红队审计 AI - 负责风险评估和合规检查",
            Self::Omega => "盲从执行 AI - 负责按计划执行任务",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::MOSS => "🧠",
            Self::L6 => "🔬",
            Self::Ultron => "🛡️",
            Self::Omega => "⚡",
        }
    }
}

/// Agent 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub role: AgentRole,
    pub text: String,
    pub tokens: u32,
    pub cost: f64,
    pub latency_ms: u64,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// 审计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub is_safe: bool,
    pub risk_score: u8, // 0-100
    pub legal_risks: Vec<String>,
    pub physical_risks: Vec<String>,
    pub ethical_risks: Vec<String>,
    pub mitigation: String,
    pub raw_response: String,
}

/// ACSA 执行日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACSAExecutionLog {
    pub user_input: String,
    pub moss_plan: Option<AgentResponse>,
    pub l6_verification: Option<AgentResponse>,
    pub ultron_audit: Option<AgentResponse>,
    pub omega_execution: Option<AgentResponse>,
    pub audit_result: Option<AuditResult>,
    pub final_output: Option<String>,
    pub total_time_ms: u64,
    pub total_cost: f64,
    pub iterations: u32,
    pub success: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ACSAExecutionLog {
    pub fn new(user_input: String) -> Self {
        Self {
            user_input,
            moss_plan: None,
            l6_verification: None,
            ultron_audit: None,
            omega_execution: None,
            audit_result: None,
            final_output: None,
            total_time_ms: 0,
            total_cost: 0.0,
            iterations: 0,
            success: false,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn complete(&mut self, success: bool) {
        self.success = success;
        self.completed_at = Some(Utc::now());
        self.total_time_ms = (self.completed_at.unwrap() - self.started_at)
            .num_milliseconds() as u64;
    }
}

/// Agent 统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub total_latency_ms: u64,
}

impl AgentStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&mut self, tokens: u32, cost: f64, latency_ms: u64) {
        self.total_calls += 1;
        self.successful_calls += 1;
        self.total_tokens += tokens as u64;
        self.total_cost += cost;
        self.total_latency_ms += latency_ms;
    }

    pub fn record_failure(&mut self, latency_ms: u64) {
        self.total_calls += 1;
        self.failed_calls += 1;
        self.total_latency_ms += latency_ms;
    }

    pub fn average_latency_ms(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.total_calls as f64
        }
    }
}

/// ACSA 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACSAConfig {
    pub max_iterations: u32,
    pub risk_threshold: u8,
    pub enable_l6: bool,
    pub enable_streaming: bool,
}

impl Default for ACSAConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            risk_threshold: 70,
            enable_l6: true,
            enable_streaming: false,
        }
    }
}

/// UI 状态
#[derive(Debug, Clone, PartialEq)]
pub enum UIState {
    Idle,
    Processing,
    Success,
    Error(String),
}

/// Agent 状态
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Completed,
    Failed,
}
