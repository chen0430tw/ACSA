// Workflow Engine - 工作流分配系统
// 集成MOSS拆解 + Jarvis排序

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::concurrency::{AsyncTask, ConcurrencyManager, TaskPriority as ConcurrentTaskPriority};
use super::jarvis::{JarvisManager, RawTask, TaskPriority};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub agent: String,
    pub dependencies: Vec<String>,
}

pub struct WorkflowEngine {
    jarvis: JarvisManager,
    concurrency: ConcurrencyManager,
}

impl WorkflowEngine {
    pub fn new(concurrency: ConcurrencyManager) -> Self {
        info!("📊 Workflow Engine initialized");
        Self {
            jarvis: JarvisManager::new(),
            concurrency,
        }
    }

    pub async fn execute_workflow(&mut self, workflow: Workflow) -> Result<()> {
        info!("🚀 Executing workflow: {}", workflow.name);

        // 1. MOSS拆解（模拟）
        let raw_tasks = self.decompose_workflow(&workflow);

        // 2. Jarvis排序
        let prioritized = self.jarvis.prioritize_tasks(raw_tasks);

        // 3. 提交到并发管理器
        for task in prioritized {
            let async_task = self.convert_to_async_task(task);
            self.concurrency.submit_task(async_task).await?;
        }

        info!("✅ Workflow submitted");
        Ok(())
    }

    fn decompose_workflow(&self, workflow: &Workflow) -> Vec<RawTask> {
        workflow.steps.iter().map(|step| RawTask {
            id: step.id.clone(),
            title: step.name.clone(),
            task_type: "workflow_step".to_string(),
            urgency_score: 5.0,
            importance_score: 7.0,
            dependency_depth: step.dependencies.len() as u32,
            estimated_duration_secs: 300,
        }).collect()
    }

    fn convert_to_async_task(&self, task: TaskPriority) -> AsyncTask {
        AsyncTask {
            id: task.task_id,
            name: task.reasoning,
            priority: ConcurrentTaskPriority::Normal,
            created_at: chrono::Utc::now(),
            agent_name: Some(task.assigned_agent),
            metadata: HashMap::new(),
        }
    }
}
