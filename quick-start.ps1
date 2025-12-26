# ACSA 一键启动脚本 (PowerShell)
# Quick Start Script for ACSA - PowerShell Version

# 设置输出编码为UTF-8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   ACSA 一键启动脚本 (PowerShell)" -ForegroundColor Cyan
Write-Host "   Quick Start Script" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

# 颜色函数
function Write-Success { param($msg) Write-Host "✓ $msg" -ForegroundColor Green }
function Write-Error { param($msg) Write-Host "✗ $msg" -ForegroundColor Red }
function Write-Warning { param($msg) Write-Host "⚠ $msg" -ForegroundColor Yellow }
function Write-Info { param($msg) Write-Host "$msg" -ForegroundColor Blue }

# 检查命令函数
function Test-Command {
    param($CommandName)
    $null = Get-Command $CommandName -ErrorAction SilentlyContinue
    return $?
}

# 步骤1：检查环境
Write-Info "[1/5] 检查环境依赖..."
Write-Host ""

$missingDeps = $false

# 检查 Rust
if (Test-Command "rustc") {
    Write-Success "Rust 已安装"
    $rustVersion = rustc --version
    Write-Host "      版本: $rustVersion" -ForegroundColor Gray
} else {
    Write-Error "Rust 未安装"
    Write-Warning "      请访问: https://rustup.rs/"
    $missingDeps = $true
}

# 检查 Cargo
if (Test-Command "cargo") {
    Write-Success "Cargo 已安装"
    $cargoVersion = cargo --version
    Write-Host "      版本: $cargoVersion" -ForegroundColor Gray
} else {
    Write-Error "Cargo 未安装"
    Write-Warning "      请访问: https://rustup.rs/"
    $missingDeps = $true
}

# 检查 Git (可选)
if (Test-Command "git") {
    Write-Success "Git 已安装"
    $gitVersion = git --version
    Write-Host "      版本: $gitVersion" -ForegroundColor Gray
}

Write-Host ""

if ($missingDeps) {
    Write-Error "错误: 缺少必要依赖，请先安装上述工具"
    Write-Host ""
    Write-Host "安装 Rust & Cargo:" -ForegroundColor Yellow
    Write-Host "  方法1: 访问 https://rustup.rs/ 下载安装器"
    Write-Host "  方法2: 使用 winget: winget install Rustlang.Rustup"
    Write-Host "  方法3: 使用 chocolatey: choco install rust"
    Write-Host ""
    Read-Host "按任意键退出"
    exit 1
}

# 步骤2：检查项目结构
Write-Info "[2/5] 检查项目结构..."
Write-Host ""

if (-not (Test-Path "o_sovereign_rust")) {
    Write-Error "错误: 找不到 o_sovereign_rust 目录"
    Write-Host "请确保在 ACSA 项目根目录执行此脚本" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "按任意键退出"
    exit 1
}

Set-Location o_sovereign_rust
Write-Success "已进入 o_sovereign_rust 目录"
Write-Host ""

# 步骤3：检查配置
Write-Info "[3/5] 检查配置..."
Write-Host ""

if (-not (Test-Path ".env.example")) {
    Write-Warning "警告: 未找到 .env.example 文件"
} else {
    if (-not (Test-Path ".env")) {
        Write-Warning "提示: 未找到 .env 文件，是否从 .env.example 创建? (Y/N)"
        $response = Read-Host "请选择"
        if ($response -eq "Y" -or $response -eq "y") {
            Copy-Item .env.example .env
            Write-Success "已创建 .env 文件"
            Write-Warning "请编辑 .env 配置您的 API 密钥"
        }
    } else {
        Write-Success ".env 配置文件已存在"
    }
}
Write-Host ""

# 步骤4：构建项目
Write-Info "[4/5] 构建项目..."
Write-Host ""
Write-Host "这可能需要几分钟时间（首次构建）..." -ForegroundColor Cyan
Write-Host ""

$buildResult = cargo build --release 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Error "✗ 构建失败"
    Write-Host ""
    Write-Host "错误详情:" -ForegroundColor Red
    Write-Host $buildResult
    Write-Host ""
    Read-Host "按任意键退出"
    Set-Location ..
    exit 1
}

Write-Host ""
Write-Success "✓ 构建成功！"
Write-Host ""

# 步骤5：运行项目
Write-Info "[5/5] 启动 ACSA..."
Write-Host ""

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   ACSA 启动选项" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "1) 运行 CLI 模式 (推荐)"
Write-Host "2) 运行测试"
Write-Host "3) 生成文档"
Write-Host "4) 退出"
Write-Host ""
$choice = Read-Host "请选择 (1-4)"

switch ($choice) {
    "1" {
        Write-Host ""
        Write-Success "启动 CLI 模式..."
        Write-Host ""
        cargo run --release --bin o-sovereign-cli
    }
    "2" {
        Write-Host ""
        Write-Success "运行测试..."
        Write-Host ""
        cargo test
    }
    "3" {
        Write-Host ""
        Write-Success "生成文档..."
        Write-Host ""
        cargo doc --no-deps --open
    }
    "4" {
        Write-Host ""
        Write-Success "退出"
        Set-Location ..
        exit 0
    }
    default {
        Write-Host ""
        Write-Error "无效选项"
        Read-Host "按任意键退出"
        Set-Location ..
        exit 1
    }
}

Write-Host ""
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   感谢使用 ACSA!" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "文档: https://github.com/acsa-project/acsa/tree/main/docs" -ForegroundColor Blue
Write-Host "问题反馈: https://github.com/acsa-project/acsa/issues" -ForegroundColor Blue
Write-Host ""

Set-Location ..
Read-Host "按任意键退出"
