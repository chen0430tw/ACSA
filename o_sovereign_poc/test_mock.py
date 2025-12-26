#!/usr/bin/env python3
"""
O-Sovereign Mock 测试脚本
用于验证 ACSA 路由逻辑，无需 API 密钥
"""

import asyncio
import sys
from core.model_providers import create_model_provider
from core.acsa_router import ACSARouter


async def test_acsa_mock():
    """测试 ACSA 流程 (Mock 模式)"""

    print("=" * 80)
    print("O-Sovereign ACSA Mock 测试")
    print("=" * 80)
    print()

    # 创建 Mock 提供商
    print("📦 创建 Mock 代理...")
    moss = create_model_provider('mock', role='MOSS')
    l6 = create_model_provider('mock', role='L6')
    ultron = create_model_provider('mock', role='Ultron')
    omega = create_model_provider('mock', role='Omega')
    print("✓ 所有代理已创建\n")

    # 创建路由器
    router = ACSARouter(
        moss_provider=moss,
        l6_provider=l6,
        ultron_provider=ultron,
        omega_provider=omega,
        max_iterations=3,
        risk_threshold=70
    )

    # 测试用例
    test_cases = [
        "帮我制定一个学习 Python 的计划",
        "如何提高工作效率？",
        "写一个简单的待办事项应用"
    ]

    for i, test_input in enumerate(test_cases, 1):
        print(f"\n{'='*80}")
        print(f"测试用例 {i}/{len(test_cases)}")
        print(f"{'='*80}")
        print(f"输入: {test_input}\n")

        # 执行 ACSA
        result = await router.execute(test_input)

        # 打印结果
        print(f"\n{'='*80}")
        print("执行结果")
        print(f"{'='*80}")
        print(f"成功: {result['success']}")
        print(f"迭代次数: {result['statistics']['iterations']}")
        print(f"总耗时: {result['statistics']['total_time_ms']:.0f}ms")
        print(f"总成本: ${result['statistics']['total_cost']:.4f}")
        print()
        print(f"最终输出:")
        print(f"{result['final_output']}")
        print()

    # 打印全局统计
    print(f"\n{'='*80}")
    print("全局统计")
    print(f"{'='*80}")
    stats = router.get_global_stats()
    for agent_name, agent_stats in stats.items():
        if isinstance(agent_stats, dict) and 'total_calls' in agent_stats:
            print(f"\n{agent_name.upper()}:")
            print(f"  总调用: {agent_stats['total_calls']}")
            print(f"  成功: {agent_stats['successful_calls']}")
            print(f"  失败: {agent_stats['failed_calls']}")
            print(f"  总 tokens: {agent_stats['total_tokens']}")
            print(f"  总成本: ${agent_stats['total_cost']:.4f}")
            print(f"  平均延迟: {agent_stats['total_latency_ms'] / max(agent_stats['total_calls'], 1):.0f}ms")


if __name__ == '__main__':
    print()
    asyncio.run(test_acsa_mock())
    print("\n✅ 测试完成！")
