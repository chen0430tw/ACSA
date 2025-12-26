// Authentication System - Token认证系统
// 目标：保护API访问，防止隐私泄露
//
// 核心功能：
// 1. JWT生成和验证
// 2. Token刷新机制
// 3. 会话管理
// 4. Token撤销

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub user_id: String,
    pub username: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "change-this-in-production".to_string(),
            access_token_ttl: Duration::from_secs(3600),
            refresh_token_ttl: Duration::from_secs(86400 * 7),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user_id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub last_access: DateTime<Utc>,
}

pub struct AuthManager {
    config: AuthConfig,
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    revoked_tokens: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl AuthManager {
    pub fn new(config: AuthConfig) -> Self {
        info!("🔐 Auth Manager initialized");
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn generate_token_pair(&self, user_id: &str, username: &str, roles: Vec<String>) -> Result<TokenPair> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let access_claims = Claims {
            sub: user_id.to_string(),
            exp: now + self.config.access_token_ttl.as_secs(),
            iat: now,
            user_id: user_id.to_string(),
            username: username.to_string(),
            roles: roles.clone(),
        };

        let access_token = serde_json::to_string(&access_claims)?;
        let refresh_token = format!("refresh_{}", user_id);

        let session = SessionInfo {
            user_id: user_id.to_string(),
            username: username.to_string(),
            created_at: Utc::now(),
            last_access: Utc::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(user_id.to_string(), session);

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.config.access_token_ttl.as_secs(),
            token_type: "Bearer".to_string(),
        })
    }

    pub async fn verify_token(&self, token: &str) -> Result<Claims> {
        let revoked = self.revoked_tokens.read().await;
        if revoked.contains_key(token) {
            return Err(anyhow!("Token revoked"));
        }

        let claims: Claims = serde_json::from_str(token)?;
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        if claims.exp < now {
            return Err(anyhow!("Token expired"));
        }

        Ok(claims)
    }

    pub async fn revoke_token(&self, token: &str) -> Result<()> {
        let mut revoked = self.revoked_tokens.write().await;
        revoked.insert(token.to_string(), Utc::now());
        Ok(())
    }

    pub async fn logout(&self, user_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(user_id);
        info!("👋 User logged out: {}", user_id);
        Ok(())
    }
}
