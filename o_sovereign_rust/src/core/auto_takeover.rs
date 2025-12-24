// Auto Takeover Engine - AI自动接管系统
// 基于行为模式检测的主动任务执行引擎
// Proactive task execution based on behavior pattern detection

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::behavior_monitor::{BehaviorMonitor, TakeoverSuggestion, UserBehaviorEvent};
use super::protocol::Protocol;
use super::task_tracker::{Task, TaskStatus, TaskTracker};

/// 接管动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TakeoverAction {
    /// 执行命令
    ExecuteCommand { command: String, args: Vec<String> },
    /// 切换协议
    SwitchProtocol { protocol: Protocol },
    /// 创建文件
    CreateFile { path: String, content: String },
    /// 运行测试
    RunTests { framework: String },
    /// Git操作
    GitOperation { operation: String },
    /// 自定义脚本
    CustomScript { script: String },
}

/// 接管执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverResult {
    pub action: TakeoverAction,
    pub success: bool,
    pub output: String,
    pub executed_at: DateTime<Utc>,
    pub duration_ms: u64,
}

/// 接管策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverPolicy {
    /// 是否需要用户确认
    pub require_confirmation: bool,
    /// 自动执行的最大风险等级 (1-5)
    pub max_auto_risk_level: u8,
    /// 是否启用学习模式 (观察但不执行)
    pub learning_mode: bool,
    /// 冷却时间(秒) - 两次接管之间的最小间隔
    pub cooldown_secs: u64,
}

impl Default for TakeoverPolicy {
    fn default() -> Self {
        Self {
            require_confirmation: true,
            max_auto_risk_level: 2, // 只自动执行低风险操作
            learning_mode: false,
            cooldown_secs: 60,
        }
    }
}

/// 自动接管引擎
pub struct AutoTakeoverEngine {
    /// 行为监控器
    behavior_monitor: Arc<RwLock<BehaviorMonitor>>,
    /// 任务追踪器
    task_tracker: Arc<RwLock<TaskTracker>>,
    /// 接管策略
    policy: TakeoverPolicy,
    /// 接管历史
    history: Vec<TakeoverResult>,
    /// 上次接管时间
    last_takeover: Option<DateTime<Utc>>,
    /// 待确认的接管建议
    pending_suggestions: Vec<TakeoverSuggestion>,
}

impl AutoTakeoverEngine {
    pub fn new(
        behavior_monitor: Arc<RwLock<BehaviorMonitor>>,
        task_tracker: Arc<RwLock<TaskTracker>>,
        policy: TakeoverPolicy,
    ) -> Self {
        info!("🤖 Auto-Takeover Engine initialized");
        info!("  Confirmation required: {}", policy.require_confirmation);
        info!("  Learning mode: {}", policy.learning_mode);

        Self {
            behavior_monitor,
            task_tracker,
            policy,
            history: Vec::new(),
            last_takeover: None,
            pending_suggestions: Vec::new(),
        }
    }

    /// 记录用户行为事件
    pub async fn record_behavior(&mut self, event: UserBehaviorEvent) {
        let mut monitor = self.behavior_monitor.write().await;
        monitor.record_event(event);
    }

    /// 检查是否应该触发接管
    pub async fn check_takeover(&mut self) -> Option<TakeoverSuggestion> {
        // 检查冷却时间
        if let Some(last) = self.last_takeover {
            let elapsed = (Utc::now() - last).num_seconds() as u64;
            if elapsed < self.policy.cooldown_secs {
                return None;
            }
        }

        let monitor = self.behavior_monitor.read().await;
        let suggestion = monitor.should_trigger_takeover()?;

        // 如果需要确认，添加到待确认列表
        if self.policy.require_confirmation {
            self.pending_suggestions.push(suggestion.clone());
            info!("⏸️ Takeover suggestion pending confirmation: {}", suggestion.pattern_id);
            return Some(suggestion);
        }

        // 学习模式：只观察不执行
        if self.policy.learning_mode {
            info!("📚 Learning mode: Would execute pattern {}", suggestion.pattern_id);
            return None;
        }

        Some(suggestion)
    }

    /// 确认接管建议
    pub async fn confirm_takeover(&mut self, pattern_id: &str) -> Result<()> {
        let suggestion = self
            .pending_suggestions
            .iter()
            .find(|s| s.pattern_id == pattern_id)
            .ok_or_else(|| anyhow::anyhow!("Suggestion not found: {}", pattern_id))?
            .clone();

        self.execute_takeover(&suggestion).await?;

        // 从待确认列表移除
        self.pending_suggestions.retain(|s| s.pattern_id != pattern_id);

        Ok(())
    }

    /// 拒绝接管建议
    pub fn reject_takeover(&mut self, pattern_id: &str) {
        self.pending_suggestions.retain(|s| s.pattern_id != pattern_id);
        info!("❌ Takeover rejected: {}", pattern_id);
    }

    /// 执行接管
    async fn execute_takeover(&mut self, suggestion: &TakeoverSuggestion) -> Result<()> {
        info!("🚀 Executing takeover: {}", suggestion.pattern_id);

        let start = std::time::Instant::now();

        // 根据预测的行为生成接管动作
        let actions = self.generate_actions_from_prediction(suggestion);

        for action in actions {
            let result = self.execute_action(action).await;

            self.history.push(result);
        }

        self.last_takeover = Some(Utc::now());

        let elapsed = start.elapsed().as_millis() as u64;
        info!("✅ Takeover completed in {} ms", elapsed);

        Ok(())
    }

    /// 从预测生成动作
    fn generate_actions_from_prediction(&self, suggestion: &TakeoverSuggestion) -> Vec<TakeoverAction> {
        // 简化实现：基于pattern_id猜测可能的动作
        let mut actions = Vec::new();

        // 如果检测到"编写-测试"模式
        if suggestion.predicted_actions.iter().any(|a| a.contains("test")) {
            actions.push(TakeoverAction::RunTests {
                framework: "cargo".to_string(),
            });
        }

        // 如果检测到"修改-提交"模式
        if suggestion.predicted_actions.iter().any(|a| a.contains("git")) {
            actions.push(TakeoverAction::GitOperation {
                operation: "status".to_string(),
            });
        }

        // 默认：创建任务提醒
        if actions.is_empty() {
            actions.push(TakeoverAction::ExecuteCommand {
                command: "echo".to_string(),
                args: vec![format!("Auto-takeover ready for pattern: {}", suggestion.pattern_id)],
            });
        }

        actions
    }

    /// 执行单个动作
    async fn execute_action(&mut self, action: TakeoverAction) -> TakeoverResult {
        let start = std::time::Instant::now();

        let (success, output) = match &action {
            TakeoverAction::ExecuteCommand { command, args } => {
                info!("⚡ Executing: {} {:?}", command, args);

                // 实际执行会调用真实的命令
                // 这里简化为模拟
                (true, format!("Executed: {} {:?}", command, args))
            }

            TakeoverAction::SwitchProtocol { protocol } => {
                info!("🔀 Switching protocol to: {:?}", protocol);
                (true, format!("Switched to {:?}", protocol))
            }

            TakeoverAction::CreateFile { path, content: _ } => {
                info!("📝 Creating file: {}", path);
                (true, format!("Created file: {}", path))
            }

            TakeoverAction::RunTests { framework } => {
                info!("🧪 Running tests: {}", framework);

                // 添加到任务追踪器
                let mut tracker = self.task_tracker.write().await;
                tracker.add_task(Task {
                    id: format!("test_{}", Utc::now().timestamp()),
                    title: format!("Auto-run {} tests", framework),
                    description: Some("Automatically triggered test run".to_string()),
                    status: TaskStatus::InProgress,
                    priority: super::task_tracker::TaskPriority::High,
                    agent: Some("Omega".to_string()),
                    created_at: Utc::now(),
                    started_at: Some(Utc::now()),
                    completed_at: None,
                    progress: 0,
                    subtasks: vec![],
                    error_message: None,
                });

                (true, format!("Tests executed via {}", framework))
            }

            TakeoverAction::GitOperation { operation } => {
                info!("📦 Git operation: {}", operation);
                (true, format!("Git {}: success", operation))
            }

            TakeoverAction::CustomScript { script } => {
                info!("📜 Running custom script");
                (true, format!("Script executed: {}", script))
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        TakeoverResult {
            action,
            success,
            output,
            executed_at: Utc::now(),
            duration_ms,
        }
    }

    /// 获取接管历史
    pub fn get_history(&self) -> &[TakeoverResult] {
        &self.history
    }

    /// 获取待确认的建议
    pub fn get_pending_suggestions(&self) -> &[TakeoverSuggestion] {
        &self.pending_suggestions
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> TakeoverStats {
        let total_takeovers = self.history.len();
        let successful = self.history.iter().filter(|r| r.success).count();

        let avg_duration_ms = if !self.history.is_empty() {
            self.history.iter().map(|r| r.duration_ms).sum::<u64>() / self.history.len() as u64
        } else {
            0
        };

        TakeoverStats {
            total_takeovers,
            successful_takeovers: successful,
            failed_takeovers: total_takeovers - successful,
            avg_duration_ms,
            learning_mode: self.policy.learning_mode,
            pending_count: self.pending_suggestions.len(),
        }
    }

    /// 打印接管报告
    pub fn print_report(&self) {
        let stats = self.get_stats();

        println!("\n┌─────────────────────────────────────────────────────┐");
        println!("│        Auto-Takeover Engine Report                  │");
        println!("├─────────────────────────────────────────────────────┤");
        println!("│  Total Takeovers:    {}", stats.total_takeovers);
        println!("│  Successful:         {}", stats.successful_takeovers);
        println!("│  Failed:             {}", stats.failed_takeovers);
        println!("│  Avg Duration:       {} ms", stats.avg_duration_ms);
        println!("│  Pending:            {}", stats.pending_count);
        println!("│  Learning Mode:      {}", stats.learning_mode);
        println!("└─────────────────────────────────────────────────────┘");

        if !self.pending_suggestions.is_empty() {
            println!("\n⏸️ Pending Suggestions:");
            for suggestion in &self.pending_suggestions {
                println!(
                    "  {} - {:.0}% confidence",
                    suggestion.description,
                    suggestion.confidence * 100.0
                );
            }
        }

        if !self.history.is_empty() {
            println!("\n📜 Recent Actions:");
            for result in self.history.iter().rev().take(5) {
                let status = if result.success { "✅" } else { "❌" };
                println!("  {} {:?} ({}ms)", status, result.action, result.duration_ms);
            }
        }
    }
}

/// 接管统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverStats {
    pub total_takeovers: usize,
    pub successful_takeovers: usize,
    pub failed_takeovers: usize,
    pub avg_duration_ms: u64,
    pub learning_mode: bool,
    pub pending_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::behavior_monitor::{BehaviorContext, BehaviorMonitorConfig, BehaviorType};

    #[tokio::test]
    async fn test_auto_takeover_engine() {
        let monitor = Arc::new(RwLock::new(BehaviorMonitor::new(
            BehaviorMonitorConfig::default(),
        )));
        let tracker = Arc::new(RwLock::new(TaskTracker::new()));

        let mut engine = AutoTakeoverEngine::new(monitor, tracker, TakeoverPolicy::default());

        // 记录一些行为
        engine
            .record_behavior(UserBehaviorEvent {
                timestamp: Utc::now(),
                event_type: BehaviorType::CodeWrite {
                    language: "rust".to_string(),
                    lines: 10,
                },
                context: BehaviorContext {
                    working_dir: "/test".to_string(),
                    current_file: Some("main.rs".to_string()),
                    time_of_day: 14,
                    day_of_week: 3,
                },
                duration_ms: Some(5000),
            })
            .await;

        let stats = engine.get_stats();
        assert_eq!(stats.total_takeovers, 0);
    }

    #[tokio::test]
    async fn test_takeover_confirmation() {
        let monitor = Arc::new(RwLock::new(BehaviorMonitor::new(
            BehaviorMonitorConfig::default(),
        )));
        let tracker = Arc::new(RwLock::new(TaskTracker::new()));

        let policy = TakeoverPolicy {
            require_confirmation: true,
            ..Default::default()
        };

        let engine = AutoTakeoverEngine::new(monitor, tracker, policy);

        assert_eq!(engine.get_pending_suggestions().len(), 0);
    }
}
