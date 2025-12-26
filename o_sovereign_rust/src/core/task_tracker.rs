// Task Tracker - 类似Claude Code的TODO系统
// 帮助用户了解Agents当前正在做什么

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已跳过
    Skipped,
}

impl TaskStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "⏳",
            TaskStatus::InProgress => "🔄",
            TaskStatus::Completed => "✅",
            TaskStatus::Failed => "❌",
            TaskStatus::Skipped => "⏭️",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "Pending",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
            TaskStatus::Skipped => "Skipped",
        }
    }
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// 单个任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub agent: Option<String>, // 负责的Agent (MOSS/L6/Ultron/Omega)
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: u8, // 0-100
    pub subtasks: Vec<Task>,
    pub error_message: Option<String>,
}

impl Task {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            description: None,
            status: TaskStatus::Pending,
            priority: TaskPriority::Normal,
            agent: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress: 0,
            subtasks: Vec::new(),
            error_message: None,
        }
    }

    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.started_at = Some(Utc::now());
    }

    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.progress = 100;
    }

    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error.into());
    }

    pub fn skip(&mut self) {
        self.status = TaskStatus::Skipped;
        self.completed_at = Some(Utc::now());
    }

    pub fn update_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
    }

    /// 获取任务耗时（秒）
    pub fn elapsed_seconds(&self) -> Option<i64> {
        if let Some(started) = self.started_at {
            let end = self.completed_at.unwrap_or_else(Utc::now);
            Some((end - started).num_seconds())
        } else {
            None
        }
    }

    /// 格式化为单行显示
    pub fn format_oneline(&self) -> String {
        let agent_str = self.agent.as_ref().map(|a| format!("[{}]", a)).unwrap_or_default();
        format!(
            "{} {} {} {}",
            self.status.icon(),
            agent_str,
            self.title,
            if self.status == TaskStatus::InProgress {
                format!("({}%)", self.progress)
            } else {
                String::new()
            }
        )
    }
}

/// 任务追踪器
pub struct TaskTracker {
    tasks: HashMap<String, Task>,
    task_order: Vec<String>, // 维护任务顺序
}

impl Default for TaskTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskTracker {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            task_order: Vec::new(),
        }
    }

    /// 添加任务
    pub fn add_task(&mut self, task: Task) {
        let id = task.id.clone();
        self.tasks.insert(id.clone(), task);
        self.task_order.push(id);
    }

    /// 获取任务
    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.get(id)
    }

    /// 获取可变任务
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.get_mut(id)
    }

    /// 开始任务
    pub fn start_task(&mut self, id: &str) {
        if let Some(task) = self.tasks.get_mut(id) {
            task.start();
        }
    }

    /// 完成任务
    pub fn complete_task(&mut self, id: &str) {
        if let Some(task) = self.tasks.get_mut(id) {
            task.complete();
        }
    }

    /// 任务失败
    pub fn fail_task(&mut self, id: &str, error: impl Into<String>) {
        if let Some(task) = self.tasks.get_mut(id) {
            task.fail(error);
        }
    }

    /// 更新进度
    pub fn update_progress(&mut self, id: &str, progress: u8) {
        if let Some(task) = self.tasks.get_mut(id) {
            task.update_progress(progress);
        }
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Vec<&Task> {
        self.task_order.iter().filter_map(|id| self.tasks.get(id)).collect()
    }

    /// 获取未完成任务
    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.get_all_tasks()
            .into_iter()
            .filter(|t| matches!(t.status, TaskStatus::Pending | TaskStatus::InProgress))
            .collect()
    }

    /// 清除已完成任务
    pub fn clear_completed(&mut self) {
        let completed_ids: Vec<_> = self
            .tasks
            .iter()
            .filter(|(_, t)| t.status == TaskStatus::Completed)
            .map(|(id, _)| id.clone())
            .collect();

        for id in completed_ids {
            self.tasks.remove(&id);
            self.task_order.retain(|x| x != &id);
        }
    }

    /// 清除所有任务
    pub fn clear_all(&mut self) {
        self.tasks.clear();
        self.task_order.clear();
    }

    /// 生成Mermaid流程图
    pub fn generate_mermaid(&self) -> String {
        let mut mermaid = String::from("graph TD\n");

        for (idx, id) in self.task_order.iter().enumerate() {
            if let Some(task) = self.tasks.get(id) {
                let node_id = format!("T{}", idx);
                let status_style = match task.status {
                    TaskStatus::Completed => "fill:#90EE90",
                    TaskStatus::InProgress => "fill:#FFD700",
                    TaskStatus::Failed => "fill:#FF6B6B",
                    TaskStatus::Pending => "fill:#E0E0E0",
                    TaskStatus::Skipped => "fill:#D3D3D3",
                };

                mermaid.push_str(&format!(
                    "    {}[\"{}{}\"]\n",
                    node_id,
                    task.status.icon(),
                    task.title
                ));
                mermaid.push_str(&format!("    style {} {}\n", node_id, status_style));

                // 连接到下一个任务
                if idx < self.task_order.len() - 1 {
                    mermaid.push_str(&format!("    {} --> T{}\n", node_id, idx + 1));
                }
            }
        }

        mermaid
    }

    /// 打印任务列表
    pub fn print_tasks(&self) {
        println!("\n┌─────────────────────────────────────────┐");
        println!("│          ACSA Task Tracker              │");
        println!("├─────────────────────────────────────────┤");

        if self.tasks.is_empty() {
            println!("│  No tasks                               │");
        } else {
            for task in self.get_all_tasks() {
                println!("│  {}  │", task.format_oneline());
            }
        }

        println!("└─────────────────────────────────────────┘");

        // 统计信息
        let total = self.tasks.len();
        let completed = self.tasks.values().filter(|t| t.status == TaskStatus::Completed).count();
        let failed = self.tasks.values().filter(|t| t.status == TaskStatus::Failed).count();
        let in_progress = self.tasks.values().filter(|t| t.status == TaskStatus::InProgress).count();

        println!(
            "Total: {}, Completed: {}, Failed: {}, In Progress: {}",
            total, completed, failed, in_progress
        );
    }

    /// 打印Mermaid图（如果有任务）
    pub fn print_mermaid_if_needed(&self) {
        if !self.tasks.is_empty() && self.tasks.len() >= 3 {
            println!("\n📊 Task Flow Diagram:\n");
            println!("```mermaid");
            println!("{}", self.generate_mermaid());
            println!("```\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("task1".to_string(), "Test task".to_string())
            .with_agent("MOSS")
            .with_priority(TaskPriority::High);

        assert_eq!(task.id, "task1");
        assert_eq!(task.agent, Some("MOSS".to_string()));
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = Task::new("task1".to_string(), "Test".to_string());

        assert_eq!(task.status, TaskStatus::Pending);

        task.start();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.started_at.is_some());

        task.update_progress(50);
        assert_eq!(task.progress, 50);

        task.complete();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.progress, 100);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_tracker() {
        let mut tracker = TaskTracker::new();

        let task1 = Task::new("t1".to_string(), "Task 1".to_string());
        let task2 = Task::new("t2".to_string(), "Task 2".to_string());

        tracker.add_task(task1);
        tracker.add_task(task2);

        assert_eq!(tracker.get_all_tasks().len(), 2);

        tracker.start_task("t1");
        tracker.complete_task("t1");

        let completed = tracker.get_all_tasks().into_iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        assert_eq!(completed, 1);

        tracker.clear_completed();
        assert_eq!(tracker.get_all_tasks().len(), 1);
    }

    #[test]
    fn test_mermaid_generation() {
        let mut tracker = TaskTracker::new();

        tracker.add_task(Task::new("t1".to_string(), "Start".to_string()));
        tracker.add_task(Task::new("t2".to_string(), "Process".to_string()));
        tracker.add_task(Task::new("t3".to_string(), "End".to_string()));

        tracker.start_task("t1");
        tracker.complete_task("t1");
        tracker.start_task("t2");

        let mermaid = tracker.generate_mermaid();

        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("Start"));
        assert!(mermaid.contains("Process"));
        assert!(mermaid.contains("End"));
    }
}
