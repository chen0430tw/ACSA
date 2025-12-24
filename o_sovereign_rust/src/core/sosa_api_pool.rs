// SOSA API Pool Manager
// 基于SOSA算法的智能API池管理系统
// Intelligent API pool with automatic failover using SOSA (Spark Seed Self-Organizing Algorithm)

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// API提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiProviderType {
    OpenAI,
    Claude,
    Gemini,
    DeepSeek,
    LocalModel,
    Custom,
}

impl ApiProviderType {
    pub fn name(&self) -> &'static str {
        match self {
            ApiProviderType::OpenAI => "OpenAI",
            ApiProviderType::Claude => "Claude",
            ApiProviderType::Gemini => "Gemini",
            ApiProviderType::DeepSeek => "DeepSeek",
            ApiProviderType::LocalModel => "Local Model",
            ApiProviderType::Custom => "Custom",
        }
    }
}

/// API端点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub id: String,
    pub provider: ApiProviderType,
    pub api_key: Option<String>,
    pub base_url: String,
    pub model_name: String,
    /// 优先级 (0-100, 越高越优先)
    pub priority: u8,
    /// 是否启用
    pub enabled: bool,
    /// 本地模型特殊配置
    pub local_config: Option<LocalModelConfig>,
}

/// 本地模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelConfig {
    /// 后端类型 (ollama, llama.cpp, vllm, etc.)
    pub backend: String,
    /// HTTP端点
    pub endpoint: String,
    /// 上下文窗口
    pub context_window: u32,
}

/// API调用事件 (用于SOSA分析)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallEvent {
    pub endpoint_id: String,
    pub timestamp: DateTime<Utc>,
    pub latency_ms: u64,
    pub success: bool,
    pub error_type: Option<ApiErrorType>,
    pub tokens_used: Option<u32>,
}

/// API错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiErrorType {
    RateLimit,
    Timeout,
    NetworkError,
    InvalidKey,
    ServiceUnavailable,
    ModelOverload,
    Unknown,
}

impl ApiErrorType {
    pub fn severity(&self) -> u8 {
        match self {
            ApiErrorType::RateLimit => 3,
            ApiErrorType::Timeout => 2,
            ApiErrorType::NetworkError => 4,
            ApiErrorType::InvalidKey => 5,
            ApiErrorType::ServiceUnavailable => 3,
            ApiErrorType::ModelOverload => 3,
            ApiErrorType::Unknown => 2,
        }
    }
}

/// Binary-Twin特征表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryTwin {
    // Continuous features
    pub avg_energy: f64,      // 平均本地势能 [0,1]
    pub diversity: f64,       // 行为多样性
    pub size_norm: f64,       // 窗口大小归一化

    // Discrete binary features
    pub bit0: bool,           // 高能量行为存在 (>0.8)
    pub bit1: bool,           // 行为模式 >= 3
    pub bit2: bool,           // 窗口事件 >= 10
}

impl BinaryTwin {
    pub fn from_events(events: &[ApiCallEvent]) -> Self {
        if events.is_empty() {
            return Self::default();
        }

        let total = events.len();
        let success_count = events.iter().filter(|e| e.success).count();
        let avg_latency = events.iter().map(|e| e.latency_ms).sum::<u64>() as f64 / total as f64;

        // 计算能量: 成功率 - 延迟惩罚
        let success_rate = success_count as f64 / total as f64;
        let latency_penalty = (avg_latency / 5000.0).min(1.0); // 5秒为满惩罚
        let avg_energy = (success_rate - latency_penalty * 0.3).max(0.0).min(1.0);

        // 计算多样性: 错误类型的种类
        let error_types: std::collections::HashSet<_> = events
            .iter()
            .filter_map(|e| e.error_type)
            .collect();
        let diversity = error_types.len() as f64 / 7.0; // 7种错误类型

        let size_norm = (total as f64 / 100.0).min(1.0);

        // Binary features
        let bit0 = avg_energy > 0.8;
        let bit1 = error_types.len() >= 3;
        let bit2 = total >= 10;

        Self {
            avg_energy,
            diversity,
            size_norm,
            bit0,
            bit1,
            bit2,
        }
    }

    pub fn to_state_id(&self) -> u32 {
        let energy_bucket = (self.avg_energy * 10.0) as u32;
        let diversity_bucket = (self.diversity * 10.0) as u32;
        let binary_flags = (self.bit0 as u32) << 2 | (self.bit1 as u32) << 1 | (self.bit2 as u32);

        energy_bucket * 1000 + diversity_bucket * 10 + binary_flags
    }
}

impl Default for BinaryTwin {
    fn default() -> Self {
        Self {
            avg_energy: 0.5,
            diversity: 0.0,
            size_norm: 0.0,
            bit0: false,
            bit1: false,
            bit2: false,
        }
    }
}

/// 稀疏马尔可夫链
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseMarkov {
    num_states: usize,
    transitions: HashMap<(u32, u32), f64>, // (from_state, to_state) -> count
    state_counts: HashMap<u32, f64>,
}

impl SparseMarkov {
    pub fn new(num_states: usize) -> Self {
        Self {
            num_states,
            transitions: HashMap::new(),
            state_counts: HashMap::new(),
        }
    }

    pub fn add_transition(&mut self, from_state: u32, to_state: u32) {
        *self.transitions.entry((from_state, to_state)).or_insert(0.0) += 1.0;
        *self.state_counts.entry(from_state).or_insert(0.0) += 1.0;
    }

    pub fn get_probability(&self, from_state: u32, to_state: u32) -> f64 {
        let count = self.transitions.get(&(from_state, to_state)).copied().unwrap_or(0.0);
        let total = self.state_counts.get(&from_state).copied().unwrap_or(1.0);
        count / total
    }

    pub fn predict_next_state(&self, current_state: u32) -> Option<u32> {
        let mut best_state = None;
        let mut best_prob = 0.0;

        for (to_state, prob) in self.transitions.iter()
            .filter(|((from, _), _)| *from == current_state)
            .map(|((_, to), count)| {
                let total = self.state_counts.get(&current_state).copied().unwrap_or(1.0);
                (*to, count / total)
            })
        {
            if prob > best_prob {
                best_prob = prob;
                best_state = Some(to_state);
            }
        }

        best_state
    }
}

/// 吸引子 (固化的模式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attractor {
    pub pattern: Vec<u32>,
    pub strength: f64,
    pub occurrences: u32,
    pub last_seen: DateTime<Utc>,
}

/// SOSA核心引擎
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SosaCore {
    /// 时间窗口 (秒)
    dt_window: f64,
    /// 分组数量
    m_groups: usize,
    /// 探索权重
    exploration_weight: f64,
    /// 马尔可夫链
    markov: SparseMarkov,
    /// 事件缓冲窗口
    window_buffer: VecDeque<ApiCallEvent>,
    /// 吸引子集合
    attractors: HashMap<String, Attractor>,
    /// 上一个状态
    last_state: Option<u32>,
}

impl SosaCore {
    pub fn new(dt_window: f64, m_groups: usize, exploration_weight: f64) -> Self {
        Self {
            dt_window,
            m_groups,
            exploration_weight,
            markov: SparseMarkov::new(10000),
            window_buffer: VecDeque::new(),
            attractors: HashMap::new(),
            last_state: None,
        }
    }

    pub fn add_event(&mut self, event: ApiCallEvent) {
        // 清理过期事件
        let cutoff = Utc::now() - chrono::Duration::seconds(self.dt_window as i64);
        while let Some(front) = self.window_buffer.front() {
            if front.timestamp < cutoff {
                self.window_buffer.pop_front();
            } else {
                break;
            }
        }

        // 添加新事件
        self.window_buffer.push_back(event.clone());

        // 更新马尔可夫链
        if self.window_buffer.len() >= 2 {
            let events: Vec<_> = self.window_buffer.iter().cloned().collect();
            let twin = BinaryTwin::from_events(&events);
            let current_state = twin.to_state_id();

            if let Some(last) = self.last_state {
                self.markov.add_transition(last, current_state);
            }

            self.last_state = Some(current_state);

            // 检测吸引子
            self.detect_attractors();
        }
    }

    fn detect_attractors(&mut self) {
        if self.window_buffer.len() < 3 {
            return;
        }

        // 简化的模式检测: 连续3个相同状态
        let recent_states: Vec<u32> = self.window_buffer
            .iter()
            .rev()
            .take(3)
            .map(|e| {
                let events = vec![e.clone()];
                BinaryTwin::from_events(&events).to_state_id()
            })
            .collect();

        if recent_states.len() == 3 && recent_states[0] == recent_states[1] && recent_states[1] == recent_states[2] {
            let pattern_key = format!("{}", recent_states[0]);

            self.attractors
                .entry(pattern_key.clone())
                .and_modify(|a| {
                    a.strength = (a.strength + 0.1).min(1.0);
                    a.occurrences += 1;
                    a.last_seen = Utc::now();
                })
                .or_insert(Attractor {
                    pattern: vec![recent_states[0]],
                    strength: 0.3,
                    occurrences: 1,
                    last_seen: Utc::now(),
                });
        }
    }

    pub fn get_health_score(&self, endpoint_id: &str) -> f64 {
        let endpoint_events: Vec<_> = self.window_buffer
            .iter()
            .filter(|e| e.endpoint_id == endpoint_id)
            .cloned()
            .collect();

        if endpoint_events.is_empty() {
            return 0.5; // 中性分数
        }

        let twin = BinaryTwin::from_events(&endpoint_events);
        twin.avg_energy
    }

    pub fn recommend_endpoint(&self, available_endpoints: &[String]) -> Option<String> {
        if available_endpoints.is_empty() {
            return None;
        }

        let mut scores: Vec<_> = available_endpoints
            .iter()
            .map(|id| {
                let health = self.get_health_score(id);
                (id.clone(), health)
            })
            .collect();

        // 加入探索因子 (使用时间戳作为伪随机源)
        let explore_random = (Utc::now().timestamp_millis() % 100) as f64 / 100.0;
        if explore_random < self.exploration_weight {
            // 探索模式: 基于时间戳选择
            let idx = (Utc::now().timestamp_millis() as usize) % scores.len();
            return Some(scores[idx].0.clone());
        }

        // 利用模式: 选择最佳
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Some(scores[0].0.clone())
    }
}

/// SOSA API池管理器
pub struct SosaApiPool {
    /// API端点集合
    endpoints: Arc<RwLock<HashMap<String, ApiEndpoint>>>,
    /// SOSA核心引擎
    sosa: Arc<RwLock<SosaCore>>,
    /// 配置
    config: PoolConfig,
}

/// 池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// SOSA时间窗口 (秒)
    pub sosa_window_secs: f64,
    /// 探索权重
    pub exploration_weight: f64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试延迟 (ms)
    pub retry_delay_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            sosa_window_secs: 300.0,  // 5分钟窗口
            exploration_weight: 0.1,   // 10%探索
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl SosaApiPool {
    pub fn new(config: PoolConfig) -> Self {
        let sosa = SosaCore::new(config.sosa_window_secs, 10, config.exploration_weight);

        info!("🌊 SOSA API Pool initialized");
        info!("  Window: {:.1}s", config.sosa_window_secs);
        info!("  Exploration: {:.1}%", config.exploration_weight * 100.0);

        Self {
            endpoints: Arc::new(RwLock::new(HashMap::new())),
            sosa: Arc::new(RwLock::new(sosa)),
            config,
        }
    }

    /// 添加API端点
    pub async fn add_endpoint(&self, endpoint: ApiEndpoint) {
        let id = endpoint.id.clone();
        self.endpoints.write().await.insert(id.clone(), endpoint);
        info!("➕ Added API endpoint: {}", id);
    }

    /// 移除API端点
    pub async fn remove_endpoint(&self, endpoint_id: &str) {
        self.endpoints.write().await.remove(endpoint_id);
        info!("➖ Removed API endpoint: {}", endpoint_id);
    }

    /// 获取可用端点列表
    async fn get_available_endpoints(&self) -> Vec<String> {
        self.endpoints
            .read()
            .await
            .iter()
            .filter(|(_, ep)| ep.enabled)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 选择最佳端点
    pub async fn select_endpoint(&self) -> Result<ApiEndpoint> {
        let available = self.get_available_endpoints().await;

        if available.is_empty() {
            return Err(anyhow!("No available API endpoints"));
        }

        let sosa = self.sosa.read().await;
        let endpoint_id = sosa.recommend_endpoint(&available)
            .ok_or_else(|| anyhow!("SOSA failed to recommend endpoint"))?;

        let endpoints = self.endpoints.read().await;
        endpoints
            .get(&endpoint_id)
            .cloned()
            .ok_or_else(|| anyhow!("Endpoint not found: {}", endpoint_id))
    }

    /// 记录API调用结果
    pub async fn record_call(&self, event: ApiCallEvent) {
        let mut sosa = self.sosa.write().await;
        sosa.add_event(event);
    }

    /// 获取端点健康分数
    pub async fn get_endpoint_health(&self, endpoint_id: &str) -> f64 {
        let sosa = self.sosa.read().await;
        sosa.get_health_score(endpoint_id)
    }

    /// 列出所有端点及其状态
    pub async fn list_endpoints(&self) -> Vec<EndpointStatus> {
        let endpoints = self.endpoints.read().await;
        let sosa = self.sosa.read().await;

        let mut statuses = Vec::new();

        for (id, endpoint) in endpoints.iter() {
            let health = sosa.get_health_score(id);

            statuses.push(EndpointStatus {
                id: id.clone(),
                provider: endpoint.provider,
                model: endpoint.model_name.clone(),
                enabled: endpoint.enabled,
                priority: endpoint.priority,
                health_score: health,
            });
        }

        statuses.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap());
        statuses
    }

    /// 打印池状态
    pub async fn print_status(&self) {
        let statuses = self.list_endpoints().await;

        println!("\n┌─────────────────────────────────────────────────────┐");
        println!("│          SOSA API Pool Status                       │");
        println!("├─────────────────────────────────────────────────────┤");
        println!("│ ID              Provider    Model        Health     │");
        println!("├─────────────────────────────────────────────────────┤");

        for status in statuses {
            let health_bar = Self::render_health_bar(status.health_score);
            let enabled_icon = if status.enabled { "✓" } else { "✗" };

            println!(
                "│ {:1} {:14} {:10} {:12} {} │",
                enabled_icon,
                status.id.chars().take(14).collect::<String>(),
                status.provider.name().chars().take(10).collect::<String>(),
                status.model.chars().take(12).collect::<String>(),
                health_bar
            );
        }

        println!("└─────────────────────────────────────────────────────┘");
    }

    fn render_health_bar(score: f64) -> String {
        let filled = (score * 10.0) as usize;
        let empty = 10 - filled;

        let bar = "█".repeat(filled) + &"░".repeat(empty);

        let color = if score > 0.7 {
            "🟢"
        } else if score > 0.4 {
            "🟡"
        } else {
            "🔴"
        };

        format!("{} {}", color, bar)
    }
}

/// 端点状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStatus {
    pub id: String,
    pub provider: ApiProviderType,
    pub model: String,
    pub enabled: bool,
    pub priority: u8,
    pub health_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_twin_from_events() {
        let events = vec![
            ApiCallEvent {
                endpoint_id: "test".to_string(),
                timestamp: Utc::now(),
                latency_ms: 100,
                success: true,
                error_type: None,
                tokens_used: Some(50),
            },
            ApiCallEvent {
                endpoint_id: "test".to_string(),
                timestamp: Utc::now(),
                latency_ms: 200,
                success: true,
                error_type: None,
                tokens_used: Some(60),
            },
        ];

        let twin = BinaryTwin::from_events(&events);
        assert!(twin.avg_energy > 0.5);
    }

    #[tokio::test]
    async fn test_sosa_api_pool() {
        let pool = SosaApiPool::new(PoolConfig::default());

        let endpoint = ApiEndpoint {
            id: "claude-1".to_string(),
            provider: ApiProviderType::Claude,
            api_key: Some("test-key".to_string()),
            base_url: "https://api.anthropic.com".to_string(),
            model_name: "claude-3-opus".to_string(),
            priority: 80,
            enabled: true,
            local_config: None,
        };

        pool.add_endpoint(endpoint).await;

        let selected = pool.select_endpoint().await;
        assert!(selected.is_ok());
    }

    #[test]
    fn test_sparse_markov() {
        let mut markov = SparseMarkov::new(1000);

        markov.add_transition(0, 1);
        markov.add_transition(0, 1);
        markov.add_transition(0, 2);

        let prob_01 = markov.get_probability(0, 1);
        let prob_02 = markov.get_probability(0, 2);

        assert!(prob_01 > prob_02);
    }
}
