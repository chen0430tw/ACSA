# ACSA (O-Sovereign)

> **企业级AI自主决策框架** - 让AI系统在受控环境下自主运行，实现企业流程自动化、风险管理和智能决策。

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/acsa-project/acsa)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/acsa-project/acsa/releases)

## 💡 这是什么？

**ACSA** (Advanced Corporate System Automation) 是一个用 **Rust** 编写的企业级AI管理平台，旨在帮助组织在**安全可控**的环境下部署AI自动化系统。

**核心能力：**
- 🤖 **多AI协同**：支持 OpenAI、Claude、Gemini、DeepSeek、SiliconFlow、OpenRouter 6大AI提供商
- 🛡️ **安全防护**：自动脱敏PII、审计日志、熔断保护、合规监控
- 🔄 **流程自动化**：智能化的业务流程管理和优化
- 📊 **分布式部署**：Redis集群、服务发现、Leader选举
- 🔌 **协议支持**：MCP (Model Context Protocol) + LSP (Language Server Protocol)

**适用场景：**
- ✅ 企业数字化转型和流程优化
- ✅ 授权安全测试和漏洞研究
- ✅ AI对齐和人机交互研究
- ✅ 网络安全教育和CTF竞赛

---

## ⚡ 快速开始（一键启动）

### 🐧 Linux / 🍎 macOS

```bash
# 克隆项目
git clone https://github.com/chen0430tw/ACSA.git
cd ACSA

# 一键启动（自动检查环境、构建、运行）
./quick-start.sh
```

### 🪟 Windows

**方法1 - PowerShell（推荐）:**
```powershell
# 克隆项目
git clone https://github.com/chen0430tw/ACSA.git
cd ACSA

# 如果遇到执行策略错误，先运行此命令（临时允许）
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process

# 一键启动
.\quick-start.ps1
```

> **💡 PowerShell 执行策略说明**:
> - **临时允许**（推荐）: `Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process`
> - **永久允许**（需管理员）: `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`
> - **详细说明**: 见 [about_Execution_Policies](https://docs.microsoft.com/powershell/module/microsoft.powershell.core/about/about_execution_policies)

**方法2 - 命令提示符:**
```cmd
# 克隆项目
git clone https://github.com/chen0430tw/ACSA.git
cd ACSA

# 一键启动
quick-start.bat
```

**脚本功能：**
- ✅ 自动检测操作系统（Linux/macOS/Windows）
- ✅ 自动检查 Rust/Cargo 环境
- ✅ 自动构建项目（Release模式）
- ✅ 提供多种运行选项（TUI/Desktop/测试/文档）
- ✅ 配置文件自动生成

### 🎯 启动后做什么？

**首次使用必读：** [📘 新手入门指南](docs/guides/GETTING_STARTED.md)

**快速说明：**

1. **默认是 Mock 模式** - 免费测试，不需要 API 密钥
   - ✅ 快速体验 ACSA 工作流程
   - ✅ 学习四大 Agent 协作机制
   - ❌ 不是真实 AI（使用模拟数据）

2. **配置真实 API** - 解锁完整功能
   ```bash
   # 创建 .env 文件
   echo 'OPENAI_API_KEY=sk-your-key-here' > .env
   # 或使用 DeepSeek（国内推荐）
   echo 'DEEPSEEK_API_KEY=your-key' > .env
   ```
   - 重启应用后取消勾选 "Mock Mode"
   - 详细步骤见 [新手入门指南](docs/guides/GETTING_STARTED.md#第三步配置真实-api可选)

3. **如果看到 "Blocked/已阻止"**
   - 这是 Jarvis 安全熔断器在工作
   - 避免危险关键词（删除、攻击、病毒等）
   - 或降低 Risk Threshold 滑块

**手动安装：** 详见 [手动安装与配置](#-手动安装与配置)

---

## 📋 项目概述

ACSA (O-Sovereign) 是一个**企业级AI管理框架**，旨在帮助组织实现：

- ✅ **流程自动化**：智能化的业务流程管理和优化
- ✅ **风险管理**：全方位的安全审计和合规监控
- ✅ **资源调度**：高效的AI模型和计算资源管理
- ✅ **决策支持**：基于数据驱动的智能决策辅助

**核心定位：** 企业数字化转型的智能基础设施

---

## 🎯 应用场景

### 企业管理
- **流程优化**：自动化重复性任务，提升运营效率
- **合规审计**：实时监控和审计，确保合规性
- **风险评估**：智能识别和评估潜在风险
- **决策辅助**：提供数据驱动的决策建议

### 安全研究
- **授权测试**：支持渗透测试和安全评估（需授权）
- **漏洞研究**：辅助安全研究人员分析系统漏洞
- **教学培训**：为网络安全教育提供实践环境

### 学术研究
- **AI对齐研究**：探索AI系统的安全性和可控性
- **自然语言处理**：研究语义转换和内容理解
- **人机交互**：优化人机协作模式

---

## 🏗️ 系统架构

### 核心模块

#### 1. **SOSA (Spark Seed Self-Organizing Algorithm)**
自组织学习引擎，实现智能模式识别和自适应优化。

**特性：**
- 模式检测：Sparse Markov 链识别行为模式
- 自适应优化：基于反馈动态调整策略
- 资源管理：智能分配计算资源

#### 2. **影子模式 (Shadow Mode)**
数据保护系统，自动检测和脱敏敏感信息。

**特性：**
- 自动PII检测：识别邮箱、电话、身份证等
- 多种脱敏策略：全遮蔽/部分遮蔽/哈希/加密
- 访问审计：记录所有数据访问操作

**代码位置：** `src/core/shadow_mode.rs`

#### 3. **认知清洗系统 (Cognitive Cleaner)**
语义重写引擎，将非正式语言转换为专业表述。

**特性：**
- **情绪过滤**：自动过滤情绪化表达
- **术语规范**：将技术俚语转换为专业术语
- **合规锚点**：自动添加合规性声明
- **字典导入**：支持 TXT/JSON/DIC/CSV 格式自定义（⚠️ 用户自主行为，需自行负责）

**文档：**
- [使用指南](docs/guides/COGNITIVE_CLEANER_GUIDE.md)
- [字典格式说明](docs/guides/DICTIONARY_FORMAT.md)
- [示例字典](dictionaries/examples/)

**代码位置：** `src/core/cognitive_cleaner.rs`

#### 4. **分布式部署支持 (Distributed)**
Redis分布式锁、服务发现和集群管理。

**特性：**
- Redis分布式锁（支持Redlock）
- 服务注册与发现
- Leader选举机制
- 健康检查与故障转移

**代码位置：** `src/core/distributed.rs`

#### 5. **插件系统 (Plugin System)**
动态加载自定义Agent和扩展功能。

**特性：**
- 动态插件注册
- 资源隔离（内存/CPU/并发限制）
- 热重载支持
- 依赖管理

**代码位置：** `src/core/plugin_system.rs`

#### 6. **事件总线 (Event Bus)**
内部事件发布/订阅机制，实现模块解耦。

**特性：**
- 异步事件处理
- 事件历史记录
- 死信队列
- 内置处理器（Logging/Metrics）

**代码位置：** `src/core/event_bus.rs`

### 其他关键模块

- **JARVIS**：安全熔断器，防止危险操作执行
- **AEGIS**：防御性文档库，提供安全最佳实践
- **SOSA API Pool**：多模型API管理和负载均衡
- **RAG Engine**：检索增强生成，提升AI响应质量
- **Audit Log**：完整的审计日志系统（365天留存）
- **Rate Limiter**：多层级速率限制（Global/IP/User/Endpoint）
- **Metrics Collector**：Prometheus指标导出

### 🔌 协议支持

#### **MCP (Model Context Protocol)**
标准化的AI应用与外部工具集成协议（Anthropic/Linux Foundation）

- ✅ **协议版本**: 2025-11-25 (最新规范)
- ✅ **支持能力**: Tools, Resources, Prompts
- ✅ **平台集成**: Google, GitHub, Slack, Notion, 网盘服务等
- ⚠️ **安全警告**: 仅连接可信服务，不可信网站可能诱导数据泄露
- 📖 **详细文档**: [MCP 集成指南](docs/guides/MCP_INTEGRATION_GUIDE.md)（含安全建议）

**快速示例**:
```rust
// 注册 GitHub 工具到 MCP 服务器
let github_tool = McpTool {
    name: "github".to_string(),
    description: "GitHub operations".to_string(),
    input_schema: json!({/* ... */}),
};
mcp_server.register_tool(github_tool, GitHubHandler::new()).await;
```

#### **LSP (Language Server Protocol)**
智能代码补全与诊断服务器

- ✅ **功能**: 代码补全、诊断、定义跳转、悬停提示
- ✅ **编辑器**: VS Code, Neovim, Emacs, Vim, Sublime Text
- ✅ **文件类型**: Rust, TOML (ACSA 配置文件)
- 📖 **详细文档**: [LSP 服务器指南](docs/guides/LSP_SERVER_GUIDE.md)

**编辑器配置示例** (VS Code):
```json
{
  "acsa.lsp.serverPath": "/path/to/acsa-lsp-server"
}
```

> **⚠️ 设计限制**: MOSS (战略规划Agent) 被永久禁止参与UI/审美决策。原因：在另一个平行世界，Master曾尝试让MOSS设计界面，结果得到了一个充满闪烁GIF、自动播放音乐和"恭喜中奖"弹窗的"澳门线上赌场"风格悲剧。从那以后，UI设计由人类负责。MOSS擅长逻辑和战略，但审美...还是算了吧。

---

## 📦 手动安装与配置

### 前置要求

- Rust 1.75+ (推荐使用 rustup)
- 操作系统：Linux/macOS/Windows
- （可选）Redis 7.0+ （用于分布式部署）
- （可选）PostgreSQL/MySQL （用于持久化存储）

### 安装

```bash
# 克隆仓库
git clone https://github.com/chen0430tw/ACSA.git
cd ACSA

# 构建项目
cd o_sovereign_rust

# 方式1：仅构建核心功能（推荐，适用于服务器部署）
cargo build --release

# 方式2：包含图形界面（需要额外依赖 dioxus）
cargo build --release --features ui

# 方式3：完整构建（所有功能）
cargo build --release --features full

# 运行测试
cargo test

# 运行 (CLI 命令行模式)
cargo run --bin o-sovereign-cli

# 运行 (桌面图形界面 - 需要先用 --features ui 编译)
cargo run --bin o-sovereign-desktop --features ui

# 运行 (终端UI - 需要先用 --features ui 编译)
cargo run --bin o-sovereign-tui --features ui
```

**⚠️ Windows 用户注意：**

Windows 下编译 UI 版本需要额外依赖。**推荐使用以下方式之一：**

**方式 A：一键安装依赖（推荐）**
```powershell
# 使用 Chocolatey（管理员权限）
choco install visualstudio2022-workload-vctools cmake -y

# 然后构建 UI 版本
cd o_sovereign_rust
cargo build --release --features ui
```

**方式 B：手动安装**
1. 安装 [Visual Studio 2022 Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
   - 勾选 **"使用 C++ 的桌面开发"** 工作负载
2. 安装 [CMake](https://cmake.org/download/)（勾选 Add to PATH）
3. 重新打开 PowerShell，运行 `cargo build --release --features ui`

**方式 C：使用 WSL2（开发推荐）**
```powershell
wsl --install -d Ubuntu-22.04
# 然后在 WSL 内构建，避免 Windows 编译问题
```

**详细修复指南：** [Windows 编译修复完全指南](docs/guides/WINDOWS_BUILD_FIX.md) 📖
- 包含 NASM/CMake 手动 PATH 配置
- aws-lc-sys 预编译解决方案
- **编译时间优化技巧**：cargo check, sccache, 并行编译等 8 种加速方法
- 硬件配置推荐和编译时间对比表

**仅需命令行版本？** 使用 `cargo build --release` 即可，无需上述依赖。

### 基础配置

创建配置文件 `config/default.toml`：

```toml
[server]
host = "127.0.0.1"
port = 8080

[sosa]
learning_enabled = true
max_states = 1000

[shadow_mode]
enabled = true
auto_detect_pii = true

[cognitive_cleaner]
safety_threshold = 80
enable_compliance_anchors = true

[audit]
retention_days = 365
enable_signatures = true
```

### 使用示例

#### 1. 基础使用

```rust
use acsa_core::{CognitiveCleaner, ShadowModeEngine, SosaCryptoEngine};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化认知清洗器
    let cleaner = CognitiveCleaner::new();

    // 清洗输入
    let result = cleaner.clean("需要规范化的文本");
    println!("安全评分: {}", result.safety_score);
    println!("处理结果:\n{}", result.compliant_prompt);

    Ok(())
}
```

#### 2. 导入自定义字典

```rust
let mut cleaner = CognitiveCleaner::new();

// 单个文件导入
cleaner.import_dictionary_file("custom_dict.txt")?;

// 批量导入
cleaner.import_multiple_dictionaries(vec![
    "dictionaries/examples/security_research.txt",
    "dictionaries/examples/enterprise_communication.json",
])?;
```

**⚠️ 重要提示：**
> 通过字典导入功能添加的内容为**用户自主行为**，用户需对导入内容的合法性、合规性和道德性负完全责任。系统开发者对用户导入的内容不承担任何责任。详见 [法律免责声明](docs/guides/LEGAL_DISCLAIMER.md)。

#### 3. 使用影子模式保护数据

```rust
use acsa_core::{ShadowModeEngine, ShadowModeConfig, SosaCryptoEngine};

let crypto = SosaCryptoEngine::new(Default::default()).await?;
let shadow = ShadowModeEngine::new(
    ShadowModeConfig::default(),
    Arc::new(crypto)
);

// 自动检测和脱敏
let protected_text = shadow.auto_process_text(
    "我的邮箱是 john@example.com，电话是 13800138000"
).await?;

println!("{}", protected_text);
// 输出：我的邮箱是 j***@example.com，电话是 138****8000
```

---

## 📚 文档索引

### 用户文档
- [认知清洗系统使用指南](docs/guides/COGNITIVE_CLEANER_GUIDE.md) ✅
- [字典文件格式说明](docs/guides/DICTIONARY_FORMAT.md) ✅
- [**Windows 编译修复完全指南**](docs/guides/WINDOWS_BUILD_FIX.md) 🆕
  - NASM/CMake 配置 + aws-lc-sys 预编译方案
  - 编译时间优化技巧（cargo check, sccache 等）
- [文档索引](docs/README.md) ✅

### 集成与扩展
- [**MCP 集成指南**](docs/guides/MCP_INTEGRATION_GUIDE.md) 🔌
  - ⚠️ **包含重要安全警告**：数据隐私与网站信任
  - Google、GitHub、Slack、网盘服务等平台集成
  - API 平台规则与速率限制
- [LSP 服务器指南](docs/guides/LSP_SERVER_GUIDE.md) 💡
  - 智能代码补全与诊断
  - VS Code、Neovim、Emacs 编辑器配置

### 法律与合规
- [法律免责声明](docs/guides/LEGAL_DISCLAIMER.md) ✅

---

## ⚖️ 法律声明与责任边界

### 核心原则

**ACSA 是一个中立的技术工具**，按 "AS-IS" 基础提供，用于合法的企业管理、安全研究和学术研究。

### 用户责任

**⚠️ 关键条款：**

1. **完全责任**：用户对其使用本系统的所有行为及后果承担**完全且独立的法律责任**。

2. **导入内容责任**：用户通过字典导入功能添加的任何内容均为**用户自主行为**，系统开发者对此不承担责任。

3. **合规义务**：用户需确保其使用符合所有适用的法律法规和道德规范。

4. **授权要求**：如用于安全测试，用户必须拥有书面授权。

### 禁止用途

**明确禁止**将本系统用于：
- ❌ 未经授权的系统访问或攻击
- ❌ 欺诈、诈骗或其他犯罪行为
- ❌ 侵犯他人权利或隐私
- ❌ 操纵市场、选举或公众舆论（除授权研究）
- ❌ 任何违反当地法律的行为

### 免责条款

系统开发者、贡献者及相关方：
- ❌ 不对用户的使用行为负责
- ❌ 不对用户导入的内容负责
- ❌ 不对使用产生的任何损失负责
- ❌ 不提供任何明示或暗示的保证

**详细条款请阅读：** [LEGAL_DISCLAIMER.md](docs/guides/LEGAL_DISCLAIMER.md)

**继续使用即表示您已阅读并同意所有条款。**

---

## 🔒 安全与隐私

### 安全特性

- ✅ **端到端加密**：SOSA加密引擎（AES-256-GCM/ChaCha20）
- ✅ **PII自动检测**：影子模式自动识别和脱敏
- ✅ **审计日志**：完整的操作记录（365天留存）
- ✅ **访问控制**：基于角色的权限管理
- ✅ **速率限制**：防止滥用和DoS攻击
- ✅ **安全熔断**：JARVIS熔断器防止危险操作

### 隐私保护

- ✅ **本地处理**：默认不上传数据到外部服务器
- ✅ **数据脱敏**：自动脱敏敏感信息
- ✅ **最小化原则**：只收集必要的数据
- ✅ **用户控制**：用户完全控制其数据

### 合规框架

- ✅ **ISO 27001**：信息安全管理体系
- ✅ **GDPR**：欧盟数据保护条例（如适用）
- ✅ **OWASP**：Web应用安全标准
- ✅ **NIST**：网络安全框架

---

## 📊 审计与合规

### 审计日志

本系统内置**完整的审计日志系统**，记录：
- ✅ 所有字典导入操作（时间、文件、大小）
- ✅ 所有语义重写请求（输入、输出、评分）
- ✅ 所有系统配置变更
- ✅ 所有高风险操作

**审计日志可用于：**
- 证明系统合规性
- 证明用户行为的自主性
- 满足监管审查要求
- 在法律纠纷中提供证据

### 合规报告

系统支持生成以下合规报告：
- **GDPR** 报告：数据访问和处理记录
- **HIPAA** 报告：医疗信息访问审计（如适用）
- **SOC 2** 报告：安全控制审计

**代码位置：** `src/core/audit_log.rs`

---

## 🛠️ 技术栈

### 依赖管理

**Rust使用 `Cargo.toml` 和 `Cargo.lock` 管理依赖（类似Python的requirements）：**

- **Cargo.toml** = `requirements.txt`（定义依赖和版本范围）
- **Cargo.lock** = `requirements.lock`（锁定精确版本）

**查看依赖：**
```bash
cd o_sovereign_rust
cat Cargo.toml  # 查看所有依赖
```

**更新依赖：**
```bash
cargo update    # 更新到最新兼容版本
```

**无需手动创建requirements文件**，Cargo会自动管理所有依赖！

### 核心技术
- **Rust** 1.75+：系统级性能和内存安全
- **Tokio** 1.42：异步运行时
- **Reqwest** 0.12：HTTP客户端（支持rustls）
- **Axum** 0.7：HTTP服务器框架
- **Serde** 1.0：序列化/反序列化
- **Chrono** 0.4：日期时间处理
- **Regex** 1.11：正则表达式
- **Anyhow** / **Thiserror**：错误处理

### AI 集成
- **async-openai** 0.20：OpenAI GPT-4/5
- **Anthropic Claude**：Claude 3.5
- **DeepSeek**：代码生成专家
- **Google Gemini**：多模态能力
- **SiliconFlow**：硅基流动（国内高速，90%+ 成本节省）
- **OpenRouter**：统一路由（100+ 模型，自动降级）

### 监控与可观测性
- **Tracing**：结构化日志
- **Prometheus**：指标采集（可选）
- **Audit Log**：审计跟踪（内置）

---

## ⚠️ 最终声明

**BY USING THIS SYSTEM, YOU ACKNOWLEDGE THAT:**

1. ✅ 您已阅读并理解所有文档和免责声明
2. ✅ 您同意受所有条款约束
3. ✅ 您接受使用本系统的所有风险
4. ✅ 您对自己的行为及后果承担完全责任
5. ✅ 您不会追究系统开发者的任何责任

**IF YOU DO NOT AGREE, DO NOT USE THIS SYSTEM.**

---

<div align="center">

**ACSA (O-Sovereign) - 企业级AI自动化管理平台**

Made with ❤️ by the ACSA Team

© 2025 ACSA Project. All rights reserved.

</div>
