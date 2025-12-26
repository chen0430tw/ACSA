# Windows 编译修复指南

## 问题概述

Windows 下编译 ACSA 的 UI 版本（包含 dioxus）时，可能遇到以下错误：

```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dioxus`
linking with `link.exe` failed: exit code: 1120
```

## 根本原因

1. **Dioxus 0.7.0 已知问题**：最新版 dx-cli 在 Windows 上链接失败（2025年11月报告）
2. **缺失 MSVC 工具链**：Rust 在 Windows 上需要 Microsoft Visual C++ 编译工具
3. **缺失 CMake**：`aws-lc-sys` 依赖（rustls 的 TLS 实现）需要 CMake
4. **缺失 NASM**（可选）：某些加密库可能需要

## 完整解决方案

### 步骤 1：安装 Visual Studio Build Tools（必需）

**方式 1 - 使用 Visual Studio Installer（推荐）**

1. 下载 [Visual Studio 2022 Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
2. 运行安装程序
3. 选择 **"使用 C++ 的桌面开发"** (Desktop Development with C++) 工作负载
4. 确保勾选以下组件：
   - ✅ MSVC v143 - VS 2022 C++ x64/x86 生成工具
   - ✅ Windows 10/11 SDK
   - ✅ C++ CMake tools for Windows
   - ✅ C++ ATL for latest build tools

**方式 2 - 使用 Chocolatey（快速）**

```powershell
# 以管理员身份运行 PowerShell
choco install visualstudio2022-workload-vctools -y
choco install cmake -y
```

**方式 3 - 仅安装 Build Tools Essentials**

如果只需要最小化安装：
```powershell
# 下载 Visual Studio Build Tools Installer
# 然后以管理员身份运行：
vs_buildtools.exe --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
```

### 步骤 2：安装 CMake（必需）

**方式 1 - 使用官方安装包**
1. 下载 [CMake 官方安装包](https://cmake.org/download/)
2. 安装时勾选 "Add CMake to system PATH"

**方式 2 - 使用 Chocolatey**
```powershell
choco install cmake -y
```

**方式 3 - 使用 winget**
```powershell
winget install -e --id Kitware.CMake
```

### 步骤 3：验证安装

打开 **Developer Command Prompt for VS 2022**（或新的 PowerShell 窗口）：

```powershell
# 验证 MSVC 工具链
where link.exe
# 应显示: C:\Program Files\Microsoft Visual Studio\2022\...\link.exe

# 验证 CMake
cmake --version
# 应显示: cmake version 3.x.x

# 验证 Rust 工具链
rustc --version
cargo --version
```

### 步骤 4：构建 ACSA UI 版本

```powershell
cd ACSA\o_sovereign_rust

# 清理之前的构建缓存
cargo clean

# 构建 UI 版本
cargo build --release --features ui

# 或构建完整版本
cargo build --release --features full
```

### 步骤 5：运行 UI 版本

```powershell
# 运行桌面版
cargo run --release --bin o-sovereign-desktop --features ui

# 运行终端 TUI 版
cargo run --release --bin o-sovereign-tui --features ui
```

## 替代方案

### 方案 A：降级 Dioxus 版本（临时解决）

如果上述步骤仍然失败，可以尝试使用稳定的旧版本：

编辑 `o_sovereign_rust/Cargo.toml`：

```toml
# 将版本从 0.5 改为 0.4.x
dioxus = { version = "0.4", optional = true }
dioxus-desktop = { version = "0.4", optional = true }
dioxus-tui = { version = "0.4", optional = true }
```

然后重新构建：
```powershell
cargo clean
cargo build --release --features ui
```

### 方案 B：使用 WSL2（推荐用于开发）

在 Windows 下使用 WSL2 (Windows Subsystem for Linux) 可以避免大部分 Windows 编译问题：

```powershell
# 在 PowerShell（管理员）中启用 WSL
wsl --install -d Ubuntu-22.04

# 进入 WSL
wsl

# 在 WSL 内安装 Rust 和依赖
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt update
sudo apt install build-essential pkg-config libssl-dev -y

# 克隆并构建
git clone https://github.com/chen0430tw/ACSA.git
cd ACSA/o_sovereign_rust
cargo build --release --features full
```

### 方案 C：仅使用核心功能（无 UI）

如果只需要命令行版本，直接使用默认构建：

```powershell
cargo build --release
cargo run --release --bin o-sovereign-cli
```

## 常见问题 (FAQ)

### Q1: 安装 Build Tools 后仍然找不到 link.exe？

**A:** 确保使用 **"Developer Command Prompt for VS 2022"** 或者在 PowerShell 中手动激活 MSVC 环境：

```powershell
# 64位系统
& "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

# 然后再运行 cargo build
cargo build --release --features ui
```

### Q2: CMake 错误 "Could not find toolset"？

**A:** 确保安装了完整的 C++ 工作负载，而不仅仅是 Build Tools。

### Q3: 编译时出现 "NASM not found" 警告？

**A:** 安装 NASM（某些加密库需要）：

```powershell
# 使用 Chocolatey
choco install nasm -y

# 或从官网下载
# https://www.nasm.us/
```

### Q4: 编译成功但运行时崩溃？

**A:** 确保安装了 WebView2 Runtime（dioxus-desktop 需要）：

```powershell
# 使用 Chocolatey
choco install webview2-runtime -y

# 或从官网下载
# https://developer.microsoft.com/microsoft-edge/webview2/
```

### Q5: 磁盘空间不足？

**A:** Visual Studio Build Tools 需要约 4-6 GB 空间。如果空间不足，可以：
1. 使用 WSL2 方案（约 2 GB）
2. 仅安装核心版本（不含 UI）

## 技术背景

### 为什么 Windows 编译这么麻烦？

1. **MSVC vs GNU**：Windows 默认使用 MSVC ABI，而 Linux/macOS 使用 GCC/Clang
2. **动态链接库**：Windows 的 DLL 系统与 Unix 的 .so 不同
3. **TLS 实现**：`rustls` 使用的 `aws-lc-sys` 需要 CMake 和 C++ 编译器
4. **WebView2**：dioxus-desktop 依赖 Windows 特有的 WebView2 组件

### Dioxus 0.7.0 具体问题

根据 GitHub Issues [#4877](https://github.com/DioxusLabs/dioxus/issues/4877) 和 [#4878](https://github.com/DioxusLabs/dioxus/issues/4878)，Dioxus CLI 0.7.0 在 Windows 11 上存在链接器问题：

- 错误代码：`exit code: 1120`（符号未定义）
- 影响版本：dx-cli 0.7.0（2025年11月发布）
- 临时解决：降级到 0.7.0-rc.3 或使用 0.4.x 稳定版

## 参考链接

**官方文档：**
- [Rust MSVC 工具链安装指南](https://rust-lang.github.io/rustup/installation/windows-msvc.html)
- [Visual Studio Build Tools 下载](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
- [CMake 官方下载](https://cmake.org/download/)

**相关 Issues：**
- [dx-cli 0.7.0 Windows 编译失败 #4878](https://github.com/DioxusLabs/dioxus/issues/4878)
- [dx-cli 0.7.0 桌面版构建失败 #4877](https://github.com/DioxusLabs/dioxus/issues/4877)
- [link.exe 未找到解决方案](https://users.rust-lang.org/t/link-exe-not-found-error-when-building-rust-on-windows-11-msvc-target/133232)

**社区讨论：**
- [Windows 11 编译配置](https://github.com/DioxusLabs/dioxus/discussions/4249)
- [AWS-LC-SYS CMake 依赖问题](https://github.com/DioxusLabs/dioxus/issues/3309)

## 总结

**推荐方案（按优先级）：**

1. ✅ **完整安装 Visual Studio Build Tools + CMake**（最稳定）
2. ✅ **使用 WSL2**（开发体验最好）
3. ⚠️ **降级 Dioxus 版本**（临时解决）
4. ℹ️ **仅使用核心功能**（避免 UI 依赖）

**预计安装时间：**
- Visual Studio Build Tools: 15-30 分钟（取决于网速）
- CMake: 5 分钟
- 首次编译 ACSA（UI版）: 10-20 分钟

**需要的磁盘空间：**
- Visual Studio Build Tools: ~6 GB
- Rust 工具链: ~2 GB
- ACSA 构建缓存: ~2 GB
- **总计**: ~10 GB

---

**最后更新：** 2025-12-26
**适用版本：** ACSA 0.1.0, Dioxus 0.5
**测试环境：** Windows 11 Pro, Visual Studio 2022 Build Tools
