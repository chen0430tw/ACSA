// Terminal Server - 终端服务端
// 目标：支持终端部署，提供socket通信和心跳包机制
//
// 核心功能：
// 1. WebSocket服务器：双向通信
// 2. 心跳包机制：检测连接状态
// 3. 会话管理：多客户端支持
// 4. 消息队列：异步消息处理
// 5. 自动重连：客户端断线恢复

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub heartbeat_interval_secs: u64,
    pub heartbeat_timeout_secs: u64,
    pub max_connections: usize,
    pub enable_compression: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8765,
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            max_connections: 1000,
            enable_compression: true,
        }
    }
}

/// 客户端连接
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub id: String,
    pub addr: String,
    pub connected_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// 心跳包
    Heartbeat {
        timestamp: DateTime<Utc>,
    },
    /// 心跳响应
    HeartbeatAck {
        timestamp: DateTime<Utc>,
    },
    /// 文本消息
    Text {
        content: String,
        metadata: HashMap<String, String>,
    },
    /// 系统消息
    System {
        event: String,
        data: serde_json::Value,
    },
    /// 错误消息
    Error {
        code: u32,
        message: String,
    },
}

/// 终端服务器
pub struct TerminalServer {
    config: ServerConfig,
    connections: Arc<RwLock<HashMap<String, ClientConnection>>>,
    message_tx: mpsc::UnboundedSender<(String, WsMessage)>,
    message_rx: Arc<RwLock<mpsc::UnboundedReceiver<(String, WsMessage)>>>,
}

impl TerminalServer {
    pub fn new(config: ServerConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        info!("🖥️ Terminal Server initialized");
        info!("   - Bind: {}:{}", config.bind_address, config.port);
        info!("   - Heartbeat: {}s / timeout: {}s",
            config.heartbeat_interval_secs,
            config.heartbeat_timeout_secs
        );
        info!("   - Max connections: {}", config.max_connections);

        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            message_tx: tx,
            message_rx: Arc::new(RwLock::new(rx)),
        }
    }

    /// 启动服务器
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Terminal Server on {}:{}",
            self.config.bind_address, self.config.port
        );

        // 启动心跳检查任务
        self.start_heartbeat_checker();

        // 实际的WebSocket服务器实现需要依赖tokio-tungstenite等库
        // 这里提供基本框架
        Ok(())
    }

    /// 添加客户端连接
    pub async fn add_client(&self, client_id: String, addr: String) -> Result<()> {
        let mut connections = self.connections.write().await;

        if connections.len() >= self.config.max_connections {
            return Err(anyhow!("Max connections reached"));
        }

        let client = ClientConnection {
            id: client_id.clone(),
            addr,
            connected_at: Utc::now(),
            last_heartbeat: Utc::now(),
            metadata: HashMap::new(),
        };

        connections.insert(client_id.clone(), client);

        info!("🔗 Client connected: {}", client_id);
        Ok(())
    }

    /// 移除客户端连接
    pub async fn remove_client(&self, client_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.remove(client_id);

        info!("🔌 Client disconnected: {}", client_id);
        Ok(())
    }

    /// 更新心跳时间
    pub async fn update_heartbeat(&self, client_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;

        if let Some(client) = connections.get_mut(client_id) {
            client.last_heartbeat = Utc::now();
            debug!("💓 Heartbeat updated: {}", client_id);
            Ok(())
        } else {
            Err(anyhow!("Client not found: {}", client_id))
        }
    }

    /// 发送消息到客户端
    pub async fn send_message(&self, client_id: String, message: WsMessage) -> Result<()> {
        self.message_tx.send((client_id.clone(), message))?;
        debug!("📤 Message queued for client: {}", client_id);
        Ok(())
    }

    /// 广播消息到所有客户端
    pub async fn broadcast(&self, message: WsMessage) -> Result<()> {
        let connections = self.connections.read().await;

        for client_id in connections.keys() {
            self.message_tx.send((client_id.clone(), message.clone()))?;
        }

        debug!("📢 Message broadcast to {} clients", connections.len());
        Ok(())
    }

    /// 启动心跳检查器
    fn start_heartbeat_checker(&self) {
        let connections = self.connections.clone();
        let timeout_secs = self.config.heartbeat_timeout_secs;
        let interval_secs = self.config.heartbeat_interval_secs;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                let mut connections_guard = connections.write().await;
                let now = Utc::now();
                let mut to_remove = Vec::new();

                for (client_id, client) in connections_guard.iter() {
                    let elapsed = (now - client.last_heartbeat).num_seconds() as u64;

                    if elapsed > timeout_secs {
                        warn!("⏱️ Client timeout: {} ({}s)", client_id, elapsed);
                        to_remove.push(client_id.clone());
                    }
                }

                for client_id in to_remove {
                    connections_guard.remove(&client_id);
                    info!("🔌 Client removed due to timeout: {}", client_id);
                }
            }
        });

        info!("💓 Heartbeat checker started");
    }

    /// 获取连接数
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// 获取所有客户端ID
    pub async fn get_client_ids(&self) -> Vec<String> {
        self.connections.read().await.keys().cloned().collect()
    }

    /// 获取客户端信息
    pub async fn get_client_info(&self, client_id: &str) -> Option<ClientConnection> {
        self.connections.read().await.get(client_id).cloned()
    }
}

/// 消息处理器trait
pub trait MessageHandler: Send + Sync {
    fn handle_message(&self, client_id: &str, message: WsMessage) -> Result<WsMessage>;
}

/// 默认消息处理器
pub struct DefaultHandler;

impl MessageHandler for DefaultHandler {
    fn handle_message(&self, client_id: &str, message: WsMessage) -> Result<WsMessage> {
        match message {
            WsMessage::Heartbeat { timestamp } => {
                Ok(WsMessage::HeartbeatAck { timestamp })
            }
            WsMessage::Text { content, metadata } => {
                info!("📨 Received from {}: {}", client_id, content);
                Ok(WsMessage::System {
                    event: "message_received".to_string(),
                    data: serde_json::json!({ "status": "ok" }),
                })
            }
            _ => Ok(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = TerminalServer::new(ServerConfig::default());
        assert_eq!(server.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_client_management() {
        let server = TerminalServer::new(ServerConfig::default());

        server.add_client("client1".to_string(), "127.0.0.1:1234".to_string())
            .await
            .unwrap();

        assert_eq!(server.connection_count().await, 1);

        server.remove_client("client1").await.unwrap();
        assert_eq!(server.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_heartbeat_update() {
        let server = TerminalServer::new(ServerConfig::default());

        server.add_client("client1".to_string(), "127.0.0.1:1234".to_string())
            .await
            .unwrap();

        let before = server.get_client_info("client1").await.unwrap();
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        server.update_heartbeat("client1").await.unwrap();

        let after = server.get_client_info("client1").await.unwrap();
        assert!(after.last_heartbeat > before.last_heartbeat);
    }
}
