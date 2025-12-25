// Agent State - Agent状态持久化系统
// 长期记忆、会话恢复和用户偏好管理
//
// 核心功能：
// 1. Agent会话状态持久化
// 2. 长期记忆存储
// 3. 用户偏好管理
// 4. 对话历史管理
// 5. 状态快照和恢复

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::database::DatabaseManager;
use super::protocol::Protocol;

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// 会话ID
    pub session_id: String,
    /// 用户ID
    pub user_id: String,
    /// 当前Protocol
    pub current_protocol: Protocol,
    /// 对话轮数
    pub turn_count: u32,
    /// 会话开始时间
    pub started_at: DateTime<Utc>,
    /// 最后活跃时间
    pub last_active_at: DateTime<Utc>,
    /// 会话元数据
    pub metadata: HashMap<String, String>,
    /// 是否已结束
    pub is_ended: bool,
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息ID
    pub message_id: String,
    /// 会话ID
    pub session_id: String,
    /// 角色（user/assistant/system）
    pub role: String,
    /// 内容
    pub content: String,
    /// Protocol
    pub protocol: Option<Protocol>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 元数据（成本、模型等）
    pub metadata: HashMap<String, String>,
}

/// 长期记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermMemory {
    /// 记忆ID
    pub memory_id: String,
    /// 用户ID
    pub user_id: String,
    /// 记忆类型（fact/preference/event/skill）
    pub memory_type: String,
    /// 记忆内容
    pub content: String,
    /// 重要性（1-10）
    pub importance: u8,
    /// 访问次数
    pub access_count: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后访问时间
    pub last_accessed_at: DateTime<Utc>,
}

/// 用户偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    /// 用户ID
    pub user_id: String,
    /// 偏好Protocol
    pub preferred_protocol: Option<Protocol>,
    /// 语言偏好
    pub language: String,
    /// 响应风格（简洁/详细/技术/友好）
    pub response_style: String,
    /// 自定义设置
    pub custom_settings: HashMap<String, String>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// 快照ID
    pub snapshot_id: String,
    /// 会话ID
    pub session_id: String,
    /// 快照数据（JSON）
    pub snapshot_data: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// Agent状态管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateConfig {
    /// 是否启用持久化
    pub enable_persistence: bool,
    /// 会话超时时间（秒）
    pub session_timeout_secs: u64,
    /// 最大对话历史保留数
    pub max_history_messages: usize,
    /// 记忆重要性阈值（低于此值会被清理）
    pub memory_importance_threshold: u8,
}

impl Default for AgentStateConfig {
    fn default() -> Self {
        Self {
            enable_persistence: true,
            session_timeout_secs: 3600, // 1小时
            max_history_messages: 100,
            memory_importance_threshold: 3,
        }
    }
}

/// Agent状态管理器
pub struct AgentStateManager {
    config: AgentStateConfig,
    /// 数据库管理器（可选）
    database: Option<Arc<DatabaseManager>>,
    /// 会话缓存
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    /// 消息缓存
    messages: Arc<RwLock<HashMap<String, Vec<Message>>>>,
    /// 长期记忆缓存
    memories: Arc<RwLock<HashMap<String, Vec<LongTermMemory>>>>,
    /// 用户偏好缓存
    preferences: Arc<RwLock<HashMap<String, UserPreference>>>,
}

impl AgentStateManager {
    /// 创建新的Agent状态管理器
    pub fn new(config: AgentStateConfig, database: Option<Arc<DatabaseManager>>) -> Self {
        info!("💾 Initializing Agent State Manager");
        info!("    Persistence: {}", config.enable_persistence);
        info!("    Session Timeout: {}s", config.session_timeout_secs);

        Self {
            config,
            database,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            memories: Arc::new(RwLock::new(HashMap::new())),
            preferences: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新会话
    pub async fn create_session(
        &self,
        user_id: String,
        protocol: Protocol,
    ) -> Result<SessionState> {
        let session_id = format!("session_{}_{}", user_id, Utc::now().timestamp_millis());

        let session = SessionState {
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            current_protocol: protocol,
            turn_count: 0,
            started_at: Utc::now(),
            last_active_at: Utc::now(),
            metadata: HashMap::new(),
            is_ended: false,
        };

        // 缓存
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        // 持久化
        if self.config.enable_persistence {
            self.persist_session(&session).await?;
        }

        info!("🆕 Created session: {}", session_id);
        Ok(session)
    }

    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Result<SessionState> {
        // 先查缓存
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                return Ok(session.clone());
            }
        }

        // 从数据库加载
        if self.config.enable_persistence && self.database.is_some() {
            self.load_session(session_id).await
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// 添加消息
    pub async fn add_message(&self, message: Message) -> Result<()> {
        // 更新会话活跃时间
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&message.session_id) {
                session.last_active_at = Utc::now();
                session.turn_count += 1;
            }
        }

        // 缓存消息
        {
            let mut messages = self.messages.write().await;
            messages
                .entry(message.session_id.clone())
                .or_insert_with(Vec::new)
                .push(message.clone());

            // 限制历史消息数量
            if let Some(msg_list) = messages.get_mut(&message.session_id) {
                if msg_list.len() > self.config.max_history_messages {
                    msg_list.drain(0..msg_list.len() - self.config.max_history_messages);
                }
            }
        }

        // 持久化
        if self.config.enable_persistence {
            self.persist_message(&message).await?;
        }

        Ok(())
    }

    /// 获取对话历史
    pub async fn get_conversation_history(&self, session_id: &str, limit: Option<usize>) -> Result<Vec<Message>> {
        let messages = self.messages.read().await;
        let msgs = messages
            .get(session_id)
            .ok_or_else(|| anyhow!("No messages found for session: {}", session_id))?;

        let limit = limit.unwrap_or(self.config.max_history_messages);
        Ok(msgs.iter().rev().take(limit).rev().cloned().collect())
    }

    /// 添加长期记忆
    pub async fn add_memory(&self, memory: LongTermMemory) -> Result<()> {
        let mut memories = self.memories.write().await;
        memories
            .entry(memory.user_id.clone())
            .or_insert_with(Vec::new)
            .push(memory.clone());

        // 持久化
        if self.config.enable_persistence {
            self.persist_memory(&memory).await?;
        }

        info!("🧠 Added long-term memory: {}", memory.memory_type);
        Ok(())
    }

    /// 检索记忆
    pub async fn retrieve_memories(
        &self,
        user_id: &str,
        memory_type: Option<&str>,
        limit: usize,
    ) -> Vec<LongTermMemory> {
        let memories = self.memories.read().await;

        let user_memories = match memories.get(user_id) {
            Some(mems) => mems,
            None => return Vec::new(),
        };

        let mut filtered: Vec<LongTermMemory> = user_memories
            .iter()
            .filter(|m| {
                if let Some(mtype) = memory_type {
                    m.memory_type == mtype
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // 按重要性和访问时间排序
        filtered.sort_by(|a, b| {
            let a_score = a.importance as f64 + (a.access_count as f64).ln();
            let b_score = b.importance as f64 + (b.access_count as f64).ln();
            b_score.partial_cmp(&a_score).unwrap()
        });

        filtered.into_iter().take(limit).collect()
    }

    /// 保存用户偏好
    pub async fn save_preference(&self, preference: UserPreference) -> Result<()> {
        let mut preferences = self.preferences.write().await;
        preferences.insert(preference.user_id.clone(), preference.clone());

        // 持久化
        if self.config.enable_persistence {
            self.persist_preference(&preference).await?;
        }

        info!("⚙️  Saved user preference for: {}", preference.user_id);
        Ok(())
    }

    /// 获取用户偏好
    pub async fn get_preference(&self, user_id: &str) -> Option<UserPreference> {
        let preferences = self.preferences.read().await;
        preferences.get(user_id).cloned()
    }

    /// 创建状态快照
    pub async fn create_snapshot(&self, session_id: &str) -> Result<StateSnapshot> {
        let session = self.get_session(session_id).await?;
        let messages = self.get_conversation_history(session_id, None).await?;

        let snapshot_data = serde_json::to_string(&(session, messages))?;

        let snapshot = StateSnapshot {
            snapshot_id: format!("snapshot_{}_{}", session_id, Utc::now().timestamp_millis()),
            session_id: session_id.to_string(),
            snapshot_data,
            created_at: Utc::now(),
        };

        // 持久化快照
        if self.config.enable_persistence {
            self.persist_snapshot(&snapshot).await?;
        }

        info!("📸 Created state snapshot: {}", snapshot.snapshot_id);
        Ok(snapshot)
    }

    /// 从快照恢复
    pub async fn restore_from_snapshot(&self, snapshot_id: &str) -> Result<String> {
        // TODO: 从数据库加载快照
        // 恢复会话和消息
        info!("🔄 Restoring from snapshot: {}", snapshot_id);
        Ok("session_id".to_string())
    }

    /// 结束会话
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.is_ended = true;

            // 持久化
            if self.config.enable_persistence {
                self.persist_session(session).await?;
            }
        }

        info!("🔚 Ended session: {}", session_id);
        Ok(())
    }

    // ===== 持久化方法（TODO：实现实际数据库操作）=====

    async fn persist_session(&self, _session: &SessionState) -> Result<()> {
        // TODO: INSERT INTO sessions
        Ok(())
    }

    async fn load_session(&self, _session_id: &str) -> Result<SessionState> {
        // TODO: SELECT FROM sessions
        Err(anyhow!("Database persistence not implemented"))
    }

    async fn persist_message(&self, _message: &Message) -> Result<()> {
        // TODO: INSERT INTO messages
        Ok(())
    }

    async fn persist_memory(&self, _memory: &LongTermMemory) -> Result<()> {
        // TODO: INSERT INTO memories
        Ok(())
    }

    async fn persist_preference(&self, _preference: &UserPreference) -> Result<()> {
        // TODO: INSERT OR REPLACE INTO preferences
        Ok(())
    }

    async fn persist_snapshot(&self, _snapshot: &StateSnapshot) -> Result<()> {
        // TODO: INSERT INTO snapshots
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let manager = AgentStateManager::new(AgentStateConfig::default(), None);

        let session = manager
            .create_session("user1".to_string(), Protocol::Architect)
            .await
            .unwrap();

        assert_eq!(session.user_id, "user1");
        assert_eq!(session.current_protocol, Protocol::Architect);
    }

    #[tokio::test]
    async fn test_message_history() {
        let manager = AgentStateManager::new(AgentStateConfig::default(), None);

        let session = manager
            .create_session("user1".to_string(), Protocol::Architect)
            .await
            .unwrap();

        let message = Message {
            message_id: "msg1".to_string(),
            session_id: session.session_id.clone(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            protocol: Some(Protocol::Architect),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        manager.add_message(message).await.unwrap();

        let history = manager
            .get_conversation_history(&session.session_id, None)
            .await
            .unwrap();

        assert_eq!(history.len(), 1);
    }
}
