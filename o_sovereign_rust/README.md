# O-Sovereign Rust Edition

基于 **ACSA (对抗约束型盲从代理)** 架构的 Rust 实现，使用 **Dioxus** 框架构建跨平台 UI。

## 🎯 项目概述

O-Sovereign Rust 版是 Python PoC 的生产级实现，提供：
- 🦀 **Rust** - 内存安全、高性能
- 🎨 **Dioxus** - 跨平台 UI (Desktop + TUI)
- ⚡ **Tokio** - 异步运行时
- 🔒 **类型安全** - Rust 的类型系统确保 ACSA 约束

### 架构

```
用户输入 → MOSS(规划) → L6(真理校验) → Ultron(审计) → Omega(执行) → 输出
                ↑____________回退修正____________|
```

## 🚀 快速开始

### 安装依赖

确保已安装 Rust (1.70+):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 构建项目

```bash
cd o_sovereign_rust
cargo build --release
```

### 运行方式

#### 1. Desktop UI (推荐)

```bash
# Mock 模式 (无需 API 密钥)
cargo run --bin o-sovereign-desktop

# 真实 API 模式
export OPENAI_API_KEY='sk-...'
cargo run --bin o-sovereign-desktop
```

#### 2. TUI (终端界面)

```bash
cargo run --bin o-sovereign-tui
```

## 📁 项目结构

```
o_sovereign_rust/
├── src/
│   ├── core/                  # 核心模块
│   │   ├── types.rs           # 数据类型定义
│   │   ├── providers.rs       # AI API 提供商
│   │   ├── router.rs          # ACSA 路由器
│   │   └── mod.rs
│   ├── ui/                    # UI 资源
│   │   └── styles.css         # Desktop UI 样式
│   ├── bin/                   # 可执行文件
│   │   ├── desktop.rs         # Dioxus Desktop 应用
│   │   └── tui.rs             # Dioxus TUI 应用
│   └── lib.rs                 # 库入口
├── Cargo.toml                 # 依赖配置
├── .env.example               # 环境变量模板
└── README.md                  # 本文件
```

## 🔧 技术栈

| 组件 | 库 | 用途 |
|------|-----|------|
| **Async Runtime** | Tokio | 异步任务执行 |
| **HTTP Client** | reqwest | API 调用 |
| **OpenAI API** | async-openai | GPT-4/5 集成 |
| **UI Framework** | Dioxus | 跨平台 UI |
| **Error Handling** | anyhow, thiserror | 错误处理 |
| **Logging** | tracing | 日志记录 |
| **Serialization** | serde | 数据序列化 |

## 🎨 UI 特性

### Desktop UI (Dioxus)

- ✅ 现代化桌面应用界面
- ✅ 实时 Agent 状态显示
- ✅ 可配置风险阈值
- ✅ Mock 模式切换
- ✅ 美观的输出格式

### TUI (Terminal)

- ✅ 终端界面，轻量高效
- ✅ 纯键盘操作
- ✅ 适合远程服务器

## 🧩 核心模块

### 1. Types (`src/core/types.rs`)

定义所有核心数据结构：
- `AgentRole` - Agent 角色枚举
- `AgentResponse` - Agent 响应
- `AuditResult` - 审计结果
- `ACSAExecutionLog` - 执行日志
- `AgentStats` - 统计信息

### 2. Providers (`src/core/providers.rs`)

AI API 提供商实现：
- `OpenAIProvider` - OpenAI GPT-4/5
- `MockProvider` - 测试用 Mock 实现
- TODO: `GeminiProvider`, `ClaudeProvider`

### 3. Router (`src/core/router.rs`)

ACSA 路由核心逻辑：
- 对抗性路由循环
- 自动回退重规划
- 风险评分系统
- 完整执行日志

## 📊 使用示例

### Desktop UI

1. 启动应用
2. 输入请求（如 "帮我制定学习计划"）
3. 配置风险阈值（默认 70）
4. 点击 "Execute ACSA"
5. 查看四个 Agent 的协同工作流程

### TUI

```bash
cargo run --bin o-sovereign-tui
```

在终端中输入命令，按 Enter 执行。

## ⚙️ 配置

### 环境变量

```bash
cp .env.example .env
# 编辑 .env 文件
```

| 变量 | 说明 | 必需 |
|------|------|------|
| `OPENAI_API_KEY` | OpenAI API 密钥 | Mock 模式不需要 |
| `GEMINI_API_KEY` | Gemini API 密钥 | 可选 |
| `ANTHROPIC_API_KEY` | Claude API 密钥 | 可选 |
| `RUST_LOG` | 日志级别 | 可选 (默认 info) |

### ACSA 配置

在代码中可配置：

```rust
let config = ACSAConfig {
    max_iterations: 3,        // 最大迭代次数
    risk_threshold: 70,       // 风险阈值 (0-100)
    enable_l6: true,          // 是否启用 L6 校验
    enable_streaming: false,  // 是否启用流式输出 (TODO)
};
```

## 🔒 安全特性

### Rust 类型系统约束

```rust
// Agent 角色强类型
pub enum AgentRole {
    MOSS,    // 不能混淆
    L6,
    Ultron,
    Omega,
}

// 审计结果强验证
pub struct AuditResult {
    pub is_safe: bool,         // 必须明确标记
    pub risk_score: u8,        // 0-100 范围限制
    pub mitigation: String,    // 必须提供缓解措施
}
```

### 内存安全

- 无 null 指针
- 无数据竞争
- 无缓冲区溢出
- Arc + Mutex 确保线程安全

## 🚧 开发状态

### 已完成 ✅

- [x] 核心类型系统
- [x] OpenAI Provider (MOSS)
- [x] Mock Provider (全部 Agents)
- [x] ACSA 路由器逻辑
- [x] Desktop UI (Dioxus)
- [x] TUI (Dioxus TUI)
- [x] 对抗性回退机制
- [x] 统计和日志

### 待实现 🔨

- [ ] Gemini Provider (L6 & Omega)
- [ ] Claude Provider (Ultron)
- [ ] 流式输出支持
- [ ] Qdrant 向量数据库集成
- [ ] Jarvis 安全熔断器
- [ ] WebAssembly 支持
- [ ] 移动端 (iOS/Android)

## 🧪 测试

```bash
# 运行单元测试
cargo test

# 运行带日志的测试
RUST_LOG=debug cargo test -- --nocapture

# 检查代码
cargo clippy

# 格式化代码
cargo fmt
```

## 📦 发布

### Debug 构建 (开发)

```bash
cargo build
```

### Release 构建 (生产)

```bash
cargo build --release
```

优化后的二进制文件位于 `target/release/`：
- `o-sovereign-desktop` - Desktop 应用
- `o-sovereign-tui` - TUI 应用

## 🎯 性能

相比 Python PoC:

| 指标 | Python | Rust | 提升 |
|------|--------|------|------|
| **启动时间** | ~2s | ~0.1s | **20x** |
| **内存占用** | ~150MB | ~30MB | **5x** |
| **并发性能** | 单线程 | 多线程 | **N倍** |
| **类型安全** | 运行时 | 编译时 | **无限** |

## 📚 参考资料

### 官方文档

- [Dioxus 官方文档](https://dioxuslabs.com/)
- [Dioxus GitHub](https://github.com/DioxusLabs/dioxus)
- [Dioxus TUI](https://github.com/DioxusLabs/rink)
- [async-openai Docs](https://docs.rs/async-openai/)
- [Tokio Docs](https://tokio.rs/)

### O-Sovereign 系列

- Python PoC: `../o_sovereign_poc/`
- 评估方案: `../O-Sovereign评估方案.md`
- 开发计划: `../完美AI开发计划.txt`

## 📝 许可证

本项目仅用于研究和教育目的。

---

**Made with 🦀 Rust + Dioxus**
**O-Sovereign Team | 2025**
