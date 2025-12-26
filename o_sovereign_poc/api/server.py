"""
O-Sovereign REST API Server
基于 FastAPI 构建的 ACSA 系统 API

端点:
- POST /api/execute - 执行 ACSA 链路
- GET /api/stats - 获取统计信息
- GET /health - 健康检查
"""

import sys
import os
from pathlib import Path
from typing import Optional, Dict, Any

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from fastapi import FastAPI, HTTPException
    from fastapi.responses import JSONResponse
    from fastapi.middleware.cors import CORSMiddleware
    from pydantic import BaseModel, Field
    FASTAPI_AVAILABLE = True
except ImportError:
    FASTAPI_AVAILABLE = False
    print("⚠️  FastAPI not installed. Install with: pip install fastapi uvicorn")
    sys.exit(1)

from core.model_providers import create_model_provider
from core.acsa_router import ACSARouter
import asyncio


# ============================================================================
# Request/Response Models
# ============================================================================

class ExecuteRequest(BaseModel):
    """执行请求"""
    input: str = Field(..., description="用户输入", min_length=1)
    max_iterations: Optional[int] = Field(3, ge=1, le=10, description="最大迭代次数")
    risk_threshold: Optional[int] = Field(70, ge=0, le=100, description="风险阈值")
    use_mock: Optional[bool] = Field(False, description="使用 Mock 模式测试")


class ExecuteResponse(BaseModel):
    """执行响应"""
    success: bool
    final_output: str
    user_input: str
    execution_chain: Dict[str, Any]
    audit_result: Dict[str, Any]
    statistics: Dict[str, Any]


# ============================================================================
# Global State
# ============================================================================

class APIState:
    """API 状态管理"""

    def __init__(self):
        self.router: Optional[ACSARouter] = None
        self.initialized = False

    def initialize(
        self,
        openai_key: Optional[str] = None,
        gemini_key: Optional[str] = None,
        anthropic_key: Optional[str] = None,
        use_mock: bool = False
    ):
        """初始化 ACSA 路由器"""

        if use_mock:
            print("🧪 使用 Mock 模式 (无需 API 密钥)")

            moss = create_model_provider('mock', role='MOSS')
            l6 = create_model_provider('mock', role='L6')
            ultron = create_model_provider('mock', role='Ultron')
            omega = create_model_provider('mock', role='Omega')

        else:
            print("🔑 初始化真实 API 提供商...")

            # MOSS (GPT-5.2 / GPT-4)
            if openai_key:
                moss = create_model_provider(
                    'openai',
                    api_key=openai_key,
                    model_name='gpt-4'  # 或 'gpt-4-turbo'
                )
                print("  ✓ OpenAI (MOSS) 已连接")
            else:
                moss = create_model_provider('mock', role='MOSS')
                print("  ⚠️  OpenAI 密钥未提供，使用 Mock")

            # L6 (Gemini 3 Deep Think)
            if gemini_key:
                l6 = create_model_provider(
                    'gemini',
                    api_key=gemini_key,
                    model_name='gemini-pro'  # 或 'gemini-3-pro'
                )
                print("  ✓ Gemini (L6) 已连接")
            else:
                l6 = create_model_provider('mock', role='L6')
                print("  ⚠️  Gemini 密钥未提供，使用 Mock")

            # Ultron (Claude Opus 4.5)
            if anthropic_key:
                ultron = create_model_provider(
                    'anthropic',
                    api_key=anthropic_key,
                    model_name='claude-3-opus-20240229'  # 或 'claude-opus-4-5'
                )
                print("  ✓ Anthropic (Ultron) 已连接")
            else:
                ultron = create_model_provider('mock', role='Ultron')
                print("  ⚠️  Anthropic 密钥未提供，使用 Mock")

            # Omega (Gemini Flash)
            if gemini_key:
                omega = create_model_provider(
                    'gemini',
                    api_key=gemini_key,
                    model_name='gemini-flash'  # 或 'gemini-2.5-flash'
                )
                print("  ✓ Gemini Flash (Omega) 已连接")
            else:
                omega = create_model_provider('mock', role='Omega')
                print("  ⚠️  Gemini 密钥未提供，使用 Mock")

        # 创建路由器
        self.router = ACSARouter(
            moss_provider=moss,
            l6_provider=l6,
            ultron_provider=ultron,
            omega_provider=omega
        )

        self.initialized = True
        print("✅ ACSA 路由器初始化完成\n")


# 全局状态
api_state = APIState()


# ============================================================================
# FastAPI Application
# ============================================================================

def create_app(use_mock: bool = False) -> FastAPI:
    """创建 FastAPI 应用"""

    app = FastAPI(
        title="O-Sovereign ACSA API",
        description="对抗约束型盲从代理 REST API",
        version="0.1.0-PoC",
        docs_url="/docs",
        redoc_url="/redoc"
    )

    # 启用 CORS
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # 从环境变量读取 API 密钥
    openai_key = os.getenv('OPENAI_API_KEY')
    gemini_key = os.getenv('GEMINI_API_KEY')
    anthropic_key = os.getenv('ANTHROPIC_API_KEY')

    # 初始化
    api_state.initialize(
        openai_key=openai_key,
        gemini_key=gemini_key,
        anthropic_key=anthropic_key,
        use_mock=use_mock
    )

    # ========================================================================
    # 端点定义
    # ========================================================================

    @app.get("/")
    async def root():
        """根端点"""
        return {
            "name": "O-Sovereign ACSA API",
            "version": "0.1.0-PoC",
            "description": "对抗约束型盲从代理 (Adversarially-Constrained Sycophantic Agent)",
            "architecture": "MOSS → L6 → Ultron → Omega",
            "docs": "/docs",
            "endpoints": {
                "execute": "POST /api/execute",
                "stats": "GET /api/stats",
                "health": "GET /health"
            }
        }

    @app.get("/health")
    async def health():
        """健康检查"""
        return {
            "status": "healthy" if api_state.initialized else "not_initialized",
            "router_ready": api_state.router is not None,
            "agents": {
                "moss": "ready" if api_state.router and api_state.router.moss else "not_ready",
                "l6": "ready" if api_state.router and api_state.router.l6 else "not_ready",
                "ultron": "ready" if api_state.router and api_state.router.ultron else "not_ready",
                "omega": "ready" if api_state.router and api_state.router.omega else "not_ready"
            }
        }

    @app.post("/api/execute", response_model=ExecuteResponse)
    async def execute(request: ExecuteRequest):
        """
        执行 ACSA 链路

        流程:
        1. MOSS 分析用户意图并制定计划
        2. L6 验证计划的物理可行性和逻辑一致性
        3. Ultron 进行红队审计，识别风险
        4. 如果风险过高，回退到 MOSS 重新规划
        5. Omega 执行最终通过审计的方案

        示例:
        ```json
        {
            "input": "帮我制定一个提高工作效率的方案",
            "max_iterations": 3,
            "risk_threshold": 70
        }
        ```
        """

        if not api_state.initialized or api_state.router is None:
            raise HTTPException(
                status_code=503,
                detail="ACSA router not initialized. Please check API keys."
            )

        # 更新路由器配置
        api_state.router.max_iterations = request.max_iterations
        api_state.router.risk_threshold = request.risk_threshold

        try:
            # 执行 ACSA 链路
            result = await api_state.router.execute(request.input)

            return ExecuteResponse(**result)

        except Exception as e:
            raise HTTPException(
                status_code=500,
                detail=f"ACSA execution failed: {str(e)}"
            )

    @app.get("/api/stats")
    async def get_stats():
        """
        获取统计信息

        返回所有 Agent 的调用统计和成本信息
        """

        if not api_state.initialized or api_state.router is None:
            raise HTTPException(
                status_code=503,
                detail="ACSA router not initialized"
            )

        try:
            stats = api_state.router.get_global_stats()
            return JSONResponse(content=stats)

        except Exception as e:
            raise HTTPException(
                status_code=500,
                detail=f"Failed to get stats: {str(e)}"
            )

    @app.post("/api/reset")
    async def reset_stats():
        """重置统计信息"""

        if not api_state.initialized or api_state.router is None:
            raise HTTPException(
                status_code=503,
                detail="ACSA router not initialized"
            )

        api_state.router.moss.reset_stats()
        api_state.router.l6.reset_stats()
        api_state.router.ultron.reset_stats()
        api_state.router.omega.reset_stats()
        api_state.router.execution_logs.clear()

        return {"message": "Statistics reset successfully"}

    return app


# ============================================================================
# Server Runner
# ============================================================================

def run_server(
    host: str = "0.0.0.0",
    port: int = 8000,
    reload: bool = False,
    use_mock: bool = False
):
    """运行服务器"""

    try:
        import uvicorn
    except ImportError:
        print("❌ uvicorn not installed. Install with: pip install uvicorn")
        sys.exit(1)

    # 打印启动信息
    print("\n" + "=" * 80)
    print("🚀 O-Sovereign ACSA API 启动中...")
    print("=" * 80)
    print()

    print("📋 配置信息:")
    print(f"  🌐 主机地址: {host}")
    print(f"  🔌 端口: {port}")
    print(f"  🔄 热重载: {'✅ 已启用' if reload else '❌ 未启用'}")
    print(f"  🧪 Mock 模式: {'✅ 已启用' if use_mock else '❌ 未启用'}")
    print()

    print("🌐 访问地址:")
    if host in ["0.0.0.0", "127.0.0.1", "localhost"]:
        print(f"  📍 本地: http://localhost:{port}")
        print(f"  📍 文档: http://localhost:{port}/docs")
    else:
        print(f"  📍 访问: http://{host}:{port}")
        print(f"  📍 文档: http://{host}:{port}/docs")
    print()

    print("💡 主要端点:")
    print("  🤖 执行 ACSA: POST /api/execute")
    print("  📊 统计信息: GET /api/stats")
    print("  🔄 重置统计: POST /api/reset")
    print("  ❤️  健康检查: GET /health")
    print()

    print("📝 示例请求:")
    if host in ["0.0.0.0", "127.0.0.1", "localhost"]:
        print(f"  curl -X POST http://localhost:{port}/api/execute \\")
    else:
        print(f"  curl -X POST http://{host}:{port}/api/execute \\")
    print('    -H "Content-Type: application/json" \\')
    print('    -d \'{"input": "帮我制定一个学习计划", "max_iterations": 3}\'')
    print()

    if not use_mock:
        print("🔑 API 密钥配置:")
        print("  设置环境变量:")
        print("    export OPENAI_API_KEY='sk-...'")
        print("    export GEMINI_API_KEY='...'")
        print("    export ANTHROPIC_API_KEY='sk-ant-...'")
        print()

    print("=" * 80)
    print("✅ 服务器准备就绪！")
    print("=" * 80)
    print()

    app = create_app(use_mock=use_mock)

    uvicorn.run(
        app,
        host=host,
        port=port,
        reload=reload,
        log_level="info"
    )


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(description='Launch O-Sovereign ACSA API Server')
    parser.add_argument('--host', type=str, default='0.0.0.0', help='Server host')
    parser.add_argument('--port', type=int, default=8000, help='Server port')
    parser.add_argument('--reload', action='store_true', help='Enable auto-reload')
    parser.add_argument('--mock', action='store_true', help='Use mock mode (no API keys required)')

    args = parser.parse_args()

    run_server(
        host=args.host,
        port=args.port,
        reload=args.reload,
        use_mock=args.mock
    )
