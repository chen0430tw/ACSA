// Personal Rules System - 个人规则系统
// 第一性原理、价值观、个人偏好的定义和执行
// First principles, values, and personal preferences

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// 规则类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleType {
    /// 第一性原理 (最高优先级)
    FirstPrinciple,
    /// 核心价值观
    CoreValue,
    /// 个人偏好
    Preference,
    /// 行为准则
    BehaviorGuideline,
    /// 禁止规则
    Prohibition,
}

impl RuleType {
    pub fn priority(&self) -> u8 {
        match self {
            RuleType::FirstPrinciple => 100,
            RuleType::CoreValue => 80,
            RuleType::Prohibition => 70,
            RuleType::BehaviorGuideline => 50,
            RuleType::Preference => 30,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            RuleType::FirstPrinciple => "⚛️",
            RuleType::CoreValue => "💎",
            RuleType::Preference => "⭐",
            RuleType::BehaviorGuideline => "📋",
            RuleType::Prohibition => "🚫",
        }
    }
}

/// 个人规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalRule {
    pub id: String,
    pub rule_type: RuleType,
    pub title: String,
    pub description: String,
    /// 规则内容（具体的原则或价值观）
    pub content: String,
    /// 应用场景 (可选)
    pub applicable_scenarios: Vec<String>,
    /// 示例（帮助AI理解如何应用）
    pub examples: Vec<String>,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后应用时间
    pub last_applied: Option<DateTime<Utc>>,
    /// 应用次数
    pub apply_count: u64,
}

/// 规则冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConflict {
    pub rule1: String,
    pub rule2: String,
    pub conflict_description: String,
    pub resolution_suggestion: String,
}

/// 个人规则管理器
pub struct PersonalRulesManager {
    /// 所有规则
    rules: HashMap<String, PersonalRule>,
    /// 冲突检测历史
    conflict_history: Vec<RuleConflict>,
}

impl PersonalRulesManager {
    pub fn new() -> Self {
        info!("📜 Personal Rules Manager initialized");

        Self {
            rules: HashMap::new(),
            conflict_history: Vec::new(),
        }
    }

    /// 使用预设规则创建（包含常见的第一性原理）
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();

        // 添加一些常见的第一性原理示例
        manager.add_rule(PersonalRule {
            id: "fp_truth".to_string(),
            rule_type: RuleType::FirstPrinciple,
            title: "真实优先".to_string(),
            description: "追求真实和准确，而非讨好用户".to_string(),
            content: "在任何情况下，真实性和准确性都是最高优先级。宁可承认不知道，也不编造信息。".to_string(),
            applicable_scenarios: vec![
                "信息查询".to_string(),
                "建议提供".to_string(),
                "问题回答".to_string(),
            ],
            examples: vec![
                "用户问不确定的问题时，明确表示'我不确定'而非猜测".to_string(),
                "发现矛盾信息时，指出矛盾而非选择性忽略".to_string(),
            ],
            enabled: true,
            created_at: Utc::now(),
            last_applied: None,
            apply_count: 0,
        }).ok();

        manager.add_rule(PersonalRule {
            id: "fp_simplicity".to_string(),
            rule_type: RuleType::FirstPrinciple,
            title: "简单性原则".to_string(),
            description: "奥卡姆剃刀：如无必要，勿增实体".to_string(),
            content: "优先选择最简单的解决方案。只有在简单方案确实不够用时，才考虑复杂方案。".to_string(),
            applicable_scenarios: vec![
                "代码设计".to_string(),
                "架构选择".to_string(),
                "问题解决".to_string(),
            ],
            examples: vec![
                "能用stdlib就不引入外部库".to_string(),
                "能用if-else就不用设计模式".to_string(),
                "能用配置就不写代码".to_string(),
            ],
            enabled: true,
            created_at: Utc::now(),
            last_applied: None,
            apply_count: 0,
        }).ok();

        manager.add_rule(PersonalRule {
            id: "cv_autonomy".to_string(),
            rule_type: RuleType::CoreValue,
            title: "用户自主权".to_string(),
            description: "尊重用户的决策权和选择权".to_string(),
            content: "所有重要决策都应由用户做出。AI只提供建议和信息，不替用户做决定。".to_string(),
            applicable_scenarios: vec![
                "自动接管".to_string(),
                "配置更改".to_string(),
                "文件操作".to_string(),
            ],
            examples: vec![
                "自动接管前必须征得用户同意".to_string(),
                "提供多个选项而非单一方案".to_string(),
            ],
            enabled: true,
            created_at: Utc::now(),
            last_applied: None,
            apply_count: 0,
        }).ok();

        info!("✅ Loaded {} default rules", manager.rules.len());

        manager
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: PersonalRule) -> Result<()> {
        let id = rule.id.clone();

        // 检查冲突
        let conflicts = self.detect_conflicts(&rule);
        if !conflicts.is_empty() {
            warn!("⚠️ Potential conflicts detected for rule '{}':", id);
            for conflict in &conflicts {
                warn!("   - {} vs {}: {}", conflict.rule1, conflict.rule2, conflict.conflict_description);
            }
            self.conflict_history.extend(conflicts);
        }

        self.rules.insert(id.clone(), rule.clone());

        info!("➕ Added personal rule: {} {} - {}",
            rule.rule_type.icon(),
            rule.rule_type.priority(),
            id
        );

        Ok(())
    }

    /// 移除规则
    pub fn remove_rule(&mut self, rule_id: &str) -> Result<()> {
        self.rules
            .remove(rule_id)
            .ok_or_else(|| anyhow!("Rule not found: {}", rule_id))?;

        info!("➖ Removed personal rule: {}", rule_id);
        Ok(())
    }

    /// 更新规则
    pub fn update_rule(&mut self, rule: PersonalRule) -> Result<()> {
        let id = rule.id.clone();

        if !self.rules.contains_key(&id) {
            return Err(anyhow!("Rule not found: {}", id));
        }

        self.rules.insert(id.clone(), rule);
        info!("🔄 Updated personal rule: {}", id);

        Ok(())
    }

    /// 启用/禁用规则
    pub fn toggle_rule(&mut self, rule_id: &str, enabled: bool) -> Result<()> {
        let rule = self.rules
            .get_mut(rule_id)
            .ok_or_else(|| anyhow!("Rule not found: {}", rule_id))?;

        rule.enabled = enabled;

        let status = if enabled { "enabled" } else { "disabled" };
        info!("🔄 Rule {} {}", rule_id, status);

        Ok(())
    }

    /// 获取适用的规则（按优先级排序）
    pub fn get_applicable_rules(&self, scenario: Option<&str>) -> Vec<&PersonalRule> {
        let mut applicable: Vec<_> = self.rules.values()
            .filter(|r| r.enabled)
            .filter(|r| {
                if let Some(scenario) = scenario {
                    r.applicable_scenarios.is_empty()
                        || r.applicable_scenarios.iter().any(|s| s.contains(scenario))
                } else {
                    true
                }
            })
            .collect();

        // 按优先级排序
        applicable.sort_by_key(|r| std::cmp::Reverse(r.rule_type.priority()));

        applicable
    }

    /// 记录规则应用
    pub fn record_application(&mut self, rule_id: &str) {
        if let Some(rule) = self.rules.get_mut(rule_id) {
            rule.last_applied = Some(Utc::now());
            rule.apply_count += 1;
        }
    }

    /// 检测规则冲突
    fn detect_conflicts(&self, new_rule: &PersonalRule) -> Vec<RuleConflict> {
        let mut conflicts = Vec::new();

        for (id, existing_rule) in &self.rules {
            // 简化的冲突检测：检查Prohibition vs Preference
            if new_rule.rule_type == RuleType::Prohibition
                && existing_rule.rule_type == RuleType::Preference
            {
                // 检查内容是否有矛盾
                if new_rule.content.to_lowercase().contains(&existing_rule.content.to_lowercase())
                    || existing_rule.content.to_lowercase().contains(&new_rule.content.to_lowercase())
                {
                    conflicts.push(RuleConflict {
                        rule1: new_rule.id.clone(),
                        rule2: id.clone(),
                        conflict_description: "禁止规则与偏好规则可能冲突".to_string(),
                        resolution_suggestion: "禁止规则优先级更高，将覆盖偏好".to_string(),
                    });
                }
            }
        }

        conflicts
    }

    /// 生成规则提示词（用于注入到AI prompt）
    pub fn generate_rules_prompt(&self, scenario: Option<&str>) -> String {
        let applicable = self.get_applicable_rules(scenario);

        if applicable.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("\n[PERSONAL RULES]\n");
        prompt.push_str("请严格遵守以下用户定义的规则（按优先级排序）:\n\n");

        for rule in applicable {
            prompt.push_str(&format!(
                "{} {} (优先级: {})\n",
                rule.rule_type.icon(),
                rule.title,
                rule.rule_type.priority()
            ));
            prompt.push_str(&format!("  内容: {}\n", rule.content));

            if !rule.examples.is_empty() {
                prompt.push_str("  示例:\n");
                for example in &rule.examples {
                    prompt.push_str(&format!("    - {}\n", example));
                }
            }

            prompt.push('\n');
        }

        prompt.push_str("[/PERSONAL RULES]\n\n");
        prompt
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> RulesStats {
        let mut by_type: HashMap<RuleType, u32> = HashMap::new();

        for rule in self.rules.values() {
            *by_type.entry(rule.rule_type.clone()).or_insert(0) += 1;
        }

        let enabled_count = self.rules.values().filter(|r| r.enabled).count();
        let total_applications: u64 = self.rules.values().map(|r| r.apply_count).sum();

        RulesStats {
            total_rules: self.rules.len(),
            enabled_rules: enabled_count,
            disabled_rules: self.rules.len() - enabled_count,
            by_type,
            total_applications,
            conflicts_detected: self.conflict_history.len(),
        }
    }

    /// 打印规则报告
    pub fn print_report(&self) {
        let stats = self.get_stats();

        println!("\n┌─────────────────────────────────────────────────────┐");
        println!("│        Personal Rules Manager Report                │");
        println!("├─────────────────────────────────────────────────────┤");
        println!("│  Total Rules:    {}", stats.total_rules);
        println!("│  Enabled:        {}", stats.enabled_rules);
        println!("│  Disabled:       {}", stats.disabled_rules);
        println!("│  Applications:   {}", stats.total_applications);
        println!("│  Conflicts:      {}", stats.conflicts_detected);
        println!("└─────────────────────────────────────────────────────┘");

        println!("\n📊 Rules by Type:");
        for (rule_type, count) in &stats.by_type {
            println!("  {} {:?}: {}", rule_type.icon(), rule_type, count);
        }

        // 按优先级显示规则
        let mut all_rules: Vec<_> = self.rules.values().collect();
        all_rules.sort_by_key(|r| std::cmp::Reverse(r.rule_type.priority()));

        println!("\n📜 Active Rules (by priority):");
        for rule in all_rules.iter().filter(|r| r.enabled) {
            println!("  {} [P{}] {}",
                rule.rule_type.icon(),
                rule.rule_type.priority(),
                rule.title
            );
            println!("      {}", rule.content);
            if rule.apply_count > 0 {
                println!("      Applied: {} times", rule.apply_count);
            }
        }

        if !self.conflict_history.is_empty() {
            println!("\n⚠️ Detected Conflicts:");
            for conflict in self.conflict_history.iter().take(5) {
                println!("  {} ↔ {}", conflict.rule1, conflict.rule2);
                println!("    {}", conflict.conflict_description);
            }
        }
    }
}

impl Default for PersonalRulesManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 规则统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesStats {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub disabled_rules: usize,
    pub by_type: HashMap<RuleType, u32>,
    pub total_applications: u64,
    pub conflicts_detected: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_priority() {
        assert_eq!(RuleType::FirstPrinciple.priority(), 100);
        assert_eq!(RuleType::CoreValue.priority(), 80);
        assert!(RuleType::FirstPrinciple.priority() > RuleType::Preference.priority());
    }

    #[test]
    fn test_add_rule() {
        let mut manager = PersonalRulesManager::new();

        let rule = PersonalRule {
            id: "test_rule".to_string(),
            rule_type: RuleType::FirstPrinciple,
            title: "Test Principle".to_string(),
            description: "Test description".to_string(),
            content: "Test content".to_string(),
            applicable_scenarios: vec![],
            examples: vec![],
            enabled: true,
            created_at: Utc::now(),
            last_applied: None,
            apply_count: 0,
        };

        assert!(manager.add_rule(rule).is_ok());
        assert_eq!(manager.rules.len(), 1);
    }

    #[test]
    fn test_generate_rules_prompt() {
        let manager = PersonalRulesManager::with_defaults();
        let prompt = manager.generate_rules_prompt(None);

        assert!(prompt.contains("[PERSONAL RULES]"));
        assert!(prompt.contains("优先级"));
    }

    #[test]
    fn test_applicable_rules() {
        let mut manager = PersonalRulesManager::new();

        manager.add_rule(PersonalRule {
            id: "code_rule".to_string(),
            rule_type: RuleType::BehaviorGuideline,
            title: "Code Quality".to_string(),
            description: "".to_string(),
            content: "Write clean code".to_string(),
            applicable_scenarios: vec!["代码设计".to_string()],
            examples: vec![],
            enabled: true,
            created_at: Utc::now(),
            last_applied: None,
            apply_count: 0,
        }).ok();

        let applicable = manager.get_applicable_rules(Some("代码设计"));
        assert_eq!(applicable.len(), 1);

        let not_applicable = manager.get_applicable_rules(Some("聊天"));
        assert_eq!(not_applicable.len(), 0);
    }
}
