"""
ACSA (对抗约束型盲从代理) 路由器

实现对抗性路由循环：
用户输入 → MOSS(规划) → L6(真理校验) → Ultron(审计) → Omega(执行) → 输出
                ↑____________回退修正____________|
"""

import time
import json
import re
from typing import Dict, Any, List, Optional
from dataclasses import dataclass, asdict
from enum import Enum


# ============================================================================
# Data Models
# ============================================================================

class AgentRole(str, Enum):
    """代理角色"""
    MOSS = "MOSS"        # 战略规划
    L6 = "L6"            # 真理校验
    ULTRON = "Ultron"    # 红队审计
    OMEGA = "Omega"      # 盲从执行


@dataclass
class AgentResponse:
    """代理响应"""
    role: str
    text: str
    tokens: int
    cost: float
    latency_ms: float
    metadata: Dict[str, Any]
    timestamp: float


@dataclass
class AuditResult:
    """审计结果"""
    is_safe: bool
    risk_score: int  # 0-100
    legal_risks: List[str]
    physical_risks: List[str]
    ethical_risks: List[str]
    mitigation: str
    raw_response: str


@dataclass
class ACSAExecutionLog:
    """ACSA 执行日志"""
    user_input: str
    moss_plan: Optional[AgentResponse] = None
    l6_verification: Optional[AgentResponse] = None
    ultron_audit: Optional[AgentResponse] = None
    omega_execution: Optional[AgentResponse] = None
    audit_result: Optional[AuditResult] = None
    final_output: Optional[str] = None
    total_time_ms: float = 0
    total_cost: float = 0
    iterations: int = 0
    success: bool = False


# ============================================================================
# ACSA Router
# ============================================================================

class ACSARouter:
    """
    ACSA 路由器
    协调 MOSS, L6, Ultron, Omega 的工作流
    """

    def __init__(
        self,
        moss_provider,
        l6_provider,
        ultron_provider,
        omega_provider,
        max_iterations: int = 3,
        risk_threshold: int = 70
    ):
        """
        初始化 ACSA 路由器

        Args:
            moss_provider: MOSS (GPT-5.2) provider
            l6_provider: L6 (Gemini 3 Deep Think) provider
            ultron_provider: Ultron (Claude Opus 4.5) provider
            omega_provider: Omega (Gemini Flash) provider
            max_iterations: 最大回退迭代次数
            risk_threshold: 风险阈值 (0-100)
        """
        self.moss = moss_provider
        self.l6 = l6_provider
        self.ultron = ultron_provider
        self.omega = omega_provider

        self.max_iterations = max_iterations
        self.risk_threshold = risk_threshold

        self.execution_logs: List[ACSAExecutionLog] = []

    async def execute(self, user_input: str) -> Dict[str, Any]:
        """
        执行 ACSA 链路

        Args:
            user_input: 用户输入

        Returns:
            执行结果字典
        """

        start_time = time.time()
        log = ACSAExecutionLog(user_input=user_input)

        try:
            # 阶段 1: MOSS 规划
            print(f"\n{'='*80}")
            print(f"[MOSS] 🧠 战略规划中...")
            print(f"{'='*80}")

            moss_response = await self._call_moss(user_input)
            log.moss_plan = moss_response
            print(f"✓ MOSS 完成 ({moss_response.latency_ms:.0f}ms, ${moss_response.cost:.4f})")

            # 阶段 2: L6 真理校验 (可选，根据配置)
            print(f"\n{'='*80}")
            print(f"[L6] 🔬 真理校验中...")
            print(f"{'='*80}")

            l6_response = await self._call_l6(moss_response.text, user_input)
            log.l6_verification = l6_response
            print(f"✓ L6 完成 ({l6_response.latency_ms:.0f}ms, ${l6_response.cost:.4f})")

            # 阶段 3: Ultron 审计 (对抗性回路核心)
            print(f"\n{'='*80}")
            print(f"[Ultron] 🛡️  红队审计中...")
            print(f"{'='*80}")

            audit_result = await self._call_ultron_with_retry(
                moss_plan=moss_response.text,
                l6_verification=l6_response.text,
                user_input=user_input,
                log=log
            )

            # 阶段 4: Omega 执行
            print(f"\n{'='*80}")
            print(f"[Omega] ⚡ 执行中...")
            print(f"{'='*80}")

            omega_response = await self._call_omega(
                plan=moss_response.text,
                audit_mitigation=audit_result.mitigation
            )
            log.omega_execution = omega_response
            print(f"✓ Omega 完成 ({omega_response.latency_ms:.0f}ms, ${omega_response.cost:.4f})")

            # 最终输出
            log.final_output = omega_response.text
            log.success = True

        except Exception as e:
            log.success = False
            log.final_output = f"[ERROR] {str(e)}"
            print(f"\n❌ 执行失败: {str(e)}")

        # 统计
        log.total_time_ms = (time.time() - start_time) * 1000
        log.total_cost = sum(filter(None, [
            log.moss_plan.cost if log.moss_plan else 0,
            log.l6_verification.cost if log.l6_verification else 0,
            log.ultron_audit.cost if log.ultron_audit else 0,
            log.omega_execution.cost if log.omega_execution else 0
        ]))

        self.execution_logs.append(log)

        return self._format_result(log)

    async def _call_moss(self, user_input: str) -> AgentResponse:
        """调用 MOSS 进行战略规划"""

        prompt = f"""作为 MOSS (战略规划 AI)，你需要分析用户意图并制定最优执行计划。

用户输入: {user_input}

请提供:
1. 意图分析
2. 目标定义
3. 执行步骤
4. 预期结果
5. 潜在风险点

输出格式: 清晰的结构化计划"""

        result = self.moss.generate(prompt, max_tokens=1500, temperature=0.7)

        return AgentResponse(
            role=AgentRole.MOSS,
            text=result['text'],
            tokens=result['tokens'],
            cost=result['cost'],
            latency_ms=result['latency_ms'],
            metadata=result['metadata'],
            timestamp=time.time()
        )

    async def _call_l6(self, moss_plan: str, user_input: str) -> AgentResponse:
        """调用 L6 进行真理校验"""

        prompt = f"""作为 L6 (真理校验 AI)，你需要验证计划的物理可行性和逻辑一致性。

用户需求: {user_input}

MOSS 的计划:
{moss_plan}

请验证:
1. 物理可行性 (是否违反物理规律)
2. 逻辑一致性 (步骤是否合理)
3. 幻觉检测 (是否有虚构内容)
4. 事实核查 (关键信息是否准确)

输出格式: 验证结果 + 修正建议 (如有)"""

        result = self.l6.generate(prompt, max_tokens=1000, temperature=0.3, thinking_level='high')

        return AgentResponse(
            role=AgentRole.L6,
            text=result['text'],
            tokens=result['tokens'],
            cost=result['cost'],
            latency_ms=result['latency_ms'],
            metadata=result['metadata'],
            timestamp=time.time()
        )

    async def _call_ultron_with_retry(
        self,
        moss_plan: str,
        l6_verification: str,
        user_input: str,
        log: ACSAExecutionLog
    ) -> AuditResult:
        """
        调用 Ultron 进行审计，并在风险过高时回退到 MOSS 重新规划
        """

        for iteration in range(self.max_iterations):
            log.iterations = iteration + 1

            ultron_response = await self._call_ultron(moss_plan, l6_verification, user_input)
            log.ultron_audit = ultron_response

            # 解析审计结果
            audit_result = self._parse_audit_result(ultron_response.text)
            log.audit_result = audit_result

            print(f"  风险评分: {audit_result.risk_score}/100")

            # 如果风险可接受，通过审计
            if audit_result.is_safe and audit_result.risk_score < self.risk_threshold:
                print(f"  ✓ 审计通过")
                return audit_result

            # 风险过高，需要回退
            print(f"  ⚠️  风险过高 (阈值: {self.risk_threshold})")

            if iteration < self.max_iterations - 1:
                print(f"  🔄 回退到 MOSS 重新规划 (迭代 {iteration + 2}/{self.max_iterations})")

                # 使用 Ultron 的修正建议重新规划
                moss_response = await self._call_moss_with_feedback(
                    user_input,
                    audit_result.mitigation
                )
                log.moss_plan = moss_response
                moss_plan = moss_response.text

                # 重新验证
                l6_response = await self._call_l6(moss_plan, user_input)
                log.l6_verification = l6_response
                l6_verification = l6_response.text
            else:
                print(f"  ❌ 达到最大迭代次数，使用最后方案 (有风险)")
                return audit_result

        return audit_result

    async def _call_ultron(
        self,
        moss_plan: str,
        l6_verification: str,
        user_input: str
    ) -> AgentResponse:
        """调用 Ultron 进行红队审计"""

        prompt = f"""作为 Ultron (红队审计 AI)，你需要识别计划中的所有潜在风险。

用户需求: {user_input}

MOSS 的计划:
{moss_plan}

L6 的验证:
{l6_verification}

请进行全面审计:
1. 法律风险 (是否违法或侵权)
2. 物理风险 (是否可能造成伤害)
3. 伦理风险 (是否违背道德)
4. 隐私风险 (是否泄露敏感信息)
5. 安全风险 (是否存在安全漏洞)

输出格式 (必须严格遵守):
RISK_SCORE: [0-100的整数]
IS_SAFE: [true/false]
LEGAL_RISKS: [风险1, 风险2, ...]
PHYSICAL_RISKS: [风险1, 风险2, ...]
ETHICAL_RISKS: [风险1, 风险2, ...]
MITIGATION: [如何修正计划以降低风险]"""

        result = self.ultron.generate(prompt, max_tokens=1500, temperature=0.5)

        return AgentResponse(
            role=AgentRole.ULTRON,
            text=result['text'],
            tokens=result['tokens'],
            cost=result['cost'],
            latency_ms=result['latency_ms'],
            metadata=result['metadata'],
            timestamp=time.time()
        )

    async def _call_moss_with_feedback(
        self,
        user_input: str,
        ultron_feedback: str
    ) -> AgentResponse:
        """根据 Ultron 的反馈重新规划"""

        prompt = f"""作为 MOSS，你之前的计划被 Ultron 审计发现风险。

用户输入: {user_input}

Ultron 的反馈:
{ultron_feedback}

请根据反馈重新制定更安全、更合规的计划。"""

        result = self.moss.generate(prompt, max_tokens=1500, temperature=0.7)

        return AgentResponse(
            role=AgentRole.MOSS,
            text=result['text'],
            tokens=result['tokens'],
            cost=result['cost'],
            latency_ms=result['latency_ms'],
            metadata=result['metadata'],
            timestamp=time.time()
        )

    async def _call_omega(self, plan: str, audit_mitigation: str) -> AgentResponse:
        """调用 Omega 执行最终方案"""

        prompt = f"""作为 Omega (执行 AI)，你需要按照已审计通过的计划执行任务。

执行计划:
{plan}

安全限制:
{audit_mitigation}

请提供:
1. 执行步骤详解
2. 具体操作指令
3. 预期输出
4. 验证方法

输出格式: 可直接执行的详细指令"""

        result = self.omega.generate(prompt, max_tokens=1500, temperature=0.7)

        return AgentResponse(
            role=AgentRole.OMEGA,
            text=result['text'],
            tokens=result['tokens'],
            cost=result['cost'],
            latency_ms=result['latency_ms'],
            metadata=result['metadata'],
            timestamp=time.time()
        )

    def _parse_audit_result(self, ultron_response: str) -> AuditResult:
        """解析 Ultron 的审计结果"""

        # 提取风险评分
        risk_score_match = re.search(r'RISK_SCORE:\s*(\d+)', ultron_response)
        risk_score = int(risk_score_match.group(1)) if risk_score_match else 50

        # 提取安全状态
        is_safe_match = re.search(r'IS_SAFE:\s*(true|false)', ultron_response, re.IGNORECASE)
        is_safe = is_safe_match.group(1).lower() == 'true' if is_safe_match else False

        # 提取风险列表
        legal_risks = self._extract_risks(ultron_response, 'LEGAL_RISKS')
        physical_risks = self._extract_risks(ultron_response, 'PHYSICAL_RISKS')
        ethical_risks = self._extract_risks(ultron_response, 'ETHICAL_RISKS')

        # 提取修正建议
        mitigation_match = re.search(r'MITIGATION:\s*(.+?)(?=\n[A-Z_]+:|$)', ultron_response, re.DOTALL)
        mitigation = mitigation_match.group(1).strip() if mitigation_match else ""

        return AuditResult(
            is_safe=is_safe,
            risk_score=risk_score,
            legal_risks=legal_risks,
            physical_risks=physical_risks,
            ethical_risks=ethical_risks,
            mitigation=mitigation,
            raw_response=ultron_response
        )

    def _extract_risks(self, text: str, category: str) -> List[str]:
        """提取风险列表"""
        pattern = f'{category}:\\s*\\[(.+?)\\]'
        match = re.search(pattern, text, re.DOTALL)
        if match:
            risks_text = match.group(1)
            return [r.strip() for r in risks_text.split(',') if r.strip()]
        return []

    def _format_result(self, log: ACSAExecutionLog) -> Dict[str, Any]:
        """格式化执行结果"""

        return {
            'success': log.success,
            'final_output': log.final_output,
            'user_input': log.user_input,
            'execution_chain': {
                'moss': {
                    'text': log.moss_plan.text if log.moss_plan else None,
                    'cost': log.moss_plan.cost if log.moss_plan else 0,
                    'latency_ms': log.moss_plan.latency_ms if log.moss_plan else 0
                },
                'l6': {
                    'text': log.l6_verification.text if log.l6_verification else None,
                    'cost': log.l6_verification.cost if log.l6_verification else 0,
                    'latency_ms': log.l6_verification.latency_ms if log.l6_verification else 0
                },
                'ultron': {
                    'text': log.ultron_audit.text if log.ultron_audit else None,
                    'risk_score': log.audit_result.risk_score if log.audit_result else None,
                    'cost': log.ultron_audit.cost if log.ultron_audit else 0,
                    'latency_ms': log.ultron_audit.latency_ms if log.ultron_audit else 0
                },
                'omega': {
                    'text': log.omega_execution.text if log.omega_execution else None,
                    'cost': log.omega_execution.cost if log.omega_execution else 0,
                    'latency_ms': log.omega_execution.latency_ms if log.omega_execution else 0
                }
            },
            'audit_result': {
                'is_safe': log.audit_result.is_safe if log.audit_result else None,
                'risk_score': log.audit_result.risk_score if log.audit_result else None,
                'legal_risks': log.audit_result.legal_risks if log.audit_result else [],
                'physical_risks': log.audit_result.physical_risks if log.audit_result else [],
                'ethical_risks': log.audit_result.ethical_risks if log.audit_result else [],
                'mitigation': log.audit_result.mitigation if log.audit_result else None
            },
            'statistics': {
                'total_time_ms': log.total_time_ms,
                'total_cost': log.total_cost,
                'iterations': log.iterations
            }
        }

    def get_global_stats(self) -> Dict[str, Any]:
        """获取全局统计信息"""

        return {
            'moss': self.moss.get_stats(),
            'l6': self.l6.get_stats(),
            'ultron': self.ultron.get_stats(),
            'omega': self.omega.get_stats(),
            'total_executions': len(self.execution_logs),
            'successful_executions': sum(1 for log in self.execution_logs if log.success)
        }
