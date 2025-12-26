#!/bin/bash
set -e

# ACSA 一键启动脚本
# Quick Start Script for ACSA (O-Sovereign)

echo "================================="
echo "   ACSA 一键启动脚本"
echo "   Quick Start Script"
echo "================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查函数
check_command() {
    if command -v $1 &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 已安装"
        return 0
    else
        echo -e "${RED}✗${NC} $1 未安装"
        return 1
    fi
}

# 步骤1：检查环境
echo -e "${BLUE}[1/5]${NC} 检查环境依赖..."
echo ""

MISSING_DEPS=0

# 检查 Rust
if check_command rustc; then
    RUST_VERSION=$(rustc --version)
    echo "      版本: $RUST_VERSION"
else
    echo -e "${YELLOW}      请安装 Rust: https://rustup.rs/${NC}"
    MISSING_DEPS=1
fi

# 检查 Cargo
if check_command cargo; then
    CARGO_VERSION=$(cargo --version)
    echo "      版本: $CARGO_VERSION"
else
    echo -e "${YELLOW}      请安装 Cargo (随 Rust 一起安装)${NC}"
    MISSING_DEPS=1
fi

# 检查 git (可选)
if check_command git; then
    GIT_VERSION=$(git --version)
    echo "      版本: $GIT_VERSION"
fi

echo ""

if [ $MISSING_DEPS -eq 1 ]; then
    echo -e "${RED}错误: 缺少必要依赖，请先安装上述工具${NC}"
    echo ""
    echo "安装 Rust & Cargo:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    exit 1
fi

# 步骤2：进入项目目录
echo -e "${BLUE}[2/5]${NC} 检查项目结构..."
echo ""

if [ ! -d "o_sovereign_rust" ]; then
    echo -e "${RED}错误: 找不到 o_sovereign_rust 目录${NC}"
    echo "请确保在 ACSA 项目根目录执行此脚本"
    exit 1
fi

cd o_sovereign_rust
echo -e "${GREEN}✓${NC} 已进入 o_sovereign_rust 目录"
echo ""

# 步骤3：检查配置文件
echo -e "${BLUE}[3/5]${NC} 检查配置..."
echo ""

if [ ! -f ".env.example" ]; then
    echo -e "${YELLOW}警告: 未找到 .env.example 文件${NC}"
else
    if [ ! -f ".env" ]; then
        echo -e "${YELLOW}提示: 未找到 .env 文件，是否从 .env.example 创建? (y/n)${NC}"
        read -r response
        if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
            cp .env.example .env
            echo -e "${GREEN}✓${NC} 已创建 .env 文件"
            echo -e "${YELLOW}请编辑 .env 配置您的 API 密钥${NC}"
        fi
    else
        echo -e "${GREEN}✓${NC} .env 配置文件已存在"
    fi
fi
echo ""

# 步骤4：构建项目
echo -e "${BLUE}[4/5]${NC} 构建项目..."
echo ""
echo "这可能需要几分钟时间（首次构建）..."
echo ""

if cargo build --release; then
    echo ""
    echo -e "${GREEN}✓${NC} 构建成功！"
else
    echo ""
    echo -e "${RED}✗${NC} 构建失败"
    exit 1
fi
echo ""

# 步骤5：运行项目
echo -e "${BLUE}[5/5]${NC} 启动 ACSA..."
echo ""

echo "================================="
echo "   ACSA 启动选项"
echo "================================="
echo ""
echo "1) 运行 CLI 模式 (推荐)"
echo "2) 运行测试"
echo "3) 生成文档"
echo "4) 退出"
echo ""
read -p "请选择 (1-4): " choice

case $choice in
    1)
        echo ""
        echo -e "${GREEN}启动 CLI 模式...${NC}"
        echo ""
        cargo run --release --bin o-sovereign-cli
        ;;
    2)
        echo ""
        echo -e "${GREEN}运行测试...${NC}"
        echo ""
        cargo test
        ;;
    3)
        echo ""
        echo -e "${GREEN}生成文档...${NC}"
        echo ""
        cargo doc --no-deps --open
        ;;
    4)
        echo ""
        echo -e "${GREEN}退出${NC}"
        exit 0
        ;;
    *)
        echo ""
        echo -e "${RED}无效选项${NC}"
        exit 1
        ;;
esac

echo ""
echo "================================="
echo "   感谢使用 ACSA!"
echo "================================="
echo ""
echo "文档: https://github.com/acsa-project/acsa/tree/main/docs"
echo "问题反馈: https://github.com/acsa-project/acsa/issues"
echo ""
