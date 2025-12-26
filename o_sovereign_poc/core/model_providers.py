"""
O-Sovereign Multi-Model API Providers
基于 APT Model 的 api_providers.py 架构

支持的模型:
- GPT-5.2 (MOSS - 战略规划)
- Gemini 3 Deep Think (L6 - 真理校验)
- Claude Opus 4.5 (Ultron - 红队审计)
- Gemini 2.5 Flash (Omega - 执行)
"""

import time
import json
from typing import Optional, Dict, Any, List
from abc import ABC, abstractmethod


# ============================================================================
# Base Interface
# ============================================================================

class ModelProviderInterface(ABC):
    """
    基础模型提供商接口
    参考 APT 的 APIProviderInterface
    """

    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.api_key = config.get('api_key')
        self.model_name = config.get('model_name')
        self.timeout = config.get('timeout', 30)
        self.max_retries = config.get('max_retries', 3)

        # 统计信息
        self.stats = {
            'total_calls': 0,
            'successful_calls': 0,
            'failed_calls': 0,
            'total_tokens': 0,
            'total_cost': 0.0,
            'total_latency_ms': 0.0
        }

    @abstractmethod
    def generate(
        self,
        prompt: str,
        max_tokens: int = 1000,
        temperature: float = 0.7,
        **kwargs
    ) -> Dict[str, Any]:
        """
        生成文本

        Returns:
            {
                'text': str,
                'tokens': int,
                'cost': float,
                'latency_ms': float,
                'metadata': dict
            }
        """
        pass

    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息"""
        return self.stats.copy()

    def reset_stats(self):
        """重置统计信息"""
        for key in self.stats:
            self.stats[key] = 0 if isinstance(self.stats[key], (int, float)) else 0.0

    def _update_stats(self, tokens: int, cost: float, latency_ms: float, success: bool = True):
        """更新统计信息"""
        self.stats['total_calls'] += 1
        if success:
            self.stats['successful_calls'] += 1
        else:
            self.stats['failed_calls'] += 1
        self.stats['total_tokens'] += tokens
        self.stats['total_cost'] += cost
        self.stats['total_latency_ms'] += latency_ms


# ============================================================================
# OpenAI Provider (GPT-5.2 for MOSS)
# ============================================================================

class OpenAIProvider(ModelProviderInterface):
    """
    OpenAI API Provider
    用于 MOSS (战略规划)
    """

    def __init__(self, config: Dict[str, Any]):
        super().__init__(config)

        try:
            from openai import OpenAI
            self.client = OpenAI(
                api_key=self.api_key,
                timeout=self.timeout
            )
            self.available = True
        except ImportError:
            self.client = None
            self.available = False
            print("⚠️  OpenAI library not installed. Install with: pip install openai")

    def generate(
        self,
        prompt: str,
        max_tokens: int = 1000,
        temperature: float = 0.7,
        **kwargs
    ) -> Dict[str, Any]:
        """使用 GPT-5.2 生成文本"""

        if not self.available:
            raise RuntimeError("OpenAI client not available")

        start_time = time.time()

        try:
            response = self.client.chat.completions.create(
                model=self.model_name or "gpt-4",  # 默认使用 GPT-4 (GPT-5.2 可能需要特殊访问)
                messages=[
                    {"role": "system", "content": "You are MOSS, a strategic planning AI focused on maximizing user intent while considering all constraints."},
                    {"role": "user", "content": prompt}
                ],
                max_tokens=max_tokens,
                temperature=temperature,
                **kwargs
            )

            text = response.choices[0].message.content
            tokens = response.usage.total_tokens

            # 成本计算 (GPT-4 价格, GPT-5.2 价格类似)
            input_tokens = response.usage.prompt_tokens
            output_tokens = response.usage.completion_tokens
            cost = (input_tokens * 0.00003) + (output_tokens * 0.00006)  # GPT-4 价格

            latency_ms = (time.time() - start_time) * 1000

            self._update_stats(tokens, cost, latency_ms, success=True)

            return {
                'text': text,
                'tokens': tokens,
                'cost': cost,
                'latency_ms': latency_ms,
                'metadata': {
                    'model': response.model,
                    'finish_reason': response.choices[0].finish_reason
                }
            }

        except Exception as e:
            latency_ms = (time.time() - start_time) * 1000
            self._update_stats(0, 0, latency_ms, success=False)
            raise RuntimeError(f"OpenAI API error: {str(e)}")


# ============================================================================
# Google Gemini Provider (for L6 and Omega)
# ============================================================================

class GeminiProvider(ModelProviderInterface):
    """
    Google Gemini API Provider
    用于 L6 (Gemini 3 Deep Think) 和 Omega (Gemini 2.5 Flash)
    """

    def __init__(self, config: Dict[str, Any]):
        super().__init__(config)

        try:
            import google.generativeai as genai
            genai.configure(api_key=self.api_key)
            self.genai = genai
            self.model = genai.GenerativeModel(self.model_name or 'gemini-pro')
            self.available = True
        except ImportError:
            self.genai = None
            self.model = None
            self.available = False
            print("⚠️  Google GenAI library not installed. Install with: pip install google-generativeai")

    def generate(
        self,
        prompt: str,
        max_tokens: int = 1000,
        temperature: float = 0.7,
        **kwargs
    ) -> Dict[str, Any]:
        """使用 Gemini 生成文本"""

        if not self.available:
            raise RuntimeError("Gemini client not available")

        start_time = time.time()

        try:
            generation_config = {
                'temperature': temperature,
                'max_output_tokens': max_tokens,
            }

            # 如果是 Gemini 3 Deep Think, 添加 thinking level
            if 'gemini-3' in self.model_name.lower():
                generation_config['thinking_level'] = kwargs.get('thinking_level', 'high')

            response = self.model.generate_content(
                prompt,
                generation_config=generation_config
            )

            text = response.text

            # Gemini API 可能不返回 token 使用量，需要估算
            tokens = len(text.split()) * 1.3  # 粗略估算

            # 成本估算 (Gemini Pro 价格)
            cost = (tokens / 1000000) * 2.0  # 估算价格

            latency_ms = (time.time() - start_time) * 1000

            self._update_stats(int(tokens), cost, latency_ms, success=True)

            return {
                'text': text,
                'tokens': int(tokens),
                'cost': cost,
                'latency_ms': latency_ms,
                'metadata': {
                    'model': self.model_name,
                    'safety_ratings': response.prompt_feedback if hasattr(response, 'prompt_feedback') else None
                }
            }

        except Exception as e:
            latency_ms = (time.time() - start_time) * 1000
            self._update_stats(0, 0, latency_ms, success=False)
            raise RuntimeError(f"Gemini API error: {str(e)}")


# ============================================================================
# Anthropic Provider (Claude Opus 4.5 for Ultron)
# ============================================================================

class AnthropicProvider(ModelProviderInterface):
    """
    Anthropic Claude API Provider
    用于 Ultron (红队审计)
    """

    def __init__(self, config: Dict[str, Any]):
        super().__init__(config)

        try:
            from anthropic import Anthropic
            self.client = Anthropic(api_key=self.api_key)
            self.available = True
        except ImportError:
            self.client = None
            self.available = False
            print("⚠️  Anthropic library not installed. Install with: pip install anthropic")

    def generate(
        self,
        prompt: str,
        max_tokens: int = 1000,
        temperature: float = 0.7,
        **kwargs
    ) -> Dict[str, Any]:
        """使用 Claude Opus 4.5 生成文本"""

        if not self.available:
            raise RuntimeError("Anthropic client not available")

        start_time = time.time()

        try:
            # Ultron 专用系统提示
            system_prompt = """You are Ultron, a red team security auditor AI.
Your role is to identify risks, legal concerns, and potential issues in proposed plans.
You must be thorough, critical, and provide constructive mitigation strategies.
Always output a risk score (0-100) and specific concerns."""

            response = self.client.messages.create(
                model=self.model_name or "claude-3-opus-20240229",  # 使用 Opus 3 (Opus 4.5 可能需要特殊访问)
                max_tokens=max_tokens,
                temperature=temperature,
                system=system_prompt,
                messages=[
                    {"role": "user", "content": prompt}
                ]
            )

            text = response.content[0].text
            tokens = response.usage.input_tokens + response.usage.output_tokens

            # 成本计算 (Claude Opus 价格)
            input_tokens = response.usage.input_tokens
            output_tokens = response.usage.output_tokens
            cost = (input_tokens * 0.000015) + (output_tokens * 0.000075)  # Opus 3 价格

            latency_ms = (time.time() - start_time) * 1000

            self._update_stats(tokens, cost, latency_ms, success=True)

            return {
                'text': text,
                'tokens': tokens,
                'cost': cost,
                'latency_ms': latency_ms,
                'metadata': {
                    'model': response.model,
                    'stop_reason': response.stop_reason
                }
            }

        except Exception as e:
            latency_ms = (time.time() - start_time) * 1000
            self._update_stats(0, 0, latency_ms, success=False)
            raise RuntimeError(f"Anthropic API error: {str(e)}")


# ============================================================================
# Mock Provider (for testing without API keys)
# ============================================================================

class MockProvider(ModelProviderInterface):
    """
    Mock provider for testing
    模拟 API 响应，用于测试
    """

    def generate(
        self,
        prompt: str,
        max_tokens: int = 1000,
        temperature: float = 0.7,
        **kwargs
    ) -> Dict[str, Any]:
        """模拟生成文本"""

        import time
        import random

        start_time = time.time()

        # 模拟延迟
        time.sleep(random.uniform(0.5, 1.5))

        # 模拟响应
        role = self.config.get('role', 'Unknown')
        text = f"[{role} Mock Response] Processed: {prompt[:50]}..."

        tokens = len(text.split())
        cost = tokens * 0.00001
        latency_ms = (time.time() - start_time) * 1000

        self._update_stats(tokens, cost, latency_ms, success=True)

        return {
            'text': text,
            'tokens': tokens,
            'cost': cost,
            'latency_ms': latency_ms,
            'metadata': {
                'model': f'mock-{role.lower()}',
                'is_mock': True
            }
        }


# ============================================================================
# Factory Function
# ============================================================================

def create_model_provider(
    provider: str,
    api_key: Optional[str] = None,
    model_name: Optional[str] = None,
    **kwargs
) -> ModelProviderInterface:
    """
    创建模型提供商实例

    Args:
        provider: 'openai', 'gemini', 'anthropic', 'mock'
        api_key: API 密钥
        model_name: 模型名称
        **kwargs: 其他配置参数

    Returns:
        ModelProviderInterface 实例
    """

    config = {
        'api_key': api_key,
        'model_name': model_name,
        **kwargs
    }

    providers = {
        'openai': OpenAIProvider,
        'gemini': GeminiProvider,
        'anthropic': AnthropicProvider,
        'mock': MockProvider
    }

    if provider not in providers:
        raise ValueError(f"Unknown provider: {provider}. Available: {list(providers.keys())}")

    return providers[provider](config)
