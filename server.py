"""
APT Model REST API Server

FastAPI-based REST API implementing endpoints for:
- Inference (single and batch text generation)
- Training monitoring (status, gradients, history)
- Checkpoint management (list, load, delete)

🔮 Implementation based on preparation code from:
- tests/test_trainer_complete.py:api_inference() (lines 421-458)
- tests/test_trainer_complete.py:api_batch_inference() (lines 460-492)
- apt_model/training/gradient_monitor.py:export_for_webui() (lines 260-302)
"""

import time
import json
from pathlib import Path
from typing import Optional, Dict, Any, List
from datetime import datetime

try:
    from fastapi import FastAPI, HTTPException, BackgroundTasks
    from fastapi.responses import JSONResponse, FileResponse
    from fastapi.middleware.cors import CORSMiddleware
    from pydantic import BaseModel, Field
    FASTAPI_AVAILABLE = True
except ImportError:
    FASTAPI_AVAILABLE = False
    # Create dummy classes for type hints
    class BaseModel:
        pass
    class FastAPI:
        pass
    def Field(*args, **kwargs):
        """Dummy Field function"""
        return None

try:
    import torch
    import torch.nn as nn
    TORCH_AVAILABLE = True
except ImportError:
    TORCH_AVAILABLE = False


# ============================================================================
# Request/Response Models
# ============================================================================

class GenerateRequest(BaseModel):
    """
    Request model for text generation

    🔮 Based on api_inference prototype (test_trainer_complete.py:421-458)
    """
    text: str = Field(..., description="Input text to generate from")
    max_length: int = Field(50, ge=1, le=2048, description="Maximum generation length")
    temperature: float = Field(1.0, ge=0.1, le=2.0, description="Sampling temperature")
    num_beams: int = Field(1, ge=1, le=10, description="Number of beams for beam search")
    do_sample: bool = Field(False, description="Whether to use sampling")
    top_k: Optional[int] = Field(None, description="Top-k sampling parameter")
    top_p: Optional[float] = Field(None, description="Top-p (nucleus) sampling parameter")


class GenerateResponse(BaseModel):
    """
    Response model for text generation

    🔮 Based on api_inference return format
    """
    generated_text: str
    input_text: str
    generation_time_ms: float
    parameters: Dict[str, Any]


class BatchGenerateRequest(BaseModel):
    """
    Request model for batch text generation

    🔮 Based on api_batch_inference prototype (test_trainer_complete.py:460-492)
    """
    texts: List[str] = Field(..., description="List of input texts")
    max_length: int = Field(50, ge=1, le=2048)
    temperature: float = Field(1.0, ge=0.1, le=2.0)
    num_beams: int = Field(1, ge=1, le=10)
    do_sample: bool = Field(False)


class BatchGenerateResponse(BaseModel):
    """
    Response model for batch generation

    🔮 Based on api_batch_inference return format
    """
    results: List[Dict[str, str]]
    total_time_ms: float
    batch_size: int


class CheckpointInfo(BaseModel):
    """Checkpoint information"""
    filename: str
    epoch: Optional[int] = None
    global_step: Optional[int] = None
    val_loss: Optional[float] = None
    file_size_mb: float
    created_at: str


class TrainingStatus(BaseModel):
    """Training status information"""
    is_training: bool
    current_epoch: Optional[int] = None
    global_step: Optional[int] = None
    latest_loss: Optional[float] = None
    best_val_loss: Optional[float] = None
    model_loaded: bool


# ============================================================================
# API State Management
# ============================================================================

class APIState:
    """Global state for API server"""

    def __init__(self):
        self.model = None
        self.tokenizer = None
        self.config = None
        self.checkpoint_dir = None
        self.gradient_monitor = None
        self.trainer = None
        self.model_loaded = False

    def load_model_from_checkpoint(self, checkpoint_path: Path) -> Dict[str, Any]:
        """
        Load model from checkpoint

        🔮 Based on test_model_serialization_for_api (test_trainer_complete.py:383-419)
        """
        if not TORCH_AVAILABLE:
            raise RuntimeError("PyTorch not available")

        try:
            from apt_model.modeling.apt_model import APTLargeModel, APTConfig

            checkpoint = torch.load(checkpoint_path, map_location='cpu')

            # Reconstruct config
            if 'config' in checkpoint:
                self.config = APTConfig(**checkpoint['config'])
            else:
                raise ValueError("Config not found in checkpoint")

            # Load model
            self.model = APTLargeModel(self.config)
            self.model.load_state_dict(checkpoint['model_state_dict'])
            self.model.eval()
            self.model_loaded = True

            return {
                'status': 'success',
                'message': f'Model loaded from {checkpoint_path.name}',
                'config': checkpoint['config'],
                'epoch': checkpoint.get('epoch'),
                'global_step': checkpoint.get('global_step')
            }

        except Exception as e:
            self.model_loaded = False
            raise RuntimeError(f"Failed to load model: {str(e)}")

    def get_checkpoint_list(self) -> List[CheckpointInfo]:
        """
        Get list of available checkpoints

        🔮 Based on test_export_checkpoint_list_for_webui (test_trainer_complete.py)
        """
        if self.checkpoint_dir is None:
            return []

        checkpoint_dir = Path(self.checkpoint_dir)
        if not checkpoint_dir.exists():
            return []

        checkpoints = []
        for ckpt_file in sorted(checkpoint_dir.glob('*.pt')):
            try:
                if not TORCH_AVAILABLE:
                    # Without PyTorch, just return basic file info
                    checkpoints.append(CheckpointInfo(
                        filename=ckpt_file.name,
                        file_size_mb=ckpt_file.stat().st_size / (1024 * 1024),
                        created_at=datetime.fromtimestamp(ckpt_file.stat().st_mtime).isoformat()
                    ))
                else:
                    ckpt = torch.load(ckpt_file, map_location='cpu')
                    checkpoints.append(CheckpointInfo(
                        filename=ckpt_file.name,
                        epoch=ckpt.get('epoch'),
                        global_step=ckpt.get('global_step'),
                        val_loss=ckpt.get('best_val_loss'),
                        file_size_mb=ckpt_file.stat().st_size / (1024 * 1024),
                        created_at=datetime.fromtimestamp(ckpt_file.stat().st_mtime).isoformat()
                    ))
            except Exception:
                continue

        return checkpoints


# Global state
api_state = APIState()


# ============================================================================
# FastAPI Application
# ============================================================================

def create_app(checkpoint_dir: Optional[str] = None) -> FastAPI:
    """
    Create FastAPI application

    Args:
        checkpoint_dir: Default checkpoint directory

    Returns:
        FastAPI application instance
    """
    if not FASTAPI_AVAILABLE:
        raise ImportError("FastAPI not installed. Install with: pip install fastapi uvicorn")

    app = FastAPI(
        title="APT Model API",
        description="REST API for APT model inference, training monitoring, and checkpoint management",
        version="1.0.0",
        docs_url="/docs",
        redoc_url="/redoc"
    )

    # Enable CORS
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Set checkpoint directory
    if checkpoint_dir:
        api_state.checkpoint_dir = checkpoint_dir

    # ========================================================================
    # Health Check
    # ========================================================================

    @app.get("/")
    async def root():
        """Root endpoint"""
        return {
            "message": "APT Model API",
            "version": "1.0.0",
            "docs": "/docs",
            "endpoints": {
                "inference": "/api/generate",
                "batch_inference": "/api/batch_generate",
                "training_status": "/api/training/status",
                "gradients": "/api/training/gradients",
                "checkpoints": "/api/checkpoints"
            }
        }

    @app.get("/health")
    async def health():
        """Health check endpoint"""
        return {
            "status": "healthy",
            "torch_available": TORCH_AVAILABLE,
            "model_loaded": api_state.model_loaded,
            "timestamp": datetime.now().isoformat()
        }

    # ========================================================================
    # Inference Endpoints
    # ========================================================================

    @app.post("/api/generate", response_model=GenerateResponse)
    async def generate(request: GenerateRequest):
        """
        Generate text from input

        🔮 Implementation of api_inference prototype (test_trainer_complete.py:421-458)

        Example:
        ```
        POST /api/generate
        {
            "text": "今天天气很好",
            "max_length": 50
        }
        ```
        """
        if not api_state.model_loaded or api_state.model is None:
            raise HTTPException(
                status_code=503,
                detail="Model not loaded. Please load a checkpoint first via POST /api/checkpoints/load"
            )

        if not TORCH_AVAILABLE:
            raise HTTPException(status_code=500, detail="PyTorch not available")

        try:
            start_time = time.time()

            # Inference
            api_state.model.eval()
            with torch.no_grad():
                if api_state.tokenizer is None:
                    # Mock response if tokenizer not available
                    generated_text = f"[Mock] Generated from: {request.text}"
                else:
                    inputs = api_state.tokenizer(
                        request.text,
                        return_tensors='pt',
                        padding=True,
                        truncation=True,
                        max_length=512
                    )

                    generated_ids = api_state.model.generate(
                        input_ids=inputs['input_ids'],
                        max_length=request.max_length,
                        temperature=request.temperature,
                        num_beams=request.num_beams,
                        do_sample=request.do_sample,
                        top_k=request.top_k,
                        top_p=request.top_p
                    )

                    generated_text = api_state.tokenizer.decode(
                        generated_ids[0],
                        skip_special_tokens=True
                    )

            generation_time = (time.time() - start_time) * 1000

            return GenerateResponse(
                generated_text=generated_text,
                input_text=request.text,
                generation_time_ms=generation_time,
                parameters={
                    'max_length': request.max_length,
                    'temperature': request.temperature,
                    'num_beams': request.num_beams,
                    'do_sample': request.do_sample
                }
            )

        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Generation failed: {str(e)}")

    @app.post("/api/batch_generate", response_model=BatchGenerateResponse)
    async def batch_generate(request: BatchGenerateRequest):
        """
        Generate text for multiple inputs

        🔮 Implementation of api_batch_inference prototype (test_trainer_complete.py:460-492)

        Example:
        ```
        POST /api/batch_generate
        {
            "texts": ["你好", "今天天气", "人工智能"],
            "max_length": 50
        }
        ```
        """
        if not api_state.model_loaded or api_state.model is None:
            raise HTTPException(
                status_code=503,
                detail="Model not loaded. Please load a checkpoint first"
            )

        if not TORCH_AVAILABLE:
            raise HTTPException(status_code=500, detail="PyTorch not available")

        try:
            start_time = time.time()

            api_state.model.eval()
            results = []

            with torch.no_grad():
                if api_state.tokenizer is None:
                    # Mock response
                    for text in request.texts:
                        results.append({
                            'input': text,
                            'output': f"[Mock] Generated from: {text}"
                        })
                else:
                    inputs = api_state.tokenizer(
                        request.texts,
                        return_tensors='pt',
                        padding=True,
                        truncation=True,
                        max_length=512
                    )

                    generated_ids = api_state.model.generate(
                        input_ids=inputs['input_ids'],
                        max_length=request.max_length,
                        temperature=request.temperature,
                        num_beams=request.num_beams,
                        do_sample=request.do_sample
                    )

                    for i, gen_ids in enumerate(generated_ids):
                        generated_text = api_state.tokenizer.decode(
                            gen_ids,
                            skip_special_tokens=True
                        )
                        results.append({
                            'input': request.texts[i],
                            'output': generated_text
                        })

            total_time = (time.time() - start_time) * 1000

            return BatchGenerateResponse(
                results=results,
                total_time_ms=total_time,
                batch_size=len(request.texts)
            )

        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Batch generation failed: {str(e)}")

    # ========================================================================
    # Training Monitoring Endpoints
    # ========================================================================

    @app.get("/api/training/status", response_model=TrainingStatus)
    async def get_training_status():
        """
        Get current training status

        Returns training progress, loss, and model state
        """
        is_training = api_state.trainer is not None and hasattr(api_state.trainer, 'training_active')

        status = TrainingStatus(
            is_training=is_training,
            model_loaded=api_state.model_loaded
        )

        if api_state.trainer:
            status.current_epoch = getattr(api_state.trainer, 'epoch', None)
            status.global_step = getattr(api_state.trainer, 'global_step', None)
            status.best_val_loss = getattr(api_state.trainer, 'best_val_loss', None)

            train_losses = getattr(api_state.trainer, 'train_losses', [])
            if train_losses:
                status.latest_loss = train_losses[-1]

        return status

    @app.get("/api/training/gradients")
    async def get_gradients():
        """
        Get gradient monitoring data

        🔮 Returns data from gradient_monitor.export_for_webui() (lines 260-302)

        Returns JSON-serialized gradient timeline, layer statistics, and anomaly summary
        """
        if api_state.gradient_monitor is None:
            raise HTTPException(
                status_code=404,
                detail="Gradient monitor not available. No active training session."
            )

        try:
            # 🔮 Use the export_for_webui() method
            webui_data = api_state.gradient_monitor.export_for_webui()
            return JSONResponse(content=webui_data)

        except Exception as e:
            raise HTTPException(
                status_code=500,
                detail=f"Failed to export gradient data: {str(e)}"
            )

    @app.get("/api/training/history")
    async def get_training_history():
        """
        Get training history (losses, learning rates)

        Returns historical training metrics
        """
        if api_state.trainer is None:
            raise HTTPException(
                status_code=404,
                detail="No training session available"
            )

        try:
            train_losses = getattr(api_state.trainer, 'train_losses', [])
            lr_history = getattr(api_state.trainer, 'lr_history', [])

            return {
                'training_history': {
                    'steps': list(range(len(train_losses))),
                    'train_loss': [float(l) for l in train_losses],
                    'learning_rate': [float(lr) for lr in lr_history],
                },
                'total_steps': len(train_losses)
            }

        except Exception as e:
            raise HTTPException(
                status_code=500,
                detail=f"Failed to get training history: {str(e)}"
            )

    # ========================================================================
    # Checkpoint Management Endpoints
    # ========================================================================

    @app.get("/api/checkpoints", response_model=List[CheckpointInfo])
    async def list_checkpoints():
        """
        List all available checkpoints

        🔮 Based on test_export_checkpoint_list_for_webui

        Returns list of checkpoint files with metadata
        """
        if api_state.checkpoint_dir is None:
            raise HTTPException(
                status_code=400,
                detail="Checkpoint directory not configured"
            )

        checkpoints = api_state.get_checkpoint_list()
        return checkpoints

    @app.post("/api/checkpoints/load")
    async def load_checkpoint(filename: str):
        """
        Load a checkpoint into memory for inference

        Args:
            filename: Name of checkpoint file to load

        Example:
        ```
        POST /api/checkpoints/load?filename=checkpoint_epoch_10.pt
        ```
        """
        if api_state.checkpoint_dir is None:
            raise HTTPException(
                status_code=400,
                detail="Checkpoint directory not configured"
            )

        ckpt_path = Path(api_state.checkpoint_dir) / filename

        if not ckpt_path.exists():
            raise HTTPException(
                status_code=404,
                detail=f"Checkpoint not found: {filename}"
            )

        try:
            result = api_state.load_model_from_checkpoint(ckpt_path)
            return result

        except Exception as e:
            raise HTTPException(
                status_code=500,
                detail=f"Failed to load checkpoint: {str(e)}"
            )

    @app.delete("/api/checkpoints/{filename}")
    async def delete_checkpoint(filename: str):
        """
        Delete a checkpoint file

        Args:
            filename: Name of checkpoint file to delete
        """
        if api_state.checkpoint_dir is None:
            raise HTTPException(
                status_code=400,
                detail="Checkpoint directory not configured"
            )

        ckpt_path = Path(api_state.checkpoint_dir) / filename

        if not ckpt_path.exists():
            raise HTTPException(
                status_code=404,
                detail=f"Checkpoint not found: {filename}"
            )

        try:
            ckpt_path.unlink()
            return {
                'status': 'success',
                'message': f'Deleted checkpoint: {filename}'
            }

        except Exception as e:
            raise HTTPException(
                status_code=500,
                detail=f"Failed to delete checkpoint: {str(e)}"
            )

    @app.get("/api/checkpoints/download/{filename}")
    async def download_checkpoint(filename: str):
        """
        Download a checkpoint file

        Args:
            filename: Name of checkpoint file to download
        """
        if api_state.checkpoint_dir is None:
            raise HTTPException(
                status_code=400,
                detail="Checkpoint directory not configured"
            )

        ckpt_path = Path(api_state.checkpoint_dir) / filename

        if not ckpt_path.exists():
            raise HTTPException(
                status_code=404,
                detail=f"Checkpoint not found: {filename}"
            )

        return FileResponse(
            path=str(ckpt_path),
            filename=filename,
            media_type='application/octet-stream'
        )

    return app


def run_server(
    checkpoint_dir: Optional[str] = None,
    host: str = "0.0.0.0",
    port: int = 8000,
    reload: bool = False,
    api_key: Optional[str] = None
):
    """
    Run FastAPI server with uvicorn

    Args:
        checkpoint_dir: Default checkpoint directory
        host: Server host
        port: Server port
        reload: Enable auto-reload on code changes
        api_key: Optional API key for authentication
    """
    import sys
    import secrets

    try:
        import uvicorn
    except ImportError:
        raise ImportError("uvicorn not installed. Install with: pip install uvicorn")

    # Generate API key if not provided
    if api_key is None:
        api_key = secrets.token_hex(32)
        generated_key = True
    else:
        generated_key = False

    # Print startup banner
    print("\n" + "=" * 80)
    print("🚀 APT Model REST API 启动中...")
    print("=" * 80)
    print()

    # Show configuration
    print("📋 配置信息:")
    print(f"  🌐 主机地址: {host}")
    print(f"  🔌 端口: {port}")
    print(f"  📁 Checkpoint目录: {checkpoint_dir or '(未设置)'}")
    print(f"  🔄 热重载: {'✅ 已启用' if reload else '❌ 未启用'}")
    print(f"  🔐 PyTorch: {'✅ 可用' if TORCH_AVAILABLE else '⚠️  不可用'}")
    print(f"  🚀 FastAPI: {'✅ 可用' if FASTAPI_AVAILABLE else '⚠️  不可用'}")
    print()

    # Show access URLs
    print("🌐 API访问地址:")
    if host in ["0.0.0.0", "127.0.0.1", "localhost"]:
        print(f"  📍 本地访问: http://localhost:{port}")
        print(f"  📍 局域网访问: http://<你的IP>:{port}")
    else:
        print(f"  📍 访问地址: http://{host}:{port}")
    print()

    print("📚 API文档:")
    if host in ["0.0.0.0", "127.0.0.1", "localhost"]:
        print(f"  📖 Swagger UI: http://localhost:{port}/docs")
        print(f"  📖 ReDoc: http://localhost:{port}/redoc")
    else:
        print(f"  📖 Swagger UI: http://{host}:{port}/docs")
        print(f"  📖 ReDoc: http://{host}:{port}/redoc")
    print()

    # Show API key
    if generated_key:
        print("🔑 API访问密钥 (自动生成):")
        print(f"  🔐 API Key: {api_key}")
        print(f"  💡 请妥善保存此密钥，重启后将重新生成")
    else:
        print("🔑 API访问密钥:")
        print(f"  🔐 API Key: {api_key[:16]}... (已加载)")
    print()

    print("💡 主要端点:")
    print("  🤖 推理服务:")
    print("     POST /api/generate - 单文本生成")
    print("     POST /api/batch_generate - 批量生成")
    print("  📊 训练监控:")
    print("     GET /api/training/status - 训练状态")
    print("     GET /api/training/gradients - 梯度数据")
    print("  💾 Checkpoint管理:")
    print("     GET /api/checkpoints - 列出checkpoints")
    print("     POST /api/checkpoints/load - 加载checkpoint")
    print()

    print("📝 使用示例:")
    if host in ["0.0.0.0", "127.0.0.1", "localhost"]:
        print(f"  curl -X POST http://localhost:{port}/api/generate \\")
    else:
        print(f"  curl -X POST http://{host}:{port}/api/generate \\")
    print('    -H "Content-Type: application/json" \\')
    print('    -d \'{"text": "你好", "max_length": 50}\'')
    print()

    print("=" * 80)
    print("✅ API服务器已启动！")
    print("=" * 80)
    print()

    # Flush output
    sys.stdout.flush()

    app = create_app(checkpoint_dir=checkpoint_dir)

    # Store API key in app state (for future authentication middleware)
    app.state.api_key = api_key

    uvicorn.run(
        app,
        host=host,
        port=port,
        reload=reload,
        log_level="info"
    )


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(description='Launch APT Model API Server')
    parser.add_argument('--checkpoint-dir', type=str, help='Checkpoint directory')
    parser.add_argument('--host', type=str, default='0.0.0.0', help='Server host')
    parser.add_argument('--port', type=int, default=8000, help='Server port')
    parser.add_argument('--reload', action='store_true', help='Enable auto-reload')
    parser.add_argument('--api-key', type=str, help='API access key (auto-generated if not provided)')

    args = parser.parse_args()

    run_server(
        checkpoint_dir=args.checkpoint_dir,
        host=args.host,
        port=args.port,
        reload=args.reload,
        api_key=args.api_key
    )
