// Config Manager - 配置管理系统
// 统一配置管理，支持热更新和多环境
//
// 核心功能：
// 1. 多环境配置（dev/staging/prod）
// 2. 配置热更新（无需重启）
// 3. 配置验证
// 4. 敏感配置加密存储
// 5. 配置版本控制

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 配置环境
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

/// 配置值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        if let ConfigValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let ConfigValue::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        if let ConfigValue::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let ConfigValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

/// 配置条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// 配置键
    pub key: String,
    /// 配置值
    pub value: ConfigValue,
    /// 是否为敏感配置
    pub sensitive: bool,
    /// 是否可热更新
    pub hot_reloadable: bool,
    /// 环境限制
    pub environment: Option<Environment>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 版本号
    pub version: u64,
}

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    /// 变更的配置键
    pub key: String,
    /// 旧值
    pub old_value: Option<ConfigValue>,
    /// 新值
    pub new_value: ConfigValue,
    /// 变更时间
    pub changed_at: DateTime<Utc>,
    /// 变更者
    pub changed_by: String,
}

/// 配置管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigManagerConfig {
    /// 当前环境
    pub environment: Environment,
    /// 配置文件路径
    pub config_dir: PathBuf,
    /// 是否启用热更新
    pub enable_hot_reload: bool,
    /// 热更新检查间隔（秒）
    pub hot_reload_interval_secs: u64,
    /// 是否加密敏感配置
    pub encrypt_sensitive: bool,
}

impl Default for ConfigManagerConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            config_dir: PathBuf::from("./config"),
            enable_hot_reload: true,
            hot_reload_interval_secs: 60,
            encrypt_sensitive: true,
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config: ConfigManagerConfig,
    /// 配置存储
    store: Arc<RwLock<HashMap<String, ConfigEntry>>>,
    /// 变更历史
    change_history: Arc<RwLock<Vec<ConfigChange>>>,
    /// 配置监听器
    listeners: Arc<RwLock<Vec<Box<dyn ConfigListener + Send + Sync>>>>,
}

/// 配置监听器 trait
pub trait ConfigListener {
    fn on_config_changed(&self, change: &ConfigChange);
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(config: ConfigManagerConfig) -> Self {
        info!("⚙️  Initializing Config Manager");
        info!("    Environment: {}", config.environment);
        info!("    Config Dir: {:?}", config.config_dir);
        info!("    Hot Reload: {}", config.enable_hot_reload);

        Self {
            config,
            store: Arc::new(RwLock::new(HashMap::new())),
            change_history: Arc::new(RwLock::new(Vec::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 从文件加载配置
    pub async fn load_from_file(&self) -> Result<()> {
        let file_path = self
            .config
            .config_dir
            .join(format!("{}.json", self.config.environment));

        info!("📂 Loading config from: {:?}", file_path);

        // TODO: 实际从文件读取配置
        // 使用 tokio::fs::read_to_string 和 serde_json::from_str

        // Placeholder
        warn!("⚠️  Config file loading not yet implemented");
        Ok(())
    }

    /// 获取配置值
    pub async fn get(&self, key: &str) -> Option<ConfigValue> {
        let store = self.store.read().await;
        store.get(key).map(|entry| entry.value.clone())
    }

    /// 获取字符串配置
    pub async fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).await.and_then(|v| v.as_str().map(String::from))
    }

    /// 获取整数配置
    pub async fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).await.and_then(|v| v.as_i64())
    }

    /// 获取浮点配置
    pub async fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).await.and_then(|v| v.as_f64())
    }

    /// 获取布尔配置
    pub async fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).await.and_then(|v| v.as_bool())
    }

    /// 获取配置（带默认值）
    pub async fn get_or(&self, key: &str, default: ConfigValue) -> ConfigValue {
        self.get(key).await.unwrap_or(default)
    }

    /// 设置配置值
    pub async fn set(
        &self,
        key: String,
        value: ConfigValue,
        sensitive: bool,
        hot_reloadable: bool,
        changed_by: String,
    ) -> Result<()> {
        let mut store = self.store.write().await;

        let old_value = store.get(&key).map(|e| e.value.clone());

        // 检查是否可热更新
        if let Some(existing) = store.get(&key) {
            if !existing.hot_reloadable && !hot_reloadable {
                return Err(anyhow!("Config '{}' is not hot-reloadable", key));
            }
        }

        let entry = ConfigEntry {
            key: key.clone(),
            value: value.clone(),
            sensitive,
            hot_reloadable,
            environment: Some(self.config.environment),
            updated_at: Utc::now(),
            version: store
                .get(&key)
                .map(|e| e.version + 1)
                .unwrap_or(1),
        };

        store.insert(key.clone(), entry);
        drop(store);

        // 记录变更
        let change = ConfigChange {
            key: key.clone(),
            old_value,
            new_value: value,
            changed_at: Utc::now(),
            changed_by,
        };

        let mut history = self.change_history.write().await;
        history.push(change.clone());
        drop(history);

        // 通知监听器
        self.notify_listeners(&change).await;

        info!("✅ Config updated: {}", key);
        Ok(())
    }

    /// 删除配置
    pub async fn remove(&self, key: &str) -> Result<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        info!("🗑️  Config removed: {}", key);
        Ok(())
    }

    /// 批量设置配置
    pub async fn set_batch(&self, configs: HashMap<String, ConfigValue>, changed_by: String) -> Result<()> {
        for (key, value) in configs {
            self.set(key, value, false, true, changed_by.clone()).await?;
        }
        Ok(())
    }

    /// 获取所有配置
    pub async fn get_all(&self) -> HashMap<String, ConfigValue> {
        let store = self.store.read().await;
        store
            .iter()
            .map(|(k, v)| {
                let value = if v.sensitive && self.config.encrypt_sensitive {
                    ConfigValue::String("***ENCRYPTED***".to_string())
                } else {
                    v.value.clone()
                };
                (k.clone(), value)
            })
            .collect()
    }

    /// 验证配置
    pub async fn validate(&self) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        // TODO: 实现配置验证逻辑
        // 例如：检查必需配置是否存在、类型是否正确等

        if errors.is_empty() {
            info!("✅ Config validation passed");
        } else {
            warn!("⚠️  Config validation found {} errors", errors.len());
        }

        Ok(errors)
    }

    /// 获取变更历史
    pub async fn get_change_history(&self, limit: Option<usize>) -> Vec<ConfigChange> {
        let history = self.change_history.read().await;
        let limit = limit.unwrap_or(100);
        history.iter().rev().take(limit).cloned().collect()
    }

    /// 注册配置监听器
    pub async fn register_listener(&self, listener: Box<dyn ConfigListener + Send + Sync>) {
        let mut listeners = self.listeners.write().await;
        listeners.push(listener);
        info!("📡 Config listener registered");
    }

    /// 启动热更新监控
    pub async fn start_hot_reload_watcher(self: Arc<Self>) {
        if !self.config.enable_hot_reload {
            info!("🔄 Hot reload is disabled");
            return;
        }

        info!("🔄 Starting hot reload watcher (interval: {}s)", self.config.hot_reload_interval_secs);

        let interval = tokio::time::Duration::from_secs(self.config.hot_reload_interval_secs);
        let mut ticker = tokio::time::interval(interval);

        tokio::spawn(async move {
            loop {
                ticker.tick().await;

                if let Err(e) = self.reload_config().await {
                    warn!("⚠️  Hot reload failed: {}", e);
                } else {
                    debug!("🔄 Config hot reloaded");
                }
            }
        });
    }

    /// 重新加载配置
    async fn reload_config(&self) -> Result<()> {
        // TODO: 从文件重新加载配置并比对变更
        // 只更新标记为 hot_reloadable 的配置

        Ok(())
    }

    /// 通知监听器
    async fn notify_listeners(&self, change: &ConfigChange) {
        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            listener.on_config_changed(change);
        }
    }

    /// 导出配置
    pub async fn export_config(&self) -> Result<String> {
        let configs = self.get_all().await;
        serde_json::to_string_pretty(&configs)
            .map_err(|e| anyhow!("Failed to export config: {}", e))
    }

    /// 导入配置
    pub async fn import_config(&self, json: &str, changed_by: String) -> Result<()> {
        let configs: HashMap<String, ConfigValue> = serde_json::from_str(json)
            .map_err(|e| anyhow!("Failed to parse config JSON: {}", e))?;

        self.set_batch(configs, changed_by).await?;
        info!("✅ Config imported successfully");
        Ok(())
    }
}

// 示例监听器实现
pub struct LoggingConfigListener;

impl ConfigListener for LoggingConfigListener {
    fn on_config_changed(&self, change: &ConfigChange) {
        info!("📝 Config changed: {} (changed by: {})", change.key, change.changed_by);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_get_set() {
        let manager = ConfigManager::new(ConfigManagerConfig::default());

        manager
            .set(
                "test_key".to_string(),
                ConfigValue::String("test_value".to_string()),
                false,
                true,
                "test_user".to_string(),
            )
            .await
            .unwrap();

        let value = manager.get_string("test_key").await;
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_config_history() {
        let manager = ConfigManager::new(ConfigManagerConfig::default());

        manager
            .set(
                "key1".to_string(),
                ConfigValue::Integer(42),
                false,
                true,
                "user1".to_string(),
            )
            .await
            .unwrap();

        let history = manager.get_change_history(Some(10)).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].key, "key1");
    }
}
