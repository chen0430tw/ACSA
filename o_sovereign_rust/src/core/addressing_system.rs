// AI Addressing System - AI称呼系统
// 目标：个性化用户称呼管理
//
// 核心功能：
// 1. Protocol动态称呼
// 2. 用户自定义称呼
// 3. 多语言支持
// 4. 称呼历史学习

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::protocol::Protocol;
use super::i18n::Language;

/// 称呼模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddressingMode {
    Custom(String),      // 用户自定义
    ProtocolBased,       // 根据Protocol动态
    Fixed(String),       // 固定称呼
}

/// 称呼风格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddressingStyle {
    Formal,    // 正式：统御者、主权者
    Intimate,  // 亲密：主人、Boss
    Neutral,   // 中性：主理人、Master
}

/// 称呼系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressingConfig {
    pub mode: AddressingMode,
    pub style: AddressingStyle,
    pub language: Language,
    pub enable_protocol_switch: bool,  // 启用Protocol动态切换
}

impl Default for AddressingConfig {
    fn default() -> Self {
        Self {
            mode: AddressingMode::ProtocolBased,
            style: AddressingStyle::Intimate,  // 默认亲密风格
            language: Language::ChineseSimplified,
            enable_protocol_switch: true,      // 默认启用动态称呼
        }
    }
}

/// AI称呼管理器
pub struct AddressingSystem {
    config: AddressingConfig,
    protocol_addressings: HashMap<Protocol, (String, String)>,  // (中文, 英文)
    custom_addressing: Option<String>,
    usage_history: Vec<AddressingEvent>,
}

/// 称呼事件（用于学习）
#[derive(Debug, Clone)]
pub struct AddressingEvent {
    pub protocol: Protocol,
    pub addressing_used: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AddressingSystem {
    pub fn new(config: AddressingConfig) -> Self {
        info!("👤 Addressing System initialized");
        info!("   - Mode: {:?}", config.mode);
        info!("   - Style: {:?}", config.style);
        info!("   - Language: {:?}", config.language);

        let mut system = Self {
            config,
            protocol_addressings: HashMap::new(),
            custom_addressing: None,
            usage_history: Vec::new(),
        };

        system.initialize_protocol_addressings();
        system
    }

    /// 初始化Protocol称呼映射
    fn initialize_protocol_addressings(&mut self) {
        match self.config.style {
            AddressingStyle::Intimate => {
                // 亲密风格：主人/Boss
                self.protocol_addressings.insert(Protocol::Architect, ("主人".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Reviewer2, ("老板".to_string(), "Boss".to_string()));
                self.protocol_addressings.insert(Protocol::Aegis, ("Boss".to_string(), "Boss".to_string()));
                self.protocol_addressings.insert(Protocol::Predator, ("老大".to_string(), "Boss".to_string()));
                self.protocol_addressings.insert(Protocol::McKinsey, ("老板".to_string(), "Chief".to_string()));
                self.protocol_addressings.insert(Protocol::Lsd, ("主人".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Ghost, ("主人".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Sunday, ("主人".to_string(), "Boss".to_string()));
            }
            AddressingStyle::Formal => {
                // 正式风格：统御者/Sovereign
                self.protocol_addressings.insert(Protocol::Architect, ("主理人".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Reviewer2, ("教授".to_string(), "Professor".to_string()));
                self.protocol_addressings.insert(Protocol::Aegis, ("委托人".to_string(), "Client".to_string()));
                self.protocol_addressings.insert(Protocol::Predator, ("首席".to_string(), "Chief".to_string()));
                self.protocol_addressings.insert(Protocol::McKinsey, ("首席".to_string(), "Chief".to_string()));
                self.protocol_addressings.insert(Protocol::Lsd, ("创造者".to_string(), "Creator".to_string()));
                self.protocol_addressings.insert(Protocol::Ghost, ("主权者".to_string(), "Sovereign".to_string()));
                self.protocol_addressings.insert(Protocol::Sunday, ("先生".to_string(), "Sir".to_string()));
            }
            AddressingStyle::Neutral => {
                // 中性风格：主理人/Master
                self.protocol_addressings.insert(Protocol::Architect, ("主理人".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Reviewer2, ("主理人".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Aegis, ("主理人".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Predator, ("Chief".to_string(), "Chief".to_string()));
                self.protocol_addressings.insert(Protocol::McKinsey, ("Chief".to_string(), "Chief".to_string()));
                self.protocol_addressings.insert(Protocol::Lsd, ("Master".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Ghost, ("Master".to_string(), "Master".to_string()));
                self.protocol_addressings.insert(Protocol::Sunday, ("Master".to_string(), "Master".to_string()));
            }
        }
    }

    /// 获取当前称呼
    pub fn get_addressing(&self, protocol: &Protocol) -> String {
        match &self.config.mode {
            AddressingMode::Custom(custom) => custom.clone(),
            AddressingMode::Fixed(fixed) => fixed.clone(),
            AddressingMode::ProtocolBased => {
                if self.config.enable_protocol_switch {
                    self.get_protocol_addressing(protocol)
                } else {
                    self.get_default_addressing()
                }
            }
        }
    }

    /// 获取Protocol对应的称呼
    fn get_protocol_addressing(&self, protocol: &Protocol) -> String {
        let (zh, en) = self.protocol_addressings
            .get(protocol)
            .cloned()
            .unwrap_or(("主人".to_string(), "Master".to_string()));

        match self.config.language {
            Language::ChineseSimplified => zh,
            Language::EnglishUS => en,
            _ => en,
        }
    }

    /// 获取默认称呼
    fn get_default_addressing(&self) -> String {
        match self.config.language {
            Language::ChineseSimplified => "主人".to_string(),
            Language::EnglishUS => "Master".to_string(),
            _ => "Master".to_string(),
        }
    }

    /// 设置自定义称呼
    pub fn set_custom_addressing(&mut self, addressing: String) {
        info!("📝 Custom addressing set: {}", addressing);
        self.custom_addressing = Some(addressing.clone());
        self.config.mode = AddressingMode::Custom(addressing);
    }

    /// 记录称呼使用
    pub fn record_usage(&mut self, protocol: Protocol, addressing: String) {
        self.usage_history.push(AddressingEvent {
            protocol,
            addressing_used: addressing,
            timestamp: chrono::Utc::now(),
        });

        // 保持历史上限
        if self.usage_history.len() > 1000 {
            self.usage_history.remove(0);
        }
    }

    /// 获取称呼统计
    pub fn get_addressing_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for event in &self.usage_history {
            *stats.entry(event.addressing_used.clone()).or_insert(0) += 1;
        }
        stats
    }

    /// 格式化问候语
    pub fn format_greeting(&self, protocol: &Protocol) -> String {
        let addressing = self.get_addressing(protocol);
        
        match protocol {
            Protocol::Architect => format!("{}，准备好编码了吗？", addressing),
            Protocol::Reviewer2 => format!("{}，今天要审阅什么？", addressing),
            Protocol::Aegis => format!("{}，有什么风险需要防范？", addressing),
            Protocol::Predator => format!("{}，让我们开始狩猎吧", addressing),
            Protocol::McKinsey => format!("{}，战略会议开始", addressing),
            Protocol::Lsd => format!("{}，让创意飞翔", addressing),
            Protocol::Ghost => format!("{}，行动开始", addressing),
            Protocol::Sunday => format!("{}，今天想做什么？", addressing),
            Protocol::Custom(_) => format!("{}，随时为您服务", addressing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addressing_system() {
        let config = AddressingConfig::default();
        let system = AddressingSystem::new(config);

        let addressing = system.get_addressing(&Protocol::Architect);
        assert_eq!(addressing, "主人"); // 亲密风格 + 中文
    }

    #[test]
    fn test_custom_addressing() {
        let config = AddressingConfig::default();
        let mut system = AddressingSystem::new(config);

        system.set_custom_addressing("老大".to_string());
        let addressing = system.get_addressing(&Protocol::Architect);
        assert_eq!(addressing, "老大");
    }

    #[test]
    fn test_greeting_format() {
        let config = AddressingConfig::default();
        let system = AddressingSystem::new(config);

        let greeting = system.format_greeting(&Protocol::Architect);
        assert!(greeting.contains("主人"));
    }
}
