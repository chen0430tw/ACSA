// Event Bus - 事件总线系统
// 解耦模块间通信，实现发布/订阅模式
//
// 核心功能：
// 1. 事件发布/订阅
// 2. 异步事件处理
// 3. 事件过滤
// 4. 事件历史记录
// 5. 死信队列

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// 事件类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// 系统事件
    System(String),
    /// AI事件
    Ai(String),
    /// 用户事件
    User(String),
    /// 数据事件
    Data(String),
    /// 自定义事件
    Custom(String),
}

/// 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// 事件ID
    pub event_id: String,
    /// 事件类型
    pub event_type: EventType,
    /// 事件源
    pub source: String,
    /// 事件数据
    pub data: serde_json::Value,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 事件处理器trait
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理事件
    async fn handle(&self, event: &Event) -> Result<()>;

    /// 事件过滤（返回true表示处理此事件）
    fn filter(&self, event_type: &EventType) -> bool;
}

/// 订阅者
struct Subscriber {
    /// 订阅者ID
    id: String,
    /// 事件处理器
    handler: Arc<dyn EventHandler>,
    /// 订阅的事件类型
    event_types: Vec<EventType>,
}

/// 事件总线配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// 事件队列容量
    pub queue_capacity: usize,
    /// 是否启用事件历史
    pub enable_history: bool,
    /// 历史记录容量
    pub history_capacity: usize,
    /// 是否启用死信队列
    pub enable_dead_letter: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 1000,
            enable_history: true,
            history_capacity: 10000,
            enable_dead_letter: true,
        }
    }
}

/// 事件总线
pub struct EventBus {
    config: EventBusConfig,
    /// 订阅者列表
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
    /// 事件发送通道
    event_tx: mpsc::UnboundedSender<Event>,
    /// 事件接收通道
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<Event>>>,
    /// 事件历史
    history: Arc<RwLock<Vec<Event>>>,
    /// 死信队列
    dead_letter: Arc<RwLock<Vec<(Event, String)>>>,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new(config: EventBusConfig) -> Self {
        info!("🚌 Initializing Event Bus");
        info!("    Queue Capacity: {}", config.queue_capacity);
        info!("    History: {}", config.enable_history);

        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            config,
            subscribers: Arc::new(RwLock::new(Vec::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            history: Arc::new(RwLock::new(Vec::new())),
            dead_letter: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 订阅事件
    pub async fn subscribe(
        &self,
        subscriber_id: String,
        handler: Arc<dyn EventHandler>,
        event_types: Vec<EventType>,
    ) -> Result<()> {
        let subscriber = Subscriber {
            id: subscriber_id.clone(),
            handler,
            event_types,
        };

        let mut subscribers = self.subscribers.write().await;
        subscribers.push(subscriber);

        info!("📬 Subscriber registered: {}", subscriber_id);
        Ok(())
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, subscriber_id: &str) -> Result<()> {
        let mut subscribers = self.subscribers.write().await;
        subscribers.retain(|s| s.id != subscriber_id);

        info!("📪 Subscriber unregistered: {}", subscriber_id);
        Ok(())
    }

    /// 发布事件
    pub async fn publish(&self, event: Event) -> Result<()> {
        debug!("📤 Publishing event: {:?}", event.event_type);

        // 记录历史
        if self.config.enable_history {
            let mut history = self.history.write().await;
            history.push(event.clone());

            // 限制历史容量
            let history_len = history.len();
            if history_len > self.config.history_capacity {
                let drain_count = history_len - self.config.history_capacity;
                history.drain(0..drain_count);
            }
        }

        // 发送到队列
        self.event_tx
            .send(event)
            .map_err(|e| anyhow::anyhow!("Failed to publish event: {}", e))?;

        Ok(())
    }

    /// 启动事件处理循环
    pub async fn start(self: Arc<Self>) {
        info!("🚀 Starting Event Bus processing loop");

        tokio::spawn(async move {
            loop {
                // 接收事件
                let event = {
                    let mut rx = self.event_rx.write().await;
                    rx.recv().await
                };

                if let Some(event) = event {
                    self.process_event(event).await;
                } else {
                    warn!("📥 Event channel closed");
                    break;
                }
            }
        });
    }

    /// 处理事件
    async fn process_event(&self, event: Event) {
        debug!("📥 Processing event: {:?}", event.event_type);

        let subscribers = self.subscribers.read().await;

        // 找到所有匹配的订阅者
        let mut matched_subscribers = Vec::new();
        for subscriber in subscribers.iter() {
            // 检查是否订阅了此类型事件
            if subscriber
                .event_types
                .iter()
                .any(|t| t == &event.event_type)
                || subscriber.handler.filter(&event.event_type)
            {
                matched_subscribers.push(subscriber.handler.clone());
            }
        }

        drop(subscribers);

        // 异步并发处理
        let mut handles = Vec::new();
        for handler in matched_subscribers {
            let event_clone = event.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = handler.handle(&event_clone).await {
                    warn!("⚠️  Event handler failed: {}", e);
                    Err((event_clone, e.to_string()))
                } else {
                    Ok(())
                }
            });
            handles.push(handle);
        }

        // 等待所有处理器完成
        for handle in handles {
            if let Ok(Err((failed_event, error))) = handle.await {
                // 记录到死信队列
                if self.config.enable_dead_letter {
                    let mut dead_letter = self.dead_letter.write().await;
                    dead_letter.push((failed_event, error));
                }
            }
        }
    }

    /// 获取事件历史
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<Event> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(100);
        history.iter().rev().take(limit).cloned().collect()
    }

    /// 获取死信队列
    pub async fn get_dead_letter(&self) -> Vec<(Event, String)> {
        self.dead_letter.read().await.clone()
    }

    /// 清空死信队列
    pub async fn clear_dead_letter(&self) {
        let mut dead_letter = self.dead_letter.write().await;
        dead_letter.clear();
        info!("🗑️  Dead letter queue cleared");
    }
}

// ===== 内置事件处理器示例 =====

/// 日志事件处理器
pub struct LoggingEventHandler;

#[async_trait::async_trait]
impl EventHandler for LoggingEventHandler {
    async fn handle(&self, event: &Event) -> Result<()> {
        info!(
            "📝 Event logged: {} from {} at {}",
            event.event_id, event.source, event.timestamp
        );
        Ok(())
    }

    fn filter(&self, _event_type: &EventType) -> bool {
        true // 接受所有事件
    }
}

/// 指标收集事件处理器
pub struct MetricsEventHandler {
    event_counts: Arc<RwLock<HashMap<String, u64>>>,
}

impl MetricsEventHandler {
    pub fn new() -> Self {
        Self {
            event_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_counts(&self) -> HashMap<String, u64> {
        self.event_counts.read().await.clone()
    }
}

#[async_trait::async_trait]
impl EventHandler for MetricsEventHandler {
    async fn handle(&self, event: &Event) -> Result<()> {
        let mut counts = self.event_counts.write().await;
        let key = format!("{:?}", event.event_type);
        *counts.entry(key).or_insert(0) += 1;
        Ok(())
    }

    fn filter(&self, _event_type: &EventType) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus() {
        let bus = Arc::new(EventBus::new(EventBusConfig::default()));

        // 启动事件总线
        bus.clone().start().await;

        // 订阅事件
        let handler = Arc::new(LoggingEventHandler);
        bus.subscribe(
            "logger".to_string(),
            handler,
            vec![EventType::System("test".to_string())],
        )
        .await
        .unwrap();

        // 发布事件
        let event = Event {
            event_id: "test1".to_string(),
            event_type: EventType::System("test".to_string()),
            source: "test".to_string(),
            data: serde_json::json!({"message": "test"}),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        bus.publish(event).await.unwrap();

        // 等待处理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // 检查历史
        let history = bus.get_history(Some(10)).await;
        assert_eq!(history.len(), 1);
    }
}
