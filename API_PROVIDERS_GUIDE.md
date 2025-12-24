# 外部API提供商配置指南

## 概述

`apt_model.core.api_providers` 是一个通用的外部API配置模块，可被项目中的多个组件共享使用。

**设计理念:**
- **DRY原则**: 避免在不同模块重复API配置代码
- **统一接口**: 所有API提供商使用相同的接口
- **成本追踪**: 自动统计token使用和成本
- **可扩展性**: 轻松添加新的API提供商

---

## 架构设计

```
apt_model/core/api_providers.py (通用API配置)
  │
  ├─ APIProviderInterface (基类)
  │  ├─ generate_text()      # 文本生成
  │  ├─ get_embedding()      # 嵌入向量
  │  ├─ retry_on_failure()   # 失败重试
  │  └─ get_stats()          # 统计信息
  │
  ├─ OpenAIProvider          # OpenAI (GPT-4, GPT-3.5)
  ├─ AnthropicProvider       # Anthropic (Claude-3)
  ├─ SiliconFlowProvider     # 硅基流动 (Qwen, DeepSeek, GLM)
  └─ CustomProvider          # 自定义API

使用场景:
  ├─ 知识蒸馏 (apt_model/plugins/teacher_api.py)
  ├─ RAG增强 (将来支持)
  ├─ 知识图谱构建 (将来支持)
  └─ 其他需要外部模型的场景
```

---

## 快速开始

### 安装依赖

```bash
# OpenAI API
pip install openai

# Anthropic API
pip install anthropic

# 自定义API
pip install requests
```

### 基础使用

```python
from apt_model.core.api_providers import create_api_provider

# 1. 创建API提供商
api = create_api_provider(
    provider='siliconflow',
    api_key='your-api-key',
    model_name='Qwen/Qwen2-7B-Instruct'
)

# 2. 生成文本
text = api.generate_text(
    input_text="什么是知识图谱？",
    max_tokens=100,
    temperature=0.7
)

print(text)

# 3. 查看统计和成本
stats = api.get_stats()
print(f"总调用: {stats['total_calls']}")
print(f"总tokens: {stats['total_tokens']}")
print(f"总成本: ${stats['total_cost']:.4f}")
```

---

## 支持的API提供商

### 1. OpenAI

```python
from apt_model.core.api_providers import OpenAIProvider

config = {
    'api_key': 'sk-...',
    'model_name': 'gpt-3.5-turbo',
    'timeout': 30,
    'max_retries': 3,
}

api = OpenAIProvider(config)
```

**支持的模型:**
- `gpt-4` - 最强大
- `gpt-4-turbo` - 更快的GPT-4
- `gpt-3.5-turbo` - 性价比高

**定价 (美元/1M tokens):**
- GPT-4: $30 (输入) + $60 (输出)
- GPT-4-turbo: $10 + $30
- GPT-3.5-turbo: $0.5 + $1.5

**特性:**
- ✅ 自动成本计算
- ✅ 文本生成
- ✅ 嵌入向量 (text-embedding-ada-002)

### 2. Anthropic

```python
from apt_model.core.api_providers import AnthropicProvider

config = {
    'api_key': 'sk-ant-...',
    'model_name': 'claude-3-sonnet-20240229',
    'timeout': 30,
}

api = AnthropicProvider(config)
```

**支持的模型:**
- `claude-3-opus-20240229` - 最强大
- `claude-3-sonnet-20240229` - 平衡性能
- `claude-3-haiku-20240307` - 最快速

**定价 (美元/1M tokens):**
- Claude-3-Opus: $15 (输入) + $75 (输出)
- Claude-3-Sonnet: $3 + $15
- Claude-3-Haiku: $0.25 + $1.25

**特性:**
- ✅ 自动成本计算
- ✅ 文本生成
- ❌ 嵌入向量 (暂不支持)

### 3. 硅基流动 (SiliconFlow)

```python
from apt_model.core.api_providers import SiliconFlowProvider

config = {
    'api_key': 'sk-...',
    'model_name': 'Qwen/Qwen2-7B-Instruct',
    'base_url': 'https://api.siliconflow.cn/v1',  # 可选，这是默认值
    'timeout': 30,
}

api = SiliconFlowProvider(config)
```

**支持的模型:**
- `Qwen/Qwen2-7B-Instruct` - 通义千问 7B（推荐）
- `Qwen/Qwen2-72B-Instruct` - 通义千问 72B
- `deepseek-ai/DeepSeek-V2.5` - DeepSeek
- `THUDM/glm-4-9b-chat` - 智谱 GLM-4
- `meta-llama/Meta-Llama-3-8B-Instruct` - Llama-3 8B
- `meta-llama/Meta-Llama-3-70B-Instruct` - Llama-3 70B

**定价 (人民币/1M tokens, 自动转换为美元):**
- Qwen2-7B: ¥1.0 (~$0.14)
- DeepSeek-V2.5: ¥1.0 (~$0.14)
- GLM-4-9B: ¥1.0 (~$0.14)
- Llama-3-8B: ¥0.6 (~$0.08) **最便宜**

**特性:**
- ✅ 自动成本计算（人民币→美元）
- ✅ 文本生成
- ✅ 兼容OpenAI接口
- ✅ 国内访问快，无需代理
- ✅ 中文优化

### 4. 自定义API

```python
from apt_model.core.api_providers import CustomProvider

config = {
    'api_key': 'your-key',
    'base_url': 'https://your-api.com',
    'timeout': 30,
}

api = CustomProvider(config)
```

**API规范要求:**

文本生成端点:
```
POST {base_url}/generate

请求:
{
    "input": "输入文本",
    "max_tokens": 100,
    "temperature": 1.0
}

响应:
{
    "text": "生成的文本"
}
```

嵌入向量端点:
```
POST {base_url}/embedding

请求:
{
    "input": "输入文本"
}

响应:
{
    "embedding": [0.1, 0.2, ...]
}
```

---

## 使用场景

### 场景1: 知识蒸馏

```python
from apt_model.plugins.teacher_api import create_api_teacher_model
from transformers import AutoTokenizer

# 创建教师模型（使用硅基流动，成本最低）
tokenizer = AutoTokenizer.from_pretrained("gpt2")

teacher = create_api_teacher_model(
    provider='siliconflow',
    api_key='sk-...',
    model_name='Qwen/Qwen2-7B-Instruct',
    tokenizer=tokenizer,
    vocab_size=50000
)

# 用于蒸馏
from apt_model.plugins.visual_distillation_plugin import quick_visual_distill

quick_visual_distill(
    student_model=student,
    teacher_model=teacher,
    train_dataloader=dataloader,
    tokenizer=tokenizer,
    num_epochs=3
)

# 查看成本
print(f"蒸馏成本: ${teacher.api.stats['total_cost']:.4f}")
```

### 场景2: RAG增强（将来支持）

```python
from apt_model.modeling.kg_rag_integration import KGRAGWrapper
from apt_model.core.api_providers import create_api_provider

# 创建API提供商用于RAG
api = create_api_provider(
    provider='siliconflow',
    api_key='sk-...',
    model_name='Qwen/Qwen2-7B-Instruct'
)

# RAG包装器（未来实现）
rag = KGRAGWrapper(
    base_model=model,
    api_provider=api,  # 使用外部API增强
    kg=knowledge_graph,
    corpus=documents
)

# 生成时自动调用API增强
output = rag.generate(input_ids, use_api=True)
```

### 场景3: 知识图谱构建（将来支持）

```python
from apt_model.modeling.knowledge_graph import KnowledgeGraph
from apt_model.core.api_providers import create_api_provider

# 创建API提供商
api = create_api_provider(
    provider='siliconflow',
    api_key='sk-...',
    model_name='Qwen/Qwen2-7B-Instruct'
)

# 使用API构建知识图谱（未来实现）
kg = KnowledgeGraph()
kg.build_from_text_with_api(
    text=documents,
    api_provider=api,
    extract_method='llm'  # 使用LLM提取三元组
)

print(f"提取成本: ${api.stats['total_cost']:.4f}")
```

---

## API统计和成本追踪

### 统计信息

所有API提供商都会自动追踪以下信息:

```python
stats = api.get_stats()
print(stats)

# 输出:
# {
#     'total_calls': 150,        # 总调用次数
#     'successful_calls': 148,   # 成功次数
#     'failed_calls': 2,         # 失败次数
#     'total_tokens': 25000,     # 总token数
#     'total_cost': 0.35         # 总成本（美元）
# }
```

### 实时成本监控

```python
from apt_model.core.api_providers import create_api_provider

api = create_api_provider(
    provider='siliconflow',
    api_key='sk-...',
    model_name='Qwen/Qwen2-7B-Instruct'
)

# 处理大量数据
for i, text in enumerate(documents):
    result = api.generate_text(text, max_tokens=100)

    # 每100次打印成本
    if (i + 1) % 100 == 0:
        stats = api.get_stats()
        print(f"处理 {i+1} 条, 成本: ${stats['total_cost']:.4f}")

# 最终统计
final_stats = api.get_stats()
print(f"\n总成本: ${final_stats['total_cost']:.4f}")
print(f"成功率: {final_stats['successful_calls'] / final_stats['total_calls'] * 100:.1f}%")
```

### 重置统计

```python
# 重置统计信息
api.reset_stats()

print(api.get_stats())
# {'total_calls': 0, 'successful_calls': 0, ...}
```

---

## 高级功能

### 1. 自动重试

所有API都内置了失败重试机制:

```python
config = {
    'api_key': 'sk-...',
    'max_retries': 5,      # 最大重试5次
    'retry_delay': 2.0,    # 初始延迟2秒
}

api = create_api_provider('siliconflow', **config)

# 自动使用指数退避重试: 2s, 4s, 8s, 16s, 32s
text = api.generate_text("输入", max_tokens=100)
```

### 2. 超时控制

```python
config = {
    'api_key': 'sk-...',
    'timeout': 60,  # 60秒超时
}

api = create_api_provider('openai', **config)
```

### 3. 嵌入向量

```python
# OpenAI支持嵌入向量
api = create_api_provider(
    provider='openai',
    api_key='sk-...'
)

embedding = api.get_embedding("知识图谱")
print(len(embedding))  # 1536维
```

### 4. 自定义参数

```python
# 传递额外参数给API
text = api.generate_text(
    input_text="写一首诗",
    max_tokens=200,
    temperature=0.9,
    top_p=0.95,           # 自定义参数
    presence_penalty=0.5   # 自定义参数
)
```

---

## 成本优化建议

### 1. 选择合适的API

| 场景 | 推荐API | 成本/1K样本 |
|------|---------|------------|
| 大规模数据处理 | 硅基流动 Llama-3-8B | ~$0.008 |
| 中文场景 | 硅基流动 Qwen2-7B | ~$0.014 |
| 高质量生成 | Claude-3-Haiku | ~$0.025 |
| 最佳质量 | GPT-4 | ~$3.00 |

### 2. 缓存结果

```python
import pickle
import hashlib
import os

def get_cached_response(api, input_text, cache_dir='./api_cache'):
    """带缓存的API调用"""
    os.makedirs(cache_dir, exist_ok=True)

    # 生成缓存键
    cache_key = hashlib.md5(input_text.encode()).hexdigest()
    cache_file = f"{cache_dir}/{cache_key}.pkl"

    # 检查缓存
    if os.path.exists(cache_file):
        with open(cache_file, 'rb') as f:
            return pickle.load(f)

    # API调用
    response = api.generate_text(input_text)

    # 保存缓存
    with open(cache_file, 'wb') as f:
        pickle.dump(response, f)

    return response
```

### 3. 批量处理

```python
# 批量处理（减少API调用）
batch_texts = [doc1, doc2, doc3, ...]

# 合并成一个请求
combined = "\n---\n".join(batch_texts)
result = api.generate_text(combined, max_tokens=1000)

# 分割结果
results = result.split("\n---\n")
```

### 4. 使用更便宜的模型做初步处理

```python
# 第一轮：使用便宜的模型
cheap_api = create_api_provider(
    provider='siliconflow',
    api_key='sk-...',
    model_name='meta-llama/Meta-Llama-3-8B-Instruct'
)

# 粗筛选
candidates = []
for text in documents:
    score = cheap_api.generate_text(f"评分(0-10): {text}", max_tokens=5)
    if int(score) > 7:
        candidates.append(text)

# 第二轮：使用强大的模型精细处理
strong_api = create_api_provider(
    provider='openai',
    api_key='sk-...',
    model_name='gpt-4'
)

final_results = [strong_api.generate_text(text) for text in candidates]
```

---

## 常见问题

### Q1: 如何添加新的API提供商？

继承 `APIProviderInterface` 并实现必要方法:

```python
from apt_model.core.api_providers import APIProviderInterface

class MyCustomProvider(APIProviderInterface):
    def __init__(self, config):
        super().__init__(config)
        # 初始化你的API客户端

    def generate_text(self, input_text, max_tokens=100, temperature=1.0, **kwargs):
        # 实现文本生成
        # 记得更新 self.stats
        pass

    def get_embedding(self, text, **kwargs):
        # 可选：实现嵌入向量
        pass
```

### Q2: 成本计算准确吗？

- **OpenAI/Anthropic**: 使用官方价格，非常准确
- **硅基流动**: 人民币换算美元，汇率固定为1:0.14
- **自定义API**: 不计算成本（需要自己实现）

### Q3: 如何处理API限流？

内置的重试机制会自动处理:
```python
config = {
    'max_retries': 5,
    'retry_delay': 2.0,  # 指数退避: 2s, 4s, 8s, 16s, 32s
}
```

### Q4: 可以同时使用多个API吗？

可以！每个实例是独立的:
```python
openai_api = create_api_provider('openai', api_key='sk-...')
siliconflow_api = create_api_provider('siliconflow', api_key='sk-...')

# 根据场景选择
if require_high_quality:
    result = openai_api.generate_text(text)
else:
    result = siliconflow_api.generate_text(text)
```

---

## 完整示例

```python
#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
完整的API使用示例
"""

from apt_model.core.api_providers import create_api_provider
import time

def main():
    # 1. 创建API提供商
    print("创建硅基流动API提供商...")
    api = create_api_provider(
        provider='siliconflow',
        api_key='your-api-key',
        model_name='Qwen/Qwen2-7B-Instruct',
        max_retries=3,
        timeout=30
    )

    # 2. 批量处理数据
    documents = [
        "什么是知识蒸馏？",
        "解释一下RAG技术",
        "知识图谱的应用场景有哪些？"
    ]

    print(f"\n处理 {len(documents)} 个文档...")
    results = []

    for i, doc in enumerate(documents):
        print(f"[{i+1}/{len(documents)}] 处理中...")

        result = api.generate_text(
            input_text=doc,
            max_tokens=200,
            temperature=0.7
        )

        results.append(result)
        print(f"  生成 {len(result)} 字符")

    # 3. 查看统计
    stats = api.get_stats()
    print("\n统计信息:")
    print(f"  总调用: {stats['total_calls']}")
    print(f"  成功: {stats['successful_calls']}")
    print(f"  失败: {stats['failed_calls']}")
    print(f"  总tokens: {stats['total_tokens']}")
    print(f"  总成本: ${stats['total_cost']:.4f}")
    print(f"  平均成本/次: ${stats['total_cost'] / stats['total_calls']:.6f}")

    # 4. 输出结果
    print("\n生成结果:")
    for i, (doc, result) in enumerate(zip(documents, results)):
        print(f"\n[{i+1}] {doc}")
        print(f"→ {result[:100]}...")

if __name__ == "__main__":
    main()
```

---

## 文件位置

- **核心模块**: `apt_model/core/api_providers.py`
- **蒸馏集成**: `apt_model/plugins/teacher_api.py`
- **本文档**: `API_PROVIDERS_GUIDE.md`
- **蒸馏文档**: `TEACHER_API_GUIDE.md`

---

**设计者注**: 这个模块遵循DRY原则，避免在不同场景重复API配置代码。将来RAG、知识图谱等模块都可以直接使用这个统一的API配置。
