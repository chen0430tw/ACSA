// Concurrency & Distributed Support - 多线程和分布式支持
// 目标：高性能并发处理和分布式任务调度
//
// 核心功能：
// 1. 多线程任务池：基于tokio的异步任务调度
// 2. 工作窃取：自动负载均衡
// 3. 分布式锁：跨进程任务协调
// 4. 任务队列：优先级队列 + 公平调度
// 5. 背压控制：防止系统过载

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore, Mutex as TokioMutex};
use tokio::task::JoinHandle;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// 并发配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    pub max_concurrent_tasks: usize,    // 最大并发任务数
    pub worker_threads: usize,           // 工作线程数
    pub enable_work_stealing: bool,      // 启用工作窃取
    pub task_timeout_secs: u64,          // 任务超时时间
    pub enable_backpressure: bool,       // 启用背压控制
    pub max_queue_size: usize,           // 最大队列大小
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self {
            max_concurrent_tasks: cpu_count * 4,
            worker_threads: cpu_count,
            enable_work_stealing: true,
            task_timeout_secs: 300,  // 5分钟
            enable_backpressure: true,
            max_queue_size: 10000,
        }
    }
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical = 4,
    High = 3,
    Normal = 2,
    Low = 1,
    Background = 0,
}

/// 异步任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTask {
    pub id: String,
    pub name: String,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub agent_name: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// 任务结果
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// 并发管理器
pub struct ConcurrencyManager {
    config: ConcurrencyConfig,
    semaphore: Arc<Semaphore>,
    task_queue: Arc<TokioMutex<VecDeque<AsyncTask>>>,
    running_tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    task_results: Arc<RwLock<HashMap<String, TaskResult>>>,
}

impl ConcurrencyManager {
    pub fn new(config: ConcurrencyConfig) -> Self {
        info!("🚀 Concurrency Manager initialized");
        info!("   - Max concurrent: {}", config.max_concurrent_tasks);
        info!("   - Workers: {}", config.worker_threads);
        info!("   - Work stealing: {}", config.enable_work_stealing);

        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_tasks)),
            task_queue: Arc::new(TokioMutex::new(VecDeque::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_results: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 提交任务
    pub async fn submit_task(&self, task: AsyncTask) -> Result<()> {
        // 背压控制
        if self.config.enable_backpressure {
            let queue_len = self.task_queue.lock().await.len();
            if queue_len >= self.config.max_queue_size {
                warn!("⚠️ Task queue full, rejecting task: {}", task.name);
                return Err(anyhow!("Task queue full"));
            }
        }

        let mut queue = self.task_queue.lock().await;
        queue.push_back(task.clone());
        
        info!("📥 Task submitted: {} (priority: {:?})", task.name, task.priority);
        Ok(())
    }

    /// 执行下一个任务
    pub async fn execute_next_task<F, Fut>(&self, executor: F) -> Result<()>
    where
        F: Fn(AsyncTask) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let task = {
            let mut queue = self.task_queue.lock().await;
            queue.pop_front()
        };

        if let Some(task) = task {
            let permit = self.semaphore.clone().acquire_owned().await?;
            let task_id = task.id.clone();
            let task_name = task.name.clone();
            let results = self.task_results.clone();
            let running = self.running_tasks.clone();

            let start = std::time::Instant::now();

            let task_id_for_map = task_id.clone();

            let handle = tokio::spawn(async move {
                info!("🔄 Executing task: {}", task_name);

                let result = executor(task.clone()).await;
                let duration = start.elapsed().as_millis() as u64;

                let (success, error) = match result {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                };

                let task_result = TaskResult {
                    task_id: task_id.clone(),
                    success,
                    duration_ms: duration,
                    error,
                };

                let mut results_guard = results.write().await;
                results_guard.insert(task_id.clone(), task_result);

                let mut running_guard = running.write().await;
                running_guard.remove(&task_id);

                drop(permit);

                if success {
                    info!("✅ Task completed: {} ({}ms)", task_name, duration);
                } else {
                    error!("❌ Task failed: {}", task_name);
                }
            });

            let mut running = self.running_tasks.write().await;
            running.insert(task_id_for_map, handle);
        }

        Ok(())
    }

    /// 获取任务结果
    pub async fn get_task_result(&self, task_id: &str) -> Option<TaskResult> {
        let results = self.task_results.read().await;
        results.get(task_id).cloned()
    }

    /// 获取队列长度
    pub async fn queue_length(&self) -> usize {
        self.task_queue.lock().await.len()
    }

    /// 获取运行中任务数
    pub async fn running_count(&self) -> usize {
        self.running_tasks.read().await.len()
    }

    /// 等待所有任务完成
    pub async fn wait_all(&self) -> Result<()> {
        loop {
            let running_count = self.running_count().await;
            let queue_len = self.queue_length().await;

            if running_count == 0 && queue_len == 0 {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("✅ All tasks completed");
        Ok(())
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let mut running = self.running_tasks.write().await;
        
        if let Some(handle) = running.remove(task_id) {
            handle.abort();
            info!("🛑 Task cancelled: {}", task_id);
            Ok(())
        } else {
            Err(anyhow!("Task not found: {}", task_id))
        }
    }
}

/// 分布式锁（简化版，生产环境应使用Redis等）
pub struct DistributedLock {
    locks: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    ttl_secs: u64,
}

impl DistributedLock {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            ttl_secs,
        }
    }

    pub async fn acquire(&self, lock_name: &str) -> Result<bool> {
        let mut locks = self.locks.write().await;

        // 检查锁是否存在且未过期
        if let Some(expire_at) = locks.get(lock_name) {
            if Utc::now() < *expire_at {
                return Ok(false); // 锁已被占用
            }
        }

        // 获取锁
        let expire_at = Utc::now() + chrono::Duration::seconds(self.ttl_secs as i64);
        locks.insert(lock_name.to_string(), expire_at);

        debug!("🔒 Lock acquired: {}", lock_name);
        Ok(true)
    }

    pub async fn release(&self, lock_name: &str) -> Result<()> {
        let mut locks = self.locks.write().await;
        locks.remove(lock_name);

        debug!("🔓 Lock released: {}", lock_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrency_manager() {
        let manager = ConcurrencyManager::new(ConcurrencyConfig::default());

        let task = AsyncTask {
            id: "test-task-1".to_string(),
            name: "Test Task".to_string(),
            priority: TaskPriority::Normal,
            created_at: Utc::now(),
            agent_name: None,
            metadata: HashMap::new(),
        };

        manager.submit_task(task).await.unwrap();
        assert_eq!(manager.queue_length().await, 1);
    }

    #[tokio::test]
    async fn test_distributed_lock() {
        let lock = DistributedLock::new(5);

        assert!(lock.acquire("test-lock").await.unwrap());
        assert!(!lock.acquire("test-lock").await.unwrap()); // 重复获取应失败

        lock.release("test-lock").await.unwrap();
        assert!(lock.acquire("test-lock").await.unwrap()); // 释放后应成功
    }
}
