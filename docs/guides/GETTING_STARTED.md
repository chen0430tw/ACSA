# ACSA 新手入门指南

> 从零开始使用 ACSA，5分钟快速上手

---

## 📌 第一步：启动应用

### Windows用户

```powershell
# 1. 克隆项目
git clone https://github.com/chen0430tw/ACSA.git
cd ACSA

# 2. 允许脚本执行（如果遇到执行策略错误）
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process

# 3. 运行启动脚本
.\quick-start.ps1

# 4. 选择运行模式
# 输入 1 选择 TUI（终端界面，推荐）
# 输入 2 选择 Desktop（图形界面）
```

### Linux/macOS用户

```bash
./quick-start.sh
```

---

## 🎯 第二步：理解 Mock 模式

**启动后你会看到什么？**

Desktop 界面默认有一个勾选的选项：**"Use Mock Mode (no API keys required)"**

### Mock 模式是什么？

- ✅ **免费测试模式**：不需要任何 API 密钥
- ✅ **模拟AI响应**：使用硬编码的测试数据
- ✅ **学习架构**：了解 ACSA 的工作流程
- ❌ **不是真实AI**：无法处理真实任务

### 什么时候使用 Mock 模式？

- 🧪 **第一次使用 ACSA** - 快速了解界面和流程
- 📚 **学习和研究** - 不想消耗 API 配额
- 🔧 **开发和测试** - 调试代码时避免真实 API 调用

---

## 🔑 第三步：配置真实 API（可选）

### 为什么需要 API 密钥？

只有配置了真实 API 密钥，才能：
- ✅ 使用真实的 AI 模型（OpenAI GPT, Claude, Gemini等）
- ✅ 处理真实的业务任务
- ✅ 获得智能化的决策支持

### 方法1：创建 .env 文件（推荐）

在项目根目录创建 `.env` 文件：

```bash
# 示例：使用 OpenAI
OPENAI_API_KEY=sk-your-api-key-here

# 示例：使用 Claude（Anthropic）
ANTHROPIC_API_KEY=sk-ant-your-key-here

# 示例：使用 DeepSeek（国内推荐）
DEEPSEEK_API_KEY=your-deepseek-key

# 示例：使用 OpenRouter（支持多模型）
OPENROUTER_API_KEY=sk-or-your-key-here
```

**保存后重启应用**，然后在界面中**取消勾选 Mock Mode**。

### 方法2：环境变量（临时）

```powershell
# Windows PowerShell
$env:OPENAI_API_KEY="sk-your-api-key-here"
.\quick-start.ps1

# Linux/macOS
export OPENAI_API_KEY="sk-your-api-key-here"
./quick-start.sh
```

### 如何获取 API 密钥？

| 提供商 | 注册地址 | 费用 | 推荐度 |
|--------|----------|------|--------|
| **DeepSeek** | https://platform.deepseek.com | 低成本（￥1/百万token）| ⭐⭐⭐⭐⭐ 国内首选 |
| **OpenRouter** | https://openrouter.ai | 按需付费 | ⭐⭐⭐⭐ 支持多模型 |
| **OpenAI** | https://platform.openai.com | 中等成本 | ⭐⭐⭐⭐ 经典选择 |
| **SiliconFlow** | https://siliconflow.cn | 低成本 | ⭐⭐⭐ 国内可用 |
| **Anthropic** | https://console.anthropic.com | 高成本 | ⭐⭐⭐ Claude 官方 |

---

## 🚀 第四步：发送第一个请求

### 在 Mock 模式下测试

1. 在输入框输入：`帮我分析一下市场趋势`
2. 点击 **🚀 Execute ACSA**
3. 观察4个Agent的状态变化：
   - 🧠 **MOSS**: 生成计划
   - 🛡️ **Ultron**: 审计安全
   - 🔬 **L6**: 合规检查
   - ⚡ **OMEGA**: 执行任务

### 使用真实 API

1. 配置好 API 密钥（见第三步）
2. **取消勾选 Mock Mode**
3. 输入真实任务，例如：
   - `帮我写一个Python HTTP服务器`
   - `分析这个错误日志：[粘贴日志]`
   - `优化我的Rust代码性能`

---

## ⚠️ 常见问题

### Q1: 界面显示 "Blocked" 或 "已阻止"

**原因：** Jarvis 安全熔断器检测到危险操作

**解决：**
- 检查输入是否包含危险关键词（删除数据、攻击、病毒等）
- 调整 **Risk Threshold** 滑块（降低风险阈值）
- 如果是合法测试，请明确说明用途（如："用于安全研究"）

### Q2: 为什么没有语言切换选项？

**当前状态：** i18n 国际化模块已实现（支持中文、英文、日语、韩语），但 UI 集成尚未完成

**临时方案：** 系统根据操作系统语言自动选择
- Windows：通过系统语言设置
- Linux/macOS：通过 `LANG` 环境变量

**计划：** 后续版本将添加 UI 语言切换器

### Q3: 没有添加 API 的按钮？

**当前设计：** API 密钥通过环境变量配置（见第三步）

**为什么这样设计？**
- 🔒 **安全性**：避免密钥明文存储在界面中
- 🔐 **最佳实践**：遵循12-Factor App原则
- 🛡️ **防泄露**：不会被意外提交到 Git

**计划：** 后续版本将添加加密的本地配置面板

### Q4: TUI 输入看不到光标？

**已修复！** 最新版本已解决光标显示问题
- 如果仍有问题，请更新到最新代码

### Q5: Desktop 启动报错 "No platform feature enabled"

**已修复！** 最新版本已正确配置 Dioxus 平台特性
- 如果仍有问题，运行 `cargo clean` 后重新构建

### Q6: Windows PowerShell 脚本报错（乱码/语法错误）

**原因：** PowerShell 5.1 UTF-8 编码问题

**解决方案：** 见 [Windows 编译问题修复指南](WINDOWS_BUILD_FIX.md#q6-powershell-51-编码问题)

---

## 📚 进阶学习

### 架构理解

- [ACSA 架构说明](../ARCHITECTURE.md)
- [四大 Agent 职责分工](../architecture/AGENTS.md)
- [MCP 协议集成指南](MCP_INTEGRATION_GUIDE.md)

### 开发指南

- [添加新的 AI 提供商](../development/ADD_NEW_PROVIDER.md)
- [自定义安全规则](../development/CUSTOM_SECURITY_RULES.md)
- [扩展插件系统](../development/PLUGIN_SYSTEM.md)

### 故障排查

- [Windows 编译问题](WINDOWS_BUILD_FIX.md)
- [常见错误代码](../troubleshooting/ERROR_CODES.md)
- [性能优化建议](../troubleshooting/PERFORMANCE.md)

---

## 🎓 下一步做什么？

### 初级用户
1. ✅ 在 Mock 模式下熟悉界面
2. ✅ 配置一个 API 密钥（推荐 DeepSeek）
3. ✅ 尝试不同类型的任务（代码、分析、写作）

### 中级用户
1. ✅ 理解四大 Agent 的协作流程
2. ✅ 调整 Risk Threshold 观察行为变化
3. ✅ 查看审计日志（`logs/` 目录）

### 高级用户
1. ✅ 阅读架构文档，理解设计理念
2. ✅ 自定义 Jarvis 安全规则
3. ✅ 贡献代码或报告 Issue

---

## 💬 需要帮助？

- 📖 **文档首页**：[docs/README.md](../README.md)
- 🐛 **报告问题**：[GitHub Issues](https://github.com/chen0430tw/ACSA/issues)
- 💡 **功能建议**：[GitHub Discussions](https://github.com/chen0430tw/ACSA/discussions)
- 📧 **联系作者**：见项目 README

---

**祝你使用愉快！** 🎉

如有问题，欢迎随时提问。
