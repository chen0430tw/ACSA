// Agent Extension System - 智能体扩充系统
// 支持添加自定义Agent，但警告边际效用递减和token消耗
// Custom agent extension with diminishing returns warning

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

use super::protocol::AgentWeights;

/// 自定义Agent定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgent {
    /// Agent名称
    pub name: String,
    /// Agent描述
    pub description: String,
    /// API配置
    pub api_config: AgentApiConfig,
    /// 性能指标
    pub metrics: AgentMetrics,
    /// 是否启用
    pub enabled: bool,
}

/// Agent API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentApiConfig {
    /// Provider类型 (openai/claude/gemini/deepseek/local)
    pub provider: String,
    /// 模型名称
    pub model_name: String,
    /// API密钥 (可选)
    pub api_key: Option<String>,
    /// 自定义端点 (可选)
    pub custom_endpoint: Option<String>,
    /// 温度参数
    pub temperature: f64,
    /// 最大tokens
    pub max_tokens: u32,
}

/// Agent性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// 平均响应时间 (ms)
    pub avg_response_time_ms: u64,
    /// token消耗率 (tokens/request)
    pub tokens_per_request: f64,
    /// 成功率
    pub success_rate: f64,
    /// 总调用次数
    pub total_calls: u64,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            avg_response_time_ms: 0,
            tokens_per_request: 0.0,
            success_rate: 1.0,
            total_calls: 0,
        }
    }
}

/// 边际效用计算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiminishingReturns {
    /// 当前Agent数量
    pub agent_count: usize,
    /// 边际效用分数 (0-1, 越低越不推荐)
    pub marginal_utility: f64,
    /// 预估额外token消耗
    pub estimated_token_cost: u64,
    /// 推荐建议
    pub recommendation: Recommendation,
}

/// 推荐等级
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recommendation {
    /// 强烈推荐 (1-2个Agent)
    HighlyRecommended,
    /// 推荐 (3-4个Agent)
    Recommended,
    /// 谨慎 (5-6个Agent)
    Caution,
    /// 不推荐 (7-8个Agent)
    NotRecommended,
    /// 强烈不推荐 (9+个Agent)
    StronglyNotRecommended,
}

impl Recommendation {
    pub fn icon(&self) -> &'static str {
        match self {
            Recommendation::HighlyRecommended => "✅",
            Recommendation::Recommended => "👍",
            Recommendation::Caution => "⚠️",
            Recommendation::NotRecommended => "⛔",
            Recommendation::StronglyNotRecommended => "🚫",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Recommendation::HighlyRecommended => "强烈推荐：系统效率最优",
            Recommendation::Recommended => "推荐：边际效用仍然较高",
            Recommendation::Caution => "谨慎：开始出现边际效用递减",
            Recommendation::NotRecommended => "不推荐：边际效用显著递减，token消耗增加",
            Recommendation::StronglyNotRecommended => "强烈不推荐：严重低效运行，大量浪费token",
        }
    }
}

/// Agent扩展管理器
pub struct AgentExtensionManager {
    /// 核心Agent (MOSS/L6/Ultron/Omega)
    core_agents: Vec<String>,
    /// 自定义Agent
    custom_agents: HashMap<String, CustomAgent>,
    /// 历史调用数据
    call_history: Vec<AgentCallRecord>,
}

/// Agent调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCallRecord {
    pub agent_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tokens_used: u32,
    pub response_time_ms: u64,
    pub success: bool,
}

impl AgentExtensionManager {
    pub fn new() -> Self {
        info!("🤖 Agent Extension Manager initialized");
        info!("  Core agents: 4 (MOSS, L6, Ultron, Omega)");

        Self {
            core_agents: vec![
                "MOSS".to_string(),
                "L6".to_string(),
                "Ultron".to_string(),
                "Omega".to_string(),
            ],
            custom_agents: HashMap::new(),
            call_history: Vec::new(),
        }
    }

    /// 添加自定义Agent（带警告）
    pub fn add_custom_agent(&mut self, agent: CustomAgent) -> Result<DiminishingReturns> {
        let current_count = self.core_agents.len() + self.custom_agents.len();

        // 计算边际效用
        let diminishing = self.calculate_diminishing_returns(current_count + 1);

        // 显示警告
        self.show_warning(&diminishing);

        // 如果是StronglyNotRecommended，需要用户明确确认
        if diminishing.recommendation == Recommendation::StronglyNotRecommended {
            warn!("🚫 Adding agent '{}' is strongly not recommended!", agent.name);
            warn!("   Current agents: {}", current_count);
            warn!("   Marginal utility: {:.1}%", diminishing.marginal_utility * 100.0);
            warn!("   Estimated token cost: +{} tokens/request", diminishing.estimated_token_cost);
        }

        let name = agent.name.clone();
        self.custom_agents.insert(name.clone(), agent);

        info!("➕ Added custom agent: {}", name);
        info!("   Total agents: {}", self.total_agent_count());

        Ok(diminishing)
    }

    /// 计算边际效用递减
    fn calculate_diminishing_returns(&self, new_count: usize) -> DiminishingReturns {
        // 边际效用公式：U(n) = 1 - (n-4)²/100
        // 4个Agent时效用最高，之后递减
        let n = new_count as f64;
        let optimal = 4.0; // 核心4个Agent

        let marginal_utility = if n <= optimal {
            1.0 // 完美效用
        } else {
            let decay = (n - optimal).powi(2) / 100.0;
            (1.0 - decay).max(0.0)
        };

        // 估算额外token消耗：每多一个Agent约增加20-30%开销
        let base_tokens = 1000u64; // 基准token数
        let overhead_per_agent = (n - optimal).max(0.0) * 250.0;
        let estimated_token_cost = overhead_per_agent as u64;

        let recommendation = match new_count {
            1..=4 => Recommendation::HighlyRecommended,
            5..=6 => Recommendation::Recommended,
            7..=8 => Recommendation::Caution,
            9..=10 => Recommendation::NotRecommended,
            _ => Recommendation::StronglyNotRecommended,
        };

        DiminishingReturns {
            agent_count: new_count,
            marginal_utility,
            estimated_token_cost,
            recommendation,
        }
    }

    /// 显示警告信息
    fn show_warning(&self, diminishing: &DiminishingReturns) {
        let icon = diminishing.recommendation.icon();
        let msg = diminishing.recommendation.message();

        println!("\n┌────────────────────────────────────────────────────┐");
        println!("│  {} Agent扩充建议                              │", icon);
        println!("├────────────────────────────────────────────────────┤");
        println!("│  当前Agent数量: {}", diminishing.agent_count);
        println!("│  边际效用:      {:.1}%", diminishing.marginal_utility * 100.0);
        println!("│  额外Token消耗: +{} tokens/request", diminishing.estimated_token_cost);
        println!("├────────────────────────────────────────────────────┤");
        println!("│  {}", msg);
        println!("└────────────────────────────────────────────────────┘\n");

        if diminishing.recommendation == Recommendation::Caution
            || diminishing.recommendation == Recommendation::NotRecommended {
            warn!("⚠️ 边际效用递减警告:");
            warn!("   - 更多Agent可能导致决策冲突");
            warn!("   - Token消耗显著增加");
            warn!("   - 响应时间变长");
            warn!("   建议: 优化现有Agent权重分配而非添加新Agent");
        }
    }

    /// 移除自定义Agent
    pub fn remove_custom_agent(&mut self, name: &str) -> Result<()> {
        self.custom_agents
            .remove(name)
            .ok_or_else(|| anyhow!("Custom agent not found: {}", name))?;

        info!("➖ Removed custom agent: {}", name);
        info!("   Total agents: {}", self.total_agent_count());

        Ok(())
    }

    /// 获取Agent总数
    pub fn total_agent_count(&self) -> usize {
        self.core_agents.len() + self.custom_agents.len()
    }

    /// 列出所有Agent
    pub fn list_agents(&self) -> AgentList {
        let core: Vec<_> = self.core_agents.iter()
            .map(|name| AgentInfo {
                name: name.clone(),
                agent_type: AgentType::Core,
                enabled: true,
                metrics: None,
            })
            .collect();

        let custom: Vec<_> = self.custom_agents.values()
            .map(|agent| AgentInfo {
                name: agent.name.clone(),
                agent_type: AgentType::Custom,
                enabled: agent.enabled,
                metrics: Some(agent.metrics.clone()),
            })
            .collect();

        AgentList {
            core_agents: core,
            custom_agents: custom,
            total_count: self.total_agent_count(),
            diminishing_returns: self.calculate_diminishing_returns(self.total_agent_count()),
        }
    }

    /// 记录Agent调用
    pub fn record_call(&mut self, record: AgentCallRecord) {
        // 更新Agent指标
        if let Some(agent) = self.custom_agents.get_mut(&record.agent_name) {
            let metrics = &mut agent.metrics;

            // 更新平均响应时间
            let total_time = metrics.avg_response_time_ms * metrics.total_calls;
            metrics.total_calls += 1;
            metrics.avg_response_time_ms = (total_time + record.response_time_ms) / metrics.total_calls;

            // 更新token消耗率
            let total_tokens = metrics.tokens_per_request * (metrics.total_calls - 1) as f64;
            metrics.tokens_per_request = (total_tokens + record.tokens_used as f64) / metrics.total_calls as f64;

            // 更新成功率
            let success_count = if record.success { 1.0 } else { 0.0 };
            metrics.success_rate = (metrics.success_rate * (metrics.total_calls - 1) as f64 + success_count)
                / metrics.total_calls as f64;
        }

        self.call_history.push(record);

        // 只保留最近1000条记录
        if self.call_history.len() > 1000 {
            self.call_history.drain(0..100);
        }
    }

    /// 获取Agent性能统计
    pub fn get_performance_stats(&self) -> HashMap<String, AgentMetrics> {
        let mut stats = HashMap::new();

        for (name, agent) in &self.custom_agents {
            stats.insert(name.clone(), agent.metrics.clone());
        }

        stats
    }

    /// 打印Agent扩充报告
    pub fn print_report(&self) {
        let list = self.list_agents();

        println!("\n┌─────────────────────────────────────────────────────┐");
        println!("│        Agent Extension Manager Report               │");
        println!("├─────────────────────────────────────────────────────┤");
        println!("│  Core Agents:    {}", list.core_agents.len());
        println!("│  Custom Agents:  {}", list.custom_agents.len());
        println!("│  Total:          {}", list.total_count);
        println!("├─────────────────────────────────────────────────────┤");
        println!("│  边际效用:       {:.1}%", list.diminishing_returns.marginal_utility * 100.0);
        println!("│  Token开销:      +{} tokens", list.diminishing_returns.estimated_token_cost);
        println!("│  推荐等级:       {} {}",
            list.diminishing_returns.recommendation.icon(),
            list.diminishing_returns.recommendation.message()
        );
        println!("└─────────────────────────────────────────────────────┘");

        if !list.custom_agents.is_empty() {
            println!("\n🤖 Custom Agents:");
            for agent in list.custom_agents {
                let status = if agent.enabled { "✅" } else { "❌" };
                println!("  {} {}", status, agent.name);

                if let Some(metrics) = agent.metrics {
                    println!("     Calls: {}, Success: {:.1}%, Tokens: {:.0}/req",
                        metrics.total_calls,
                        metrics.success_rate * 100.0,
                        metrics.tokens_per_request
                    );
                }
            }
        }
    }
}

impl Default for AgentExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub agent_type: AgentType,
    pub enabled: bool,
    pub metrics: Option<AgentMetrics>,
}

/// Agent类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Core,
    Custom,
}

/// Agent列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentList {
    pub core_agents: Vec<AgentInfo>,
    pub custom_agents: Vec<AgentInfo>,
    pub total_count: usize,
    pub diminishing_returns: DiminishingReturns,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diminishing_returns() {
        let manager = AgentExtensionManager::new();

        // 4个Agent - 最优
        let dr4 = manager.calculate_diminishing_returns(4);
        assert_eq!(dr4.marginal_utility, 1.0);
        assert_eq!(dr4.recommendation, Recommendation::HighlyRecommended);

        // 8个Agent - 谨慎
        let dr8 = manager.calculate_diminishing_returns(8);
        assert!(dr8.marginal_utility < 1.0);
        assert_eq!(dr8.recommendation, Recommendation::Caution);

        // 12个Agent - 强烈不推荐
        let dr12 = manager.calculate_diminishing_returns(12);
        assert!(dr12.marginal_utility < 0.5);
        assert_eq!(dr12.recommendation, Recommendation::StronglyNotRecommended);
    }

    #[test]
    fn test_add_custom_agent() {
        let mut manager = AgentExtensionManager::new();

        let agent = CustomAgent {
            name: "TestAgent".to_string(),
            description: "Test agent".to_string(),
            api_config: AgentApiConfig {
                provider: "openai".to_string(),
                model_name: "gpt-4".to_string(),
                api_key: None,
                custom_endpoint: None,
                temperature: 0.7,
                max_tokens: 2000,
            },
            metrics: AgentMetrics::default(),
            enabled: true,
        };

        let result = manager.add_custom_agent(agent);
        assert!(result.is_ok());
        assert_eq!(manager.total_agent_count(), 5);
    }
}
