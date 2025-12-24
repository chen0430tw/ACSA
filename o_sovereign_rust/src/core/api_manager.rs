// API Key Management and Cost Tracking Module
// API密钥管理与花费统计系统

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, warn};

/// API提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiProvider {
    OpenAI,
    Claude,
    Gemini,
    DeepSeek,
}

impl ApiProvider {
    pub fn name(&self) -> &'static str {
        match self {
            ApiProvider::OpenAI => "OpenAI",
            ApiProvider::Claude => "Claude",
            ApiProvider::Gemini => "Gemini",
            ApiProvider::DeepSeek => "DeepSeek",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            ApiProvider::OpenAI => "gpt-4",
            ApiProvider::Claude => "claude-3-opus-20240229",
            ApiProvider::Gemini => "gemini-pro",
            ApiProvider::DeepSeek => "deepseek-coder",
        }
    }

    pub fn pricing_info(&self) -> &'static str {
        match self {
            ApiProvider::OpenAI => "$30/1M tokens (GPT-4)",
            ApiProvider::Claude => "$15-75/1M tokens (Opus)",
            ApiProvider::Gemini => "$0.50/1M tokens (Pro)",
            ApiProvider::DeepSeek => "$0.14-0.28/1M tokens (Coder)",
        }
    }

    pub fn all() -> Vec<ApiProvider> {
        vec![
            ApiProvider::OpenAI,
            ApiProvider::Claude,
            ApiProvider::Gemini,
            ApiProvider::DeepSeek,
        ]
    }
}

/// API密钥配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub provider: ApiProvider,
    pub api_key: String,
    pub model: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl ApiKeyConfig {
    pub fn new(provider: ApiProvider, api_key: String) -> Self {
        Self {
            provider,
            api_key,
            model: None,
            enabled: true,
            created_at: Utc::now(),
            last_used: None,
            notes: None,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    /// 获取掩码后的密钥（用于显示）
    pub fn masked_key(&self) -> String {
        let key = &self.api_key;
        if key.len() <= 8 {
            return "*".repeat(key.len());
        }
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    }
}

/// 单次API调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallRecord {
    pub provider: ApiProvider,
    pub timestamp: DateTime<Utc>,
    pub tokens_used: u32,
    pub cost: f64,
    pub latency_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub agent_role: Option<String>,
}

impl ApiCallRecord {
    pub fn new_success(
        provider: ApiProvider,
        tokens: u32,
        cost: f64,
        latency_ms: u64,
        agent_role: Option<String>,
    ) -> Self {
        Self {
            provider,
            timestamp: Utc::now(),
            tokens_used: tokens,
            cost,
            latency_ms,
            success: true,
            error_message: None,
            agent_role,
        }
    }

    pub fn new_failure(
        provider: ApiProvider,
        latency_ms: u64,
        error: String,
        agent_role: Option<String>,
    ) -> Self {
        Self {
            provider,
            timestamp: Utc::now(),
            tokens_used: 0,
            cost: 0.0,
            latency_ms,
            success: false,
            error_message: Some(error),
            agent_role,
        }
    }
}

/// 提供商统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub provider: ApiProvider,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_latency_ms: f64,
    pub first_call: Option<DateTime<Utc>>,
    pub last_call: Option<DateTime<Utc>>,
}

impl Default for ProviderStats {
    fn default() -> Self {
        Self::new(ApiProvider::OpenAI)
    }
}

impl ProviderStats {
    pub fn new(provider: ApiProvider) -> Self {
        Self {
            provider,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            avg_latency_ms: 0.0,
            first_call: None,
            last_call: None,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.successful_calls as f64 / self.total_calls as f64) * 100.0
    }
}

/// API管理器
pub struct ApiManager {
    /// API密钥配置 (provider -> config)
    api_keys: HashMap<ApiProvider, ApiKeyConfig>,
    /// API调用历史记录
    call_history: Vec<ApiCallRecord>,
    /// 提供商统计缓存
    provider_stats: HashMap<ApiProvider, ProviderStats>,
    /// 数据持久化路径
    data_dir: PathBuf,
}

impl ApiManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            api_keys: HashMap::new(),
            call_history: Vec::new(),
            provider_stats: HashMap::new(),
            data_dir,
        }
    }

    /// 初始化 - 从磁盘加载数据
    pub async fn init(&mut self) -> Result<()> {
        info!("📊 Initializing API Manager...");

        // 确保数据目录存在
        fs::create_dir_all(&self.data_dir).await?;

        // 加载API密钥配置
        self.load_api_keys().await?;

        // 加载调用历史
        self.load_call_history().await?;

        // 重新计算统计数据
        self.recalculate_stats();

        info!("✅ API Manager initialized");
        info!("  API Keys: {}", self.api_keys.len());
        info!("  Call History: {} records", self.call_history.len());

        Ok(())
    }

    /// 添加API密钥
    pub async fn add_api_key(&mut self, config: ApiKeyConfig) -> Result<()> {
        let provider = config.provider;
        info!("➕ Adding API key for {}", provider.name());

        self.api_keys.insert(provider, config);
        self.save_api_keys().await?;

        Ok(())
    }

    /// 删除API密钥
    pub async fn remove_api_key(&mut self, provider: ApiProvider) -> Result<()> {
        info!("➖ Removing API key for {}", provider.name());

        if self.api_keys.remove(&provider).is_some() {
            self.save_api_keys().await?;
            Ok(())
        } else {
            Err(anyhow!("API key for {} not found", provider.name()))
        }
    }

    /// 更新API密钥
    pub async fn update_api_key(&mut self, config: ApiKeyConfig) -> Result<()> {
        let provider = config.provider;
        info!("🔄 Updating API key for {}", provider.name());

        if !self.api_keys.contains_key(&provider) {
            return Err(anyhow!("API key for {} not found", provider.name()));
        }

        self.api_keys.insert(provider, config);
        self.save_api_keys().await?;

        Ok(())
    }

    /// 获取API密钥
    pub fn get_api_key(&self, provider: ApiProvider) -> Option<&ApiKeyConfig> {
        self.api_keys.get(&provider)
    }

    /// 获取所有API密钥
    pub fn get_all_api_keys(&self) -> Vec<&ApiKeyConfig> {
        self.api_keys.values().collect()
    }

    /// 记录API调用
    pub async fn record_call(&mut self, record: ApiCallRecord) -> Result<()> {
        debug!("📝 Recording API call: {:?}", record.provider);

        // 更新last_used时间
        if let Some(config) = self.api_keys.get_mut(&record.provider) {
            config.last_used = Some(Utc::now());
        }

        // 添加到历史记录
        self.call_history.push(record.clone());

        // 更新统计数据
        let stats = self
            .provider_stats
            .entry(record.provider)
            .or_insert_with(|| ProviderStats::new(record.provider));

        stats.total_calls += 1;
        if record.success {
            stats.successful_calls += 1;
            stats.total_tokens += record.tokens_used as u64;
            stats.total_cost += record.cost;
        } else {
            stats.failed_calls += 1;
        }

        // 更新平均延迟
        let total_latency = stats.avg_latency_ms * (stats.total_calls - 1) as f64
            + record.latency_ms as f64;
        stats.avg_latency_ms = total_latency / stats.total_calls as f64;

        // 更新时间戳
        if stats.first_call.is_none() {
            stats.first_call = Some(record.timestamp);
        }
        stats.last_call = Some(record.timestamp);

        // 定期持久化（每10次调用）
        if self.call_history.len() % 10 == 0 {
            self.save_call_history().await?;
            self.save_api_keys().await?;
        }

        Ok(())
    }

    /// 获取提供商统计信息
    pub fn get_provider_stats(&self, provider: ApiProvider) -> Option<&ProviderStats> {
        self.provider_stats.get(&provider)
    }

    /// 获取所有统计信息
    pub fn get_all_stats(&self) -> Vec<&ProviderStats> {
        self.provider_stats.values().collect()
    }

    /// 获取总花费
    pub fn get_total_cost(&self) -> f64 {
        self.provider_stats.values().map(|s| s.total_cost).sum()
    }

    /// 获取总Token数
    pub fn get_total_tokens(&self) -> u64 {
        self.provider_stats.values().map(|s| s.total_tokens).sum()
    }

    /// 获取最近N条调用记录
    pub fn get_recent_calls(&self, limit: usize) -> Vec<&ApiCallRecord> {
        let start = self.call_history.len().saturating_sub(limit);
        self.call_history[start..].iter().rev().collect()
    }

    /// 清除历史记录（保留最近N条）
    pub async fn trim_history(&mut self, keep_recent: usize) -> Result<()> {
        warn!("🗑️ Trimming call history, keeping {} recent records", keep_recent);

        let current_len = self.call_history.len();
        if current_len > keep_recent {
            let start = current_len - keep_recent;
            self.call_history.drain(0..start);
            info!("  Removed {} old records", current_len - keep_recent);
        }

        self.save_call_history().await?;
        Ok(())
    }

    /// 导出统计报告（Markdown格式）
    pub fn export_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# O-Sovereign API Usage Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        report.push_str("## Overall Statistics\n\n");
        report.push_str(&format!("- **Total Cost**: ${:.4}\n", self.get_total_cost()));
        report.push_str(&format!("- **Total Tokens**: {}\n", self.get_total_tokens()));
        report.push_str(&format!("- **Total Calls**: {}\n\n", self.call_history.len()));

        report.push_str("## Provider Breakdown\n\n");
        report.push_str("| Provider | Calls | Success Rate | Tokens | Cost | Avg Latency |\n");
        report.push_str("|----------|-------|--------------|--------|------|-------------|\n");

        for provider in ApiProvider::all() {
            if let Some(stats) = self.provider_stats.get(&provider) {
                report.push_str(&format!(
                    "| {} | {} | {:.1}% | {} | ${:.4} | {}ms |\n",
                    provider.name(),
                    stats.total_calls,
                    stats.success_rate(),
                    stats.total_tokens,
                    stats.total_cost,
                    stats.avg_latency_ms as u64
                ));
            }
        }

        report.push_str("\n## Recent Calls (Last 10)\n\n");
        report.push_str("| Time | Provider | Success | Tokens | Cost | Latency |\n");
        report.push_str("|------|----------|---------|--------|------|----------|\n");

        for record in self.get_recent_calls(10) {
            report.push_str(&format!(
                "| {} | {} | {} | {} | ${:.4} | {}ms |\n",
                record.timestamp.format("%H:%M:%S"),
                record.provider.name(),
                if record.success { "✅" } else { "❌" },
                record.tokens_used,
                record.cost,
                record.latency_ms
            ));
        }

        report
    }

    /// 重新计算统计数据（从历史记录）
    fn recalculate_stats(&mut self) {
        debug!("🔄 Recalculating provider stats from history...");

        self.provider_stats.clear();

        for record in &self.call_history {
            let stats = self
                .provider_stats
                .entry(record.provider)
                .or_insert_with(|| ProviderStats::new(record.provider));

            stats.total_calls += 1;
            if record.success {
                stats.successful_calls += 1;
                stats.total_tokens += record.tokens_used as u64;
                stats.total_cost += record.cost;
            } else {
                stats.failed_calls += 1;
            }

            // 更新平均延迟
            let total_latency = stats.avg_latency_ms * (stats.total_calls - 1) as f64
                + record.latency_ms as f64;
            stats.avg_latency_ms = total_latency / stats.total_calls as f64;

            // 更新时间戳
            if stats.first_call.is_none() {
                stats.first_call = Some(record.timestamp);
            }
            stats.last_call = Some(record.timestamp);
        }
    }

    // === 持久化方法 ===

    fn api_keys_path(&self) -> PathBuf {
        self.data_dir.join("api_keys.json")
    }

    fn call_history_path(&self) -> PathBuf {
        self.data_dir.join("call_history.json")
    }

    async fn save_api_keys(&self) -> Result<()> {
        let path = self.api_keys_path();
        let json = serde_json::to_string_pretty(&self.api_keys)?;
        fs::write(path, json).await?;
        debug!("💾 Saved API keys");
        Ok(())
    }

    async fn load_api_keys(&mut self) -> Result<()> {
        let path = self.api_keys_path();
        if !path.exists() {
            debug!("No existing API keys file found");
            return Ok(());
        }

        let json = fs::read_to_string(path).await?;
        self.api_keys = serde_json::from_str(&json)?;
        info!("📂 Loaded {} API keys", self.api_keys.len());
        Ok(())
    }

    async fn save_call_history(&self) -> Result<()> {
        let path = self.call_history_path();
        let json = serde_json::to_string_pretty(&self.call_history)?;
        fs::write(path, json).await?;
        debug!("💾 Saved call history ({} records)", self.call_history.len());
        Ok(())
    }

    async fn load_call_history(&mut self) -> Result<()> {
        let path = self.call_history_path();
        if !path.exists() {
            debug!("No existing call history file found");
            return Ok(());
        }

        let json = fs::read_to_string(path).await?;
        self.call_history = serde_json::from_str(&json)?;
        info!("📂 Loaded {} call records", self.call_history.len());
        Ok(())
    }

    /// 手动保存所有数据
    pub async fn save_all(&self) -> Result<()> {
        info!("💾 Saving all API Manager data...");
        self.save_api_keys().await?;
        self.save_call_history().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_api_manager_init() {
        let dir = tempdir().unwrap();
        let mut manager = ApiManager::new(dir.path().to_path_buf());
        assert!(manager.init().await.is_ok());
    }

    #[tokio::test]
    async fn test_add_remove_api_key() {
        let dir = tempdir().unwrap();
        let mut manager = ApiManager::new(dir.path().to_path_buf());
        manager.init().await.unwrap();

        let config = ApiKeyConfig::new(ApiProvider::OpenAI, "sk-test123".to_string());
        assert!(manager.add_api_key(config).await.is_ok());

        assert!(manager.get_api_key(ApiProvider::OpenAI).is_some());

        assert!(manager.remove_api_key(ApiProvider::OpenAI).await.is_ok());
        assert!(manager.get_api_key(ApiProvider::OpenAI).is_none());
    }

    #[tokio::test]
    async fn test_record_call() {
        let dir = tempdir().unwrap();
        let mut manager = ApiManager::new(dir.path().to_path_buf());
        manager.init().await.unwrap();

        let record = ApiCallRecord::new_success(
            ApiProvider::OpenAI,
            1000,
            0.03,
            500,
            Some("MOSS".to_string()),
        );

        assert!(manager.record_call(record).await.is_ok());
        assert_eq!(manager.call_history.len(), 1);

        let stats = manager.get_provider_stats(ApiProvider::OpenAI).unwrap();
        assert_eq!(stats.total_calls, 1);
        assert_eq!(stats.successful_calls, 1);
        assert_eq!(stats.total_tokens, 1000);
    }

    #[tokio::test]
    async fn test_masked_key() {
        let config = ApiKeyConfig::new(ApiProvider::OpenAI, "sk-1234567890abcdef".to_string());
        assert_eq!(config.masked_key(), "sk-1...cdef");
    }

    #[test]
    fn test_provider_stats() {
        let mut stats = ProviderStats::new(ApiProvider::OpenAI);
        stats.total_calls = 100;
        stats.successful_calls = 95;
        assert_eq!(stats.success_rate(), 95.0);
    }

    #[test]
    fn test_export_report() {
        let dir = tempdir().unwrap();
        let manager = ApiManager::new(dir.path().to_path_buf());
        let report = manager.export_report();
        assert!(report.contains("# O-Sovereign API Usage Report"));
        assert!(report.contains("Overall Statistics"));
    }
}
