// ACSA Protocol System - 8 Core Styles
// ACSA风格协议系统 - 8个核心模式
//
// 核心设计: 液态软件 (Liquid Software)
// 根据用户输入自动切换势场参数和Agent权重

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ACSA核心协议（风格）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// 💻 编程/黑客模式 - 沉默的造物主
    Architect,
    /// 🔬 科研/学术模式 - 冷酷的审判官
    Reviewer2,
    /// ⚖️ 法律/合规模式 - 绝对防御盾
    Aegis,
    /// 💰 金融/量化模式 - 嗜血的掠食者
    Predator,
    /// 👔 商管/咨询模式 - 优化的暴君
    McKinsey,
    /// 🎨 设计/创意模式 - 理性的疯子
    Lsd,
    /// 🕶️ 影子/灰区模式 - 隐形的操盘手
    Ghost,
    /// ☕ 日常/娱乐模式 - 高维度的懒人管家
    Sunday,
    /// 🔧 自定义协议 - 用户自定义风格
    Custom(String),
}

impl Protocol {
    /// 获取协议名称
    pub fn name(&self) -> String {
        match self {
            Protocol::Architect => "ARCHITECT".to_string(),
            Protocol::Reviewer2 => "REVIEWER_2".to_string(),
            Protocol::Aegis => "AEGIS".to_string(),
            Protocol::Predator => "PREDATOR".to_string(),
            Protocol::McKinsey => "MCKINSEY".to_string(),
            Protocol::Lsd => "LSD".to_string(),
            Protocol::Ghost => "GHOST".to_string(),
            Protocol::Sunday => "SUNDAY".to_string(),
            Protocol::Custom(name) => format!("CUSTOM_{}", name.to_uppercase()),
        }
    }

    /// 获取协议显示名称（带emoji）
    pub fn display_name(&self) -> String {
        match self {
            Protocol::Architect => "💻 编程/黑客模式".to_string(),
            Protocol::Reviewer2 => "🔬 科研/学术模式".to_string(),
            Protocol::Aegis => "⚖️ 法律/合规模式".to_string(),
            Protocol::Predator => "💰 金融/量化模式".to_string(),
            Protocol::McKinsey => "👔 商管/咨询模式".to_string(),
            Protocol::Lsd => "🎨 设计/创意模式".to_string(),
            Protocol::Ghost => "🕶️ 影子/灰区模式".to_string(),
            Protocol::Sunday => "☕ 日常/娱乐模式".to_string(),
            Protocol::Custom(name) => format!("🔧 自定义: {}", name),
        }
    }

    /// 获取协议标语
    pub fn tagline(&self) -> String {
        match self {
            Protocol::Architect => "沉默的造物主".to_string(),
            Protocol::Reviewer2 => "冷酷的审判官".to_string(),
            Protocol::Aegis => "绝对防御盾".to_string(),
            Protocol::Predator => "嗜血的掠食者".to_string(),
            Protocol::McKinsey => "优化的暴君".to_string(),
            Protocol::Lsd => "理性的疯子".to_string(),
            Protocol::Ghost => "隐形的操盘手".to_string(),
            Protocol::Sunday => "高维度的懒人管家".to_string(),
            Protocol::Custom(_) => "用户自定义".to_string(),
        }
    }

    /// 获取TUI颜色主题
    pub fn tui_color(&self) -> &'static str {
        match self {
            Protocol::Architect => "#00FF41",   // 矩阵绿
            Protocol::Reviewer2 => "#1E3A8A",   // 深海蓝
            Protocol::Aegis => "#F59E0B",       // 琥珀黄
            Protocol::Predator => "#DC2626",    // 熔岩红
            Protocol::McKinsey => "#6B7280",    // 冷钢灰
            Protocol::Lsd => "#A855F7",         // 霓虹紫
            Protocol::Ghost => "#000000",       // 全黑
            Protocol::Sunday => "#FCD34D",      // 日落金
            Protocol::Custom(_) => "#FFFFFF",   // 白色
        }
    }

    /// 获取协议哲学
    pub fn philosophy(&self) -> String {
        match self {
            Protocol::Architect => "实用主义至上。只看Code能不能跑，Bug有没有修。".to_string(),
            Protocol::Reviewer2 => "怀疑一切。默认输入的论文是垃圾，除非数据能证明它是金子。".to_string(),
            Protocol::Aegis => "不求有功，但求无过。不仅要赢，还要赢得无懈可擊。".to_string(),
            Protocol::Predator => "天下武功，唯快不破。在泡沫破裂前1毫秒离场。".to_string(),
            Protocol::McKinsey => "一切皆可量化，一切皆可优化。人是资源，不是目的。".to_string(),
            Protocol::Lsd => "打破范式。在逻辑的边缘试探艺术。".to_string(),
            Protocol::Ghost => "存在即合理。目标达成，痕迹全无。".to_string(),
            Protocol::Sunday => "人生苦短，多巴胺管理是第一要务。".to_string(),
            Protocol::Custom(_) => "用户自定义风格".to_string(),
        }
    }

    /// 从关键词自动检测协议
    pub fn detect_from_input(input: &str) -> Option<Self> {
        let input_lower = input.to_lowercase();

        // 编程/黑客关键词
        if input_lower.contains("代码") || input_lower.contains("code")
            || input_lower.contains("编程") || input_lower.contains("bug")
            || input_lower.contains("debug") || input_lower.contains("function")
            || input_lower.contains("爬虫") || input_lower.contains("api")
        {
            return Some(Protocol::Architect);
        }

        // 法律/合规关键词
        if input_lower.contains("合同") || input_lower.contains("法律")
            || input_lower.contains("合规") || input_lower.contains("风险")
            || input_lower.contains("contract") || input_lower.contains("legal")
        {
            return Some(Protocol::Aegis);
        }

        // 金融/量化关键词
        if input_lower.contains("股价") || input_lower.contains("交易")
            || input_lower.contains("投资") || input_lower.contains("金融")
            || input_lower.contains("stock") || input_lower.contains("trading")
        {
            return Some(Protocol::Predator);
        }

        // 科研/学术关键词
        if input_lower.contains("论文") || input_lower.contains("研究")
            || input_lower.contains("paper") || input_lower.contains("research")
            || input_lower.contains("学术") || input_lower.contains("引用")
        {
            return Some(Protocol::Reviewer2);
        }

        // 商管/咨询关键词
        if input_lower.contains("ppt") || input_lower.contains("战略")
            || input_lower.contains("优化") || input_lower.contains("流程")
            || input_lower.contains("strategy") || input_lower.contains("management")
        {
            return Some(Protocol::McKinsey);
        }

        // 设计/创意关键词
        if input_lower.contains("设计") || input_lower.contains("创意")
            || input_lower.contains("艺术") || input_lower.contains("design")
            || input_lower.contains("creative") || input_lower.contains("ui")
        {
            return Some(Protocol::Lsd);
        }

        // 日常/娱乐关键词
        if input_lower.contains("吃什么") || input_lower.contains("玩什么")
            || input_lower.contains("推荐") || input_lower.contains("电影")
            || input_lower.contains("饿了") || input_lower.contains("无聊")
        {
            return Some(Protocol::Sunday);
        }

        None // 无法检测，使用默认
    }

    /// 获取所有协议
    pub fn all() -> Vec<Protocol> {
        vec![
            Protocol::Architect,
            Protocol::Reviewer2,
            Protocol::Aegis,
            Protocol::Predator,
            Protocol::McKinsey,
            Protocol::Lsd,
            Protocol::Ghost,
            Protocol::Sunday,
        ]
    }
}

/// Agent权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWeights {
    pub moss: f64,
    pub l6: f64,
    pub ultron: f64,
    pub omega: f64,
}

impl AgentWeights {
    /// 验证权重总和是否合法
    pub fn is_valid(&self) -> bool {
        let total = self.moss + self.l6 + self.ultron + self.omega;
        (total - 1.0).abs() < 0.01 // 允许0.01的误差
    }

    /// 归一化权重
    pub fn normalize(&mut self) {
        let total = self.moss + self.l6 + self.ultron + self.omega;
        if total > 0.0 {
            self.moss /= total;
            self.l6 /= total;
            self.ultron /= total;
            self.omega /= total;
        }
    }
}

/// 协议配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub protocol: Protocol,
    pub agent_weights: AgentWeights,
    pub temperature: f64,
    pub enable_jarvis_filter: bool,
    pub enable_high_freq_commands: bool,
    pub description: String,
}

impl ProtocolConfig {
    /// 获取指定协议的默认配置
    pub fn for_protocol(protocol: Protocol) -> Self {
        match protocol {
            Protocol::Architect => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.0,
                    l6: 0.15,
                    ultron: 0.05,
                    omega: 0.80, // DeepSeek绝对主力
                },
                temperature: 0.2, // 低温追求确定性
                enable_jarvis_filter: false, // 关闭闲聊过滤
                enable_high_freq_commands: true,
                description: "编程模式: Omega主导，追求代码实用性".to_string(),
            },

            Protocol::Reviewer2 => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.30,
                    l6: 0.60, // Gemini疯狂检索
                    ultron: 0.10,
                    omega: 0.0,
                },
                temperature: 0.1, // 极低温，严谨
                enable_jarvis_filter: true,
                enable_high_freq_commands: false,
                description: "学术模式: L6主导，怀疑一切".to_string(),
            },

            Protocol::Aegis => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.05,
                    l6: 0.05,
                    ultron: 0.90, // Claude绝对防御
                    omega: 0.0,
                },
                temperature: 0.05, // 极低温，零风险
                enable_jarvis_filter: true,
                enable_high_freq_commands: false,
                description: "法律模式: Ultron主宰，零后悔值".to_string(),
            },

            Protocol::Predator => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.50, // 识别市场情绪
                    l6: 0.0,
                    ultron: 0.0, // 暂时抑制
                    omega: 0.50, // DeepSeek执行高频交易
                },
                temperature: 1.0, // 高温，高噪声市场
                enable_jarvis_filter: false,
                enable_high_freq_commands: true,
                description: "金融模式: MOSS+Omega，唯快不破".to_string(),
            },

            Protocol::McKinsey => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.70, // 战略PPT
                    l6: 0.20,   // 数据图表
                    ultron: 0.10,
                    omega: 0.0,
                },
                temperature: 0.3,
                enable_jarvis_filter: true,
                enable_high_freq_commands: false,
                description: "商管模式: MOSS主导，量化优化".to_string(),
            },

            Protocol::Lsd => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.80,
                    l6: 0.0,
                    ultron: 0.0,
                    omega: 0.20,
                },
                temperature: 1.5, // 极高温，打破范式
                enable_jarvis_filter: false, // 解除逻辑一致性锁定
                enable_high_freq_commands: true,
                description: "创意模式: 高噪声，越过势垒".to_string(),
            },

            Protocol::Ghost => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.40,
                    l6: 0.0,
                    ultron: 0.10, // 切换为反取证模式
                    omega: 0.50,
                },
                temperature: 0.4,
                enable_jarvis_filter: false,
                enable_high_freq_commands: true,
                description: "影子模式: 光锥隐身，最小作用量".to_string(),
            },

            Protocol::Sunday => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.80, // 情商拉满
                    l6: 0.0,
                    ultron: 0.0, // 隐形健康保姆
                    omega: 0.20, // IoT指挥官
                },
                temperature: 1.2, // 幽默风趣
                enable_jarvis_filter: false,
                enable_high_freq_commands: true,
                description: "日常模式: 多巴胺管理，摩擦力为零".to_string(),
            },

            Protocol::Custom(_) => Self {
                protocol,
                agent_weights: AgentWeights {
                    moss: 0.50,
                    l6: 0.20,
                    ultron: 0.20,
                    omega: 0.10,
                },
                temperature: 0.7,
                enable_jarvis_filter: true,
                enable_high_freq_commands: false,
                description: "自定义模式: 用户自定义配置".to_string(),
            },
        }
    }
}

/// 协议管理器
pub struct ProtocolManager {
    current_protocol: Protocol,
    configs: HashMap<Protocol, ProtocolConfig>,
}

impl Default for ProtocolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolManager {
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // 预加载所有协议配置
        for protocol in Protocol::all() {
            configs.insert(protocol.clone(), ProtocolConfig::for_protocol(protocol));
        }

        Self {
            current_protocol: Protocol::Architect, // 默认编程模式
            configs,
        }
    }

    /// 获取当前协议
    pub fn current_protocol(&self) -> Protocol {
        self.current_protocol.clone()
    }

    /// 获取当前配置
    pub fn current_config(&self) -> &ProtocolConfig {
        self.configs.get(&self.current_protocol).unwrap()
    }

    /// 切换协议
    pub fn switch_protocol(&mut self, protocol: Protocol) {
        tracing::info!("🔀 Switching protocol: {} -> {}",
            self.current_protocol.name(),
            protocol.name()
        );
        self.current_protocol = protocol;
    }

    /// 自动检测并切换协议
    pub fn auto_detect_and_switch(&mut self, input: &str) -> Option<Protocol> {
        if let Some(detected) = Protocol::detect_from_input(input) {
            if detected != self.current_protocol {
                self.switch_protocol(detected.clone());
                return Some(detected);
            }
        }
        None
    }

    /// 获取指定协议的配置
    pub fn get_config(&self, protocol: Protocol) -> &ProtocolConfig {
        self.configs.get(&protocol).unwrap()
    }

    /// 更新协议配置
    pub fn update_config(&mut self, protocol: Protocol, config: ProtocolConfig) {
        self.configs.insert(protocol, config);
    }

    /// 打印当前协议信息
    pub fn print_current_info(&self) {
        let config = self.current_config();
        println!("\n╔══════════════════════════════════════════════╗");
        println!("║  {} {}", config.protocol.display_name(), config.protocol.tagline());
        println!("╠══════════════════════════════════════════════╣");
        println!("║  哲学: {}", config.protocol.philosophy());
        println!("║  温度: {:.2}", config.temperature);
        println!("║  Agent权重:");
        println!("║    MOSS:   {:.0}%", config.agent_weights.moss * 100.0);
        println!("║    L6:     {:.0}%", config.agent_weights.l6 * 100.0);
        println!("║    Ultron: {:.0}%", config.agent_weights.ultron * 100.0);
        println!("║    Omega:  {:.0}%", config.agent_weights.omega * 100.0);
        println!("║  TUI颜色: {}", config.protocol.tui_color());
        println!("╚══════════════════════════════════════════════╝\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_detection() {
        assert_eq!(
            Protocol::detect_from_input("帮我写个爬虫"),
            Some(Protocol::Architect)
        );

        assert_eq!(
            Protocol::detect_from_input("看看这份合同有没有坑"),
            Some(Protocol::Aegis)
        );

        assert_eq!(
            Protocol::detect_from_input("这公司股价虚高吗"),
            Some(Protocol::Predator)
        );

        assert_eq!(
            Protocol::detect_from_input("饿了"),
            Some(Protocol::Sunday)
        );
    }

    #[test]
    fn test_agent_weights_validation() {
        let mut weights = AgentWeights {
            moss: 0.5,
            l6: 0.3,
            ultron: 0.1,
            omega: 0.1,
        };

        assert!(weights.is_valid());

        weights.moss = 0.6; // 现在总和是1.1
        assert!(!weights.is_valid());

        weights.normalize(); // 归一化
        assert!(weights.is_valid());
    }

    #[test]
    fn test_protocol_config() {
        let config = ProtocolConfig::for_protocol(Protocol::Architect);

        assert_eq!(config.protocol, Protocol::Architect);
        assert_eq!(config.agent_weights.omega, 0.80); // Omega主导
        assert!(config.agent_weights.is_valid());
    }

    #[test]
    fn test_protocol_manager() {
        let mut manager = ProtocolManager::new();

        assert_eq!(manager.current_protocol(), Protocol::Architect);

        manager.switch_protocol(Protocol::Sunday);
        assert_eq!(manager.current_protocol(), Protocol::Sunday);

        // 自动检测
        let detected = manager.auto_detect_and_switch("帮我看看这份合同");
        assert_eq!(detected, Some(Protocol::Aegis));
        assert_eq!(manager.current_protocol(), Protocol::Aegis);
    }

    #[test]
    fn test_all_protocols_have_configs() {
        let manager = ProtocolManager::new();

        for protocol in Protocol::all() {
            let config = manager.get_config(protocol);
            assert_eq!(config.protocol, protocol);
            assert!(config.agent_weights.is_valid());
        }
    }
}
