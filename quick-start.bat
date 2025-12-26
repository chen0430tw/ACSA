@echo off
REM ACSA 一键启动脚本 (Windows)
REM Quick Start Script for ACSA - Windows Version

chcp 65001 >nul
setlocal enabledelayedexpansion

echo ========================================
echo    ACSA 一键启动脚本 (Windows)
echo    Quick Start Script
echo ========================================
echo.

REM 步骤1：检查环境
echo [1/5] 检查环境依赖...
echo.

set MISSING_DEPS=0

REM 检查 Rust
where rustc >nul 2>&1
if %errorlevel% equ 0 (
    echo [92m✓[0m Rust 已安装
    for /f "tokens=*" %%i in ('rustc --version') do echo       版本: %%i
) else (
    echo [91m✗[0m Rust 未安装
    echo       请访问: https://rustup.rs/
    set MISSING_DEPS=1
)

REM 检查 Cargo
where cargo >nul 2>&1
if %errorlevel% equ 0 (
    echo [92m✓[0m Cargo 已安装
    for /f "tokens=*" %%i in ('cargo --version') do echo       版本: %%i
) else (
    echo [91m✗[0m Cargo 未安装
    echo       请访问: https://rustup.rs/
    set MISSING_DEPS=1
)

REM 检查 Git (可选)
where git >nul 2>&1
if %errorlevel% equ 0 (
    echo [92m✓[0m Git 已安装
    for /f "tokens=*" %%i in ('git --version') do echo       版本: %%i
)

echo.

if %MISSING_DEPS% equ 1 (
    echo [91m错误: 缺少必要依赖，请先安装上述工具[0m
    echo.
    echo 安装 Rust ^& Cargo:
    echo   访问 https://rustup.rs/ 下载安装器
    echo   或使用 winget: winget install Rustlang.Rustup
    echo.
    pause
    exit /b 1
)

REM 步骤2：检查项目结构
echo [2/5] 检查项目结构...
echo.

if not exist "o_sovereign_rust" (
    echo [91m错误: 找不到 o_sovereign_rust 目录[0m
    echo 请确保在 ACSA 项目根目录执行此脚本
    echo.
    pause
    exit /b 1
)

cd o_sovereign_rust
echo [92m✓[0m 已进入 o_sovereign_rust 目录
echo.

REM 步骤3：检查配置
echo [3/5] 检查配置...
echo.

if not exist ".env.example" (
    echo [93m警告: 未找到 .env.example 文件[0m
) else (
    if not exist ".env" (
        echo [93m提示: 未找到 .env 文件，是否从 .env.example 创建? (Y/N)[0m
        set /p response=请选择:
        if /i "!response!"=="Y" (
            copy .env.example .env >nul
            echo [92m✓[0m 已创建 .env 文件
            echo [93m请编辑 .env 配置您的 API 密钥[0m
        )
    ) else (
        echo [92m✓[0m .env 配置文件已存在
    )
)
echo.

REM 步骤4：构建项目
echo [4/5] 构建项目...
echo.
echo 这可能需要几分钟时间（首次构建）...
echo.

cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo [91m✗ 构建失败[0m
    echo.
    pause
    exit /b 1
)

echo.
echo [92m✓ 构建成功！[0m
echo.

REM 步骤5：运行项目
echo [5/5] 启动 ACSA...
echo.

echo ========================================
echo    ACSA 启动选项
echo ========================================
echo.
echo 1) 运行 CLI 模式 (推荐)
echo 2) 运行测试
echo 3) 生成文档
echo 4) 退出
echo.
set /p choice=请选择 (1-4):

if "%choice%"=="1" (
    echo.
    echo [92m启动 CLI 模式...[0m
    echo.
    cargo run --release --bin o-sovereign-cli
) else if "%choice%"=="2" (
    echo.
    echo [92m运行测试...[0m
    echo.
    cargo test
) else if "%choice%"=="3" (
    echo.
    echo [92m生成文档...[0m
    echo.
    cargo doc --no-deps --open
) else if "%choice%"=="4" (
    echo.
    echo [92m退出[0m
    goto :end
) else (
    echo.
    echo [91m无效选项[0m
    pause
    exit /b 1
)

echo.
echo ========================================
echo    感谢使用 ACSA!
echo ========================================
echo.
echo 文档: https://github.com/acsa-project/acsa/tree/main/docs
echo 问题反馈: https://github.com/acsa-project/acsa/issues
echo.

:end
cd ..
pause
