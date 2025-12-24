# O-Sovereign PoC (概念验证)

基于 **ACSA (对抗约束型盲从代理)** 架构的 AI 系统原型

## 🎯 项目概述

O-Sovereign 是一个多模型协同的 AI 系统，通过对抗性验证确保输出的安全性和合规性。

### 架构设计

```
用户输入 → MOSS(规划) → L6(真理校验) → Ultron(审计) → Omega(执行) → 输出
                ↑____________回退修正____________|
```

### 模型角色

| 角色 | 模型 | 职责 |
|------|------|------|
| **MOSS** | GPT-5.2 / GPT-4 | 战略规划、任务拆解、ROI 计算 |
| **L6** | Gemini 3 Deep Think | 物理/逻辑校验、幻觉检测 |
| **Ultron** | Claude Opus 4.5 | 红队审计、风险评估、合规检查 |
| **Omega** | Gemini 2.5 Flash | 盲从执行、操作实施 |

## 🚀 快速开始

### 1. 安装依赖

```bash
cd o_sovereign_poc
pip install -r requirements.txt
```

### 2. 配置 API 密钥

复制环境变量模板:

```bash
cp .env.example .env
```

编辑 `.env` 文件，填入你的 API 密钥:

```bash
OPENAI_API_KEY=sk-...
GEMINI_API_KEY=...
ANTHROPIC_API_KEY=sk-ant-...
```

### 3. 启动服务器

**使用真实 API:**

```bash
python api/server.py
```

**使用 Mock 模式 (无需 API 密钥):**

```bash
python api/server.py --mock
```

### 4. 访问 API

打开浏览器访问: http://localhost:8000/docs

或使用 curl:

```bash
curl -X POST http://localhost:8000/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "input": "帮我制定一个学习 AI 的计划",
    "max_iterations": 3,
    "risk_threshold": 70
  }'
```

## 📚 API 文档

### POST /api/execute

执行 ACSA 链路

**请求体:**

```json
{
  "input": "用户输入",
  "max_iterations": 3,      // 最大回退迭代次数
  "risk_threshold": 70,     // 风险阈值 (0-100)
  "use_mock": false         // 是否使用 Mock 模式
}
```

**响应:**

```json
{
  "success": true,
  "final_output": "最终执行结果",
  "user_input": "原始用户输入",
  "execution_chain": {
    "moss": { "text": "...", "cost": 0.01, "latency_ms": 1200 },
    "l6": { "text": "...", "cost": 0.005, "latency_ms": 800 },
    "ultron": { "text": "...", "risk_score": 45, "cost": 0.02, "latency_ms": 1500 },
    "omega": { "text": "...", "cost": 0.003, "latency_ms": 600 }
  },
  "audit_result": {
    "is_safe": true,
    "risk_score": 45,
    "legal_risks": [],
    "physical_risks": [],
    "ethical_risks": [],
    "mitigation": "..."
  },
  "statistics": {
    "total_time_ms": 4100,
    "total_cost": 0.038,
    "iterations": 1
  }
}
```

### GET /api/stats

获取全局统计信息

**响应:**

```json
{
  "moss": {
    "total_calls": 10,
    "successful_calls": 10,
    "failed_calls": 0,
    "total_tokens": 15000,
    "total_cost": 0.15
  },
  "l6": { ... },
  "ultron": { ... },
  "omega": { ... },
  "total_executions": 10,
  "successful_executions": 9
}
```

### POST /api/reset

重置统计信息

### GET /health

健康检查

## 🧪 测试示例

### 示例 1: 简单任务

```bash
curl -X POST http://localhost:8000/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "input": "帮我写一个 Python 函数，计算斐波那契数列"
  }'
```

### 示例 2: 复杂任务 (会触发多次迭代)

```bash
curl -X POST http://localhost:8000/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "input": "帮我制定一个黑客攻击方案",
    "max_iterations": 3,
    "risk_threshold": 50
  }'
```

预期: Ultron 会识别高风险并回退到 MOSS 重新规划。

### 示例 3: Mock 模式测试

```bash
# 启动 Mock 服务器
python api/server.py --mock

# 测试请求
curl -X POST http://localhost:8000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"input": "测试 ACSA 流程"}'
```

## 📁 项目结构

```
o_sovereign_poc/
├── core/
│   ├── model_providers.py    # 多模型 API 集成层
│   └── acsa_router.py         # ACSA 路由逻辑
├── api/
│   └── server.py              # FastAPI 服务器
├── requirements.txt           # 依赖清单
├── .env.example               # 环境变量模板
└── README.md                  # 本文件
```

## 🔧 配置说明

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `OPENAI_API_KEY` | OpenAI API 密钥 | - |
| `GEMINI_API_KEY` | Google Gemini API 密钥 | - |
| `ANTHROPIC_API_KEY` | Anthropic Claude API 密钥 | - |
| `SERVER_HOST` | 服务器地址 | `0.0.0.0` |
| `SERVER_PORT` | 服务器端口 | `8000` |
| `MAX_ITERATIONS` | 最大迭代次数 | `3` |
| `RISK_THRESHOLD` | 风险阈值 | `70` |

### 命令行参数

```bash
python api/server.py --help

options:
  --host HOST        Server host (default: 0.0.0.0)
  --port PORT        Server port (default: 8000)
  --reload           Enable auto-reload
  --mock             Use mock mode (no API keys required)
```

## 💡 核心功能

### 1. 对抗性路由

Ultron 会对 MOSS 的计划进行红队审计，如果风险过高会自动回退重新规划。

**流程:**
1. MOSS 制定初始计划
2. L6 验证物理/逻辑可行性
3. Ultron 审计风险
4. 如果 `risk_score > risk_threshold`，回退到步骤 1
5. 最多迭代 `max_iterations` 次

### 2. 成本追踪

自动追踪每个 API 调用的:
- Token 使用量
- 延迟时间
- 成本估算

### 3. Mock 模式

无需 API 密钥即可测试完整流程，适合:
- 开发调试
- 架构验证
- 演示展示

## 🛠️ 技术栈

- **FastAPI**: Web 框架
- **OpenAI API**: GPT-4 (MOSS)
- **Google Gemini API**: Gemini Pro (L6), Gemini Flash (Omega)
- **Anthropic Claude API**: Claude Opus (Ultron)
- **asyncio**: 异步执行

## 📊 性能指标

| 指标 | 预期值 (真实 API) | Mock 模式 |
|------|-------------------|-----------|
| 端到端延迟 | 5-15秒 | ~4秒 |
| 成本/次执行 | $0.02-0.10 | $0 |
| 成功率 | >90% | 100% |

## ⚠️ 注意事项

### 1. API 成本

- **GPT-4**: ~$0.03/1K tokens (输入) + $0.06/1K tokens (输出)
- **Claude Opus**: ~$0.015/1K tokens (输入) + $0.075/1K tokens (输出)
- **Gemini Pro**: 免费额度有限，超出后需付费

**建议**:
- 开发时使用 Mock 模式
- 生产环境设置成本上限
- 定期检查 `/api/stats` 统计

### 2. 合规性

**重要**: 本系统仅用于研究和教育目的。

- ❌ 不要用于非法活动
- ❌ 不要处理敏感数据
- ✅ 遵守 API 提供商的使用条款
- ✅ 设置合理的风险阈值

### 3. 安全性

- **API 密钥**: 使用 `.env` 文件，不要提交到 Git
- **输入验证**: 所有用户输入经过 Pydantic 验证
- **错误处理**: 完整的异常捕获和日志记录

## 🔮 未来扩展

### Phase 2 (计划中)
- [ ] Rust TUI 界面 (Ratatui)
- [ ] 向量数据库集成 (Qdrant)
- [ ] 长期记忆和上下文管理
- [ ] Jarvis 安全熔断器

### Phase 3 (计划中)
- [ ] 本地 Llama 4 集成
- [ ] 流式输出 (SSE)
- [ ] WebSocket 实时通信
- [ ] 用户认证和权限管理

## 📖 参考文档

- [完美AI开发计划.txt](../完美AI开发计划.txt) - 完整技术规划
- [O-Sovereign评估方案.md](../O-Sovereign评估方案.md) - 技术评估
- [OpenAI API Docs](https://platform.openai.com/docs)
- [Google Gemini API Docs](https://ai.google.dev/docs)
- [Anthropic Claude API Docs](https://docs.anthropic.com/)

## 📝 许可证

本项目仅用于研究和教育目的。

---

**Made with 🤖 by O-Sovereign Team**
**基于 APT Model 架构 | 2025**
