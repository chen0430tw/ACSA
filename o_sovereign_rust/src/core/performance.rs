// Performance Optimization Module
// 性能优化模块 - 启动加速和事件响应优化
//
// 核心优化：
// 1. 懒加载（LazyLock）- 延迟初始化重量级对象
// 2. 并行初始化 - 利用多核并行启动
// 3. 缓存预热 - 提前加载常用资源
// 4. 事件去抖动 - 防止重复触发
// 5. 异步优先级调度 - 关键任务优先

use std::sync::{LazyLock, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, debug};

/// 启动性能统计
#[derive(Debug, Clone)]
pub struct StartupMetrics {
    /// 总启动时间
    pub total_duration: Duration,
    /// 各阶段耗时
    pub phase_durations: HashMap<String, Duration>,
    /// 并行任务数
    pub parallel_tasks: usize,
}

/// 性能优化器
pub struct PerformanceOptimizer {
    /// 启动时间跟踪
    startup_time: Instant,
    /// 阶段耗时记录
    phase_times: Arc<RwLock<HashMap<String, Duration>>>,
    /// 事件去抖动限制器
    debounce_limiter: Arc<Semaphore>,
}

impl PerformanceOptimizer {
    /// 创建性能优化器
    pub fn new() -> Self {
        info!("🚀 Performance Optimizer initialized");
        Self {
            startup_time: Instant::now(),
            phase_times: Arc::new(RwLock::new(HashMap::new())),
            debounce_limiter: Arc::new(Semaphore::new(10)), // 最多10个并发事件
        }
    }

    /// 记录阶段开始
    pub async fn start_phase(&self, phase_name: &str) -> PhaseTracker {
        debug!("⏱️  Starting phase: {}", phase_name);
        PhaseTracker {
            name: phase_name.to_string(),
            start: Instant::now(),
            phase_times: self.phase_times.clone(),
        }
    }

    /// 获取启动指标
    pub async fn get_startup_metrics(&self) -> StartupMetrics {
        let total_duration = self.startup_time.elapsed();
        let phase_durations = self.phase_times.read().await.clone();
        let parallel_tasks = phase_durations.len();

        StartupMetrics {
            total_duration,
            phase_durations,
            parallel_tasks,
        }
    }

    /// 事件去抖动（防止重复触发）
    pub async fn debounce_event<F, Fut, T>(
        &self,
        event_handler: F,
    ) -> Option<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        // 尝试获取许可（非阻塞）
        if let Ok(permit) = self.debounce_limiter.clone().try_acquire_owned() {
            let result = event_handler().await;
            drop(permit); // 释放许可
            Some(result)
        } else {
            warn!("⚠️  Event dropped due to debounce limit");
            None
        }
    }

    /// 并行初始化（加速启动）
    pub async fn parallel_init<F, Fut>(tasks: Vec<(&str, F)>) -> Vec<anyhow::Result<()>>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
    {
        info!("🔄 Starting {} parallel initialization tasks", tasks.len());

        let handles: Vec<_> = tasks
            .into_iter()
            .map(|(name, task)| {
                let name = name.to_string();
                tokio::spawn(async move {
                    let start = Instant::now();
                    info!("  ⏳ Initializing: {}", name);
                    let result = task().await;
                    let elapsed = start.elapsed();
                    match &result {
                        Ok(_) => info!("  ✅ Initialized {} in {:?}", name, elapsed),
                        Err(e) => warn!("  ❌ Failed to initialize {}: {}", name, e),
                    }
                    result
                })
            })
            .collect();

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(anyhow::anyhow!("Task panicked: {}", e))),
            }
        }

        results
    }
}

impl Default for PerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 阶段跟踪器（自动记录耗时）
pub struct PhaseTracker {
    name: String,
    start: Instant,
    phase_times: Arc<RwLock<HashMap<String, Duration>>>,
}

impl Drop for PhaseTracker {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        debug!("  ✓ Phase '{}' completed in {:?}", self.name, elapsed);

        // 异步记录（非阻塞）
        let name = self.name.clone();
        let phase_times = self.phase_times.clone();
        tokio::spawn(async move {
            phase_times.write().await.insert(name, elapsed);
        });
    }
}

/// 懒加载示例（使用 std::sync::LazyLock）
///
/// 用法示例：
/// ```
/// use std::sync::LazyLock;
///
/// static HEAVY_RESOURCE: LazyLock<HeavyResource> = LazyLock::new(|| {
///     HeavyResource::initialize()
/// });
/// ```

/// 缓存预热器
pub struct CacheWarmer {
    /// 预热任务列表
    tasks: Vec<Box<dyn FnOnce() -> () + Send>>,
}

impl CacheWarmer {
    /// 创建缓存预热器
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// 添加预热任务
    pub fn add_task<F>(&mut self, task: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
        self.tasks.push(Box::new(task));
    }

    /// 执行预热（后台异步）
    pub fn warm_up(self) {
        info!("🔥 Starting cache warm-up with {} tasks", self.tasks.len());

        tokio::spawn(async move {
            for (i, task) in self.tasks.into_iter().enumerate() {
                task();
                debug!("  ✓ Warm-up task {} completed", i + 1);
            }
            info!("✅ Cache warm-up completed");
        });
    }
}

impl Default for CacheWarmer {
    fn default() -> Self {
        Self::new()
    }
}

/// 事件批处理器（优化按钮响应）
pub struct EventBatcher<T: Send> {
    /// 待处理事件
    pending: Arc<RwLock<Vec<T>>>,
}

impl<T: Send + 'static> EventBatcher<T> {
    /// 创建事件批处理器
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 添加事件到批次
    pub async fn add_event(&self, event: T) {
        self.pending.write().await.push(event);
    }

    /// 获取并清空当前批次（手动批处理）
    pub async fn take_batch(&self) -> Vec<T> {
        let mut guard = self.pending.write().await;
        std::mem::take(&mut *guard)
    }

    /// 获取当前批次大小
    pub async fn batch_size(&self) -> usize {
        self.pending.read().await.len()
    }
}

impl<T: Send + 'static> Default for EventBatcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

type TaskBox = Box<dyn FnOnce() -> () + Send>;

/// 优先级任务调度器
pub struct PriorityScheduler {
    high_priority: Arc<tokio::sync::mpsc::UnboundedSender<TaskBox>>,
    low_priority: Arc<tokio::sync::mpsc::UnboundedSender<TaskBox>>,
}

impl PriorityScheduler {
    /// 创建优先级调度器
    pub fn new() -> Self {
        let (high_tx, mut high_rx) = tokio::sync::mpsc::unbounded_channel::<TaskBox>();
        let (low_tx, mut low_rx) = tokio::sync::mpsc::unbounded_channel::<TaskBox>();

        // 高优先级任务处理器
        tokio::spawn(async move {
            while let Some(task) = high_rx.recv().await {
                task();
            }
        });

        // 低优先级任务处理器
        tokio::spawn(async move {
            while let Some(task) = low_rx.recv().await {
                task();
            }
        });

        Self {
            high_priority: Arc::new(high_tx),
            low_priority: Arc::new(low_tx),
        }
    }

    /// 提交高优先级任务（立即执行）
    pub fn submit_high<F>(&self, task: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let _ = self.high_priority.send(Box::new(task));
    }

    /// 提交低优先级任务（后台执行）
    pub fn submit_low<F>(&self, task: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let _ = self.low_priority.send(Box::new(task));
    }
}

impl Default for PriorityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局性能优化器实例（懒加载）
pub static GLOBAL_OPTIMIZER: LazyLock<PerformanceOptimizer> =
    LazyLock::new(|| PerformanceOptimizer::new());

/// 全局优先级调度器实例（懒加载）
pub static GLOBAL_SCHEDULER: LazyLock<PriorityScheduler> =
    LazyLock::new(|| PriorityScheduler::new());

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_phase_tracking() {
        let optimizer = PerformanceOptimizer::new();

        {
            let _phase = optimizer.start_phase("test_phase").await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        tokio::time::sleep(Duration::from_millis(50)).await; // 等待异步记录

        let metrics = optimizer.get_startup_metrics().await;
        assert!(metrics.phase_durations.contains_key("test_phase"));
    }

    #[tokio::test]
    async fn test_debounce() {
        let optimizer = PerformanceOptimizer::new();

        let result = optimizer.debounce_event(|| async { 42 }).await;
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn test_parallel_init() {
        let tasks = vec![
            ("task1", || async { Ok(()) }),
            ("task2", || async { Ok(()) }),
        ];

        let results = PerformanceOptimizer::parallel_init(tasks).await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
