// Plugin System - 插件系统
// 动态加载和管理自定义Agent插件
//
// 核心功能：
// 1. 插件注册和发现
// 2. 动态加载（.so/.dll/.dylib）
// 3. 插件生命周期管理
// 4. 资源隔离和限制
// 5. 插件通信（via Event Bus）
// 6. 热重载（Hot Reload）
// 7. 依赖管理

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::event_bus::{Event, EventBus};
use super::protocol::Protocol;

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 错误
    Error,
}

/// 插件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// Agent插件（完整的Agent实现）
    Agent,
    /// 中间件插件（请求/响应拦截）
    Middleware,
    /// 工具插件（提供工具函数）
    Tool,
    /// 模型插件（自定义模型Provider）
    ModelProvider,
    /// 协议插件（自定义Protocol）
    ProtocolExtension,
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件ID
    pub plugin_id: String,
    /// 插件名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 作者
    pub author: String,
    /// 描述
    pub description: String,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 依赖的其他插件
    pub dependencies: Vec<String>,
    /// 支持的Protocol（如果是Agent插件）
    pub supported_protocols: Vec<Protocol>,
    /// 资源限制
    pub resource_limits: ResourceLimits,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 最大内存（MB）
    pub max_memory_mb: u64,
    /// 最大CPU占用（%）
    pub max_cpu_percent: u8,
    /// 最大并发请求数
    pub max_concurrent_requests: usize,
    /// 请求超时时间（秒）
    pub request_timeout_secs: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_percent: 50,
            max_concurrent_requests: 10,
            request_timeout_secs: 30,
        }
    }
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 配置项
    pub settings: HashMap<String, String>,
}

/// 插件实例
pub struct Plugin {
    /// 元数据
    pub metadata: PluginMetadata,
    /// 当前状态
    pub state: Arc<RwLock<PluginState>>,
    /// 配置
    pub config: PluginConfig,
    /// 插件路径
    pub plugin_path: PathBuf,
    /// 插件处理器
    pub handler: Arc<RwLock<Option<Box<dyn PluginHandler>>>>,
    /// 加载时间
    pub loaded_at: Option<DateTime<Utc>>,
    /// 统计信息
    pub stats: Arc<RwLock<PluginStats>>,
}

/// 插件统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功数
    pub successful_requests: u64,
    /// 失败数
    pub failed_requests: u64,
    /// 平均响应时间（ms）
    pub avg_response_time_ms: f64,
    /// 当前并发数
    pub current_concurrent: usize,
    /// 内存使用（MB）
    pub memory_usage_mb: f64,
}

/// 插件处理器trait
#[async_trait::async_trait]
pub trait PluginHandler: Send + Sync {
    /// 初始化插件
    async fn initialize(&mut self, config: &PluginConfig) -> Result<()>;

    /// 处理请求
    async fn handle_request(&self, request: PluginRequest) -> Result<PluginResponse>;

    /// 处理事件
    async fn handle_event(&self, event: &Event) -> Result<()>;

    /// 关闭插件
    async fn shutdown(&mut self) -> Result<()>;

    /// 健康检查
    async fn health_check(&self) -> bool;
}

/// 插件请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    /// 请求ID
    pub request_id: String,
    /// 用户输入
    pub user_input: String,
    /// 上下文
    pub context: HashMap<String, String>,
    /// Protocol
    pub protocol: Option<Protocol>,
}

/// 插件响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    /// 请求ID
    pub request_id: String,
    /// 响应内容
    pub content: String,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 是否成功
    pub success: bool,
}

/// 插件系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemConfig {
    /// 插件目录
    pub plugins_dir: PathBuf,
    /// 是否启用热重载
    pub enable_hot_reload: bool,
    /// 重载检查间隔（秒）
    pub reload_check_interval_secs: u64,
    /// 是否启用沙箱
    pub enable_sandboxing: bool,
    /// 最大插件数
    pub max_plugins: usize,
}

impl Default for PluginSystemConfig {
    fn default() -> Self {
        Self {
            plugins_dir: PathBuf::from("./plugins"),
            enable_hot_reload: true,
            reload_check_interval_secs: 5,
            enable_sandboxing: true,
            max_plugins: 100,
        }
    }
}

/// 插件系统
pub struct PluginSystem {
    config: PluginSystemConfig,
    /// 已加载的插件
    plugins: Arc<RwLock<HashMap<String, Arc<Plugin>>>>,
    /// Event Bus（用于插件通信）
    event_bus: Option<Arc<EventBus>>,
    /// 插件注册表（插件ID -> 元数据）
    registry: Arc<RwLock<HashMap<String, PluginMetadata>>>,
}

impl PluginSystem {
    /// 创建新的插件系统
    pub fn new(config: PluginSystemConfig, event_bus: Option<Arc<EventBus>>) -> Self {
        info!("🔌 Initializing Plugin System");
        info!("    Plugins Dir: {:?}", config.plugins_dir);
        info!("    Hot Reload: {}", config.enable_hot_reload);
        info!("    Sandboxing: {}", config.enable_sandboxing);

        Self {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 发现插件（扫描插件目录）
    pub async fn discover_plugins(&self) -> Result<Vec<PluginMetadata>> {
        info!("🔍 Discovering plugins in {:?}", self.config.plugins_dir);

        // TODO: 实际实现插件扫描
        // 1. 扫描插件目录
        // 2. 读取每个插件的 manifest.json
        // 3. 验证插件签名（安全性）
        // 4. 解析元数据

        // Placeholder
        Ok(Vec::new())
    }

    /// 注册插件
    pub async fn register_plugin(&self, metadata: PluginMetadata) -> Result<()> {
        let mut registry = self.registry.write().await;

        if registry.len() >= self.config.max_plugins {
            return Err(anyhow!("Maximum number of plugins ({}) reached", self.config.max_plugins));
        }

        registry.insert(metadata.plugin_id.clone(), metadata.clone());
        info!("📝 Registered plugin: {} v{}", metadata.name, metadata.version);

        Ok(())
    }

    /// 加载插件
    pub async fn load_plugin(
        &self,
        plugin_id: &str,
        plugin_path: PathBuf,
        config: PluginConfig,
    ) -> Result<()> {
        info!("📦 Loading plugin: {}", plugin_id);

        // 检查是否已加载
        {
            let plugins = self.plugins.read().await;
            if plugins.contains_key(plugin_id) {
                return Err(anyhow!("Plugin already loaded: {}", plugin_id));
            }
        }

        // 获取元数据
        let metadata = {
            let registry = self.registry.read().await;
            registry
                .get(plugin_id)
                .cloned()
                .ok_or_else(|| anyhow!("Plugin not registered: {}", plugin_id))?
        };

        // 检查依赖
        self.check_dependencies(&metadata.dependencies).await?;

        // 创建插件实例
        let plugin = Arc::new(Plugin {
            metadata: metadata.clone(),
            state: Arc::new(RwLock::new(PluginState::Loading)),
            config: config.clone(),
            plugin_path: plugin_path.clone(),
            handler: Arc::new(RwLock::new(None)),
            loaded_at: None,
            stats: Arc::new(RwLock::new(PluginStats::default())),
        });

        // TODO: 实际动态加载
        // 使用 libloading crate:
        // unsafe {
        //     let lib = Library::new(&plugin_path)?;
        //     let create_fn: Symbol<fn() -> Box<dyn PluginHandler>> = lib.get(b"create_plugin")?;
        //     let handler = create_fn();
        //     ...
        // }

        // 初始化插件（placeholder）
        // if let Some(mut handler) = plugin.handler.write().await.as_deref_mut() {
        //     handler.initialize(&config).await?;
        // }

        // 更新状态
        *plugin.state.write().await = PluginState::Loaded;

        // 存储插件
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_id.to_string(), plugin);

        info!("✅ Plugin loaded: {} v{}", metadata.name, metadata.version);
        Ok(())
    }

    /// 卸载插件
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        info!("📤 Unloading plugin: {}", plugin_id);

        let mut plugins = self.plugins.write().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?
            .clone();

        // 关闭插件
        if let Some(handler) = plugin.handler.write().await.as_mut() {
            handler.shutdown().await?;
        }

        // 更新状态
        *plugin.state.write().await = PluginState::Unloaded;

        // 移除
        plugins.remove(plugin_id);

        info!("✅ Plugin unloaded: {}", plugin_id);
        Ok(())
    }

    /// 重新加载插件（热重载）
    pub async fn reload_plugin(&self, plugin_id: &str) -> Result<()> {
        info!("🔄 Reloading plugin: {}", plugin_id);

        // 获取插件信息
        let (plugin_path, config) = {
            let plugins = self.plugins.read().await;
            let plugin = plugins
                .get(plugin_id)
                .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;

            (plugin.plugin_path.clone(), plugin.config.clone())
        };

        // 卸载
        self.unload_plugin(plugin_id).await?;

        // 重新加载
        self.load_plugin(plugin_id, plugin_path, config).await?;

        info!("✅ Plugin reloaded: {}", plugin_id);
        Ok(())
    }

    /// 启动插件
    pub async fn start_plugin(&self, plugin_id: &str) -> Result<()> {
        let plugins = self.plugins.read().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;

        *plugin.state.write().await = PluginState::Running;
        info!("▶️  Started plugin: {}", plugin_id);

        Ok(())
    }

    /// 停止插件
    pub async fn stop_plugin(&self, plugin_id: &str) -> Result<()> {
        let plugins = self.plugins.read().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;

        *plugin.state.write().await = PluginState::Paused;
        info!("⏸️  Stopped plugin: {}", plugin_id);

        Ok(())
    }

    /// 发送请求到插件
    pub async fn send_request(
        &self,
        plugin_id: &str,
        request: PluginRequest,
    ) -> Result<PluginResponse> {
        let start = std::time::Instant::now();

        let plugins = self.plugins.read().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;

        // 检查状态
        let state = *plugin.state.read().await;
        if state != PluginState::Running {
            return Err(anyhow!("Plugin not running: {:?}", state));
        }

        // 检查资源限制
        let stats = plugin.stats.read().await;
        if stats.current_concurrent >= plugin.metadata.resource_limits.max_concurrent_requests {
            return Err(anyhow!("Plugin concurrent request limit exceeded"));
        }
        drop(stats);

        // 更新统计
        {
            let mut stats = plugin.stats.write().await;
            stats.current_concurrent += 1;
            stats.total_requests += 1;
        }

        // 调用处理器
        let response = if let Some(handler) = plugin.handler.read().await.as_ref() {
            // TODO: 添加超时控制
            match handler.handle_request(request).await {
                Ok(resp) => {
                    let mut stats = plugin.stats.write().await;
                    stats.successful_requests += 1;
                    Ok(resp)
                }
                Err(e) => {
                    let mut stats = plugin.stats.write().await;
                    stats.failed_requests += 1;
                    Err(e)
                }
            }
        } else {
            Err(anyhow!("Plugin handler not initialized"))
        };

        // 更新响应时间
        let elapsed = start.elapsed().as_millis() as f64;
        {
            let mut stats = plugin.stats.write().await;
            stats.current_concurrent = stats.current_concurrent.saturating_sub(1);
            let total = stats.total_requests as f64;
            stats.avg_response_time_ms =
                (stats.avg_response_time_ms * (total - 1.0) + elapsed) / total;
        }

        response
    }

    /// 获取插件列表
    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        let registry = self.registry.read().await;
        registry.values().cloned().collect()
    }

    /// 获取插件状态
    pub async fn get_plugin_state(&self, plugin_id: &str) -> Option<PluginState> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|p| *p.state.blocking_read())
    }

    /// 获取插件统计
    pub async fn get_plugin_stats(&self, plugin_id: &str) -> Option<PluginStats> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|p| p.stats.blocking_read().clone())
    }

    /// 健康检查所有插件
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        let plugins = self.plugins.read().await;

        for (plugin_id, plugin) in plugins.iter() {
            let healthy = if let Some(handler) = plugin.handler.read().await.as_ref() {
                handler.health_check().await
            } else {
                false
            };

            results.insert(plugin_id.clone(), healthy);

            if !healthy {
                warn!("⚠️  Plugin unhealthy: {}", plugin_id);
            }
        }

        results
    }

    // ===== 内部方法 =====

    /// 检查依赖
    async fn check_dependencies(&self, dependencies: &[String]) -> Result<()> {
        let plugins = self.plugins.read().await;

        for dep_id in dependencies {
            if !plugins.contains_key(dep_id) {
                return Err(anyhow!("Missing dependency plugin: {}", dep_id));
            }

            let dep_plugin = plugins.get(dep_id).unwrap();
            let state = *dep_plugin.state.read().await;

            if state == PluginState::Error || state == PluginState::Unloaded {
                return Err(anyhow!("Dependency plugin not available: {} ({:?})", dep_id, state));
            }
        }

        Ok(())
    }

    /// 启动热重载监控
    pub async fn start_hot_reload_monitor(self: Arc<Self>) {
        if !self.config.enable_hot_reload {
            return;
        }

        info!("🔥 Starting hot reload monitor");

        tokio::spawn(async move {
            let interval = tokio::time::Duration::from_secs(self.config.reload_check_interval_secs);

            loop {
                tokio::time::sleep(interval).await;

                // TODO: 实现文件变化检测
                // 1. 监控插件目录变化（使用 notify crate）
                // 2. 检测插件文件修改时间
                // 3. 自动重载已修改的插件

                // Placeholder
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_registration() {
        let system = PluginSystem::new(PluginSystemConfig::default(), None);

        let metadata = PluginMetadata {
            plugin_id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "ACSA Team".to_string(),
            description: "A test plugin".to_string(),
            plugin_type: PluginType::Tool,
            dependencies: vec![],
            supported_protocols: vec![],
            resource_limits: ResourceLimits::default(),
            created_at: Utc::now(),
        };

        system.register_plugin(metadata).await.unwrap();

        let plugins = system.list_plugins().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "Test Plugin");
    }
}
