# Windows 编译修复指南

## 问题概述

Windows 下编译 ACSA 的 UI 版本（包含 Dioxus 0.7 + Ratatui）时，可能遇到以下错误：

```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dioxus`
error[E0433]: failed to resolve: use of unresolved crate `dioxus_tui`
linking with `link.exe` failed: exit code: 1120
```

## 根本原因

1. **Dioxus 架构变更**：
   - Dioxus 0.7 移除了 TUI 后端（从 0.5 开始）
   - 现在使用 **Dioxus 0.7**（桌面 GUI）+ **Ratatui 0.29**（终端 TUI）
   - `use_signal` 现在通过 `dioxus::prelude::*` 引入

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

### 步骤 3：安装 NASM（可选但推荐）

NASM 是 aws-lc-sys（TLS 库）在 Windows 上进行性能优化的必需工具。

**方式 1 - 使用 Chocolatey**
```powershell
choco install nasm -y
```

**方式 2 - 手动下载**
1. 下载 [NASM 官方安装包](https://www.nasm.us/)
2. 安装到默认位置（通常是 `C:\Program Files\NASM`）

**方式 3 - 手动加入系统 PATH（立即生效）**

如果 NASM 已安装但未被识别，在 PowerShell（管理员权限）中执行：

```powershell
# 永久加入系统 PATH
[System.Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Program Files\NASM", "Machine")
```

**执行后需要关闭所有 PowerShell 窗口并重新开启！**

**验证 NASM 安装**:
```powershell
# 重新开启 PowerShell 后
nasm --version
# 应显示: NASM version 2.x.x
```

**如果 NASM 仍然无法识别（替代方案）**:

使用 aws-lc-sys 的预编译模式：
```powershell
$env:AWS_LC_SYS_PREBUILT_NASM = "1"
cargo build --release --features full
```

这会告诉编译器使用预编译的二进制文件，而不需要本机 NASM。

### 步骤 4：验证安装

打开 **Developer Command Prompt for VS 2022**（或新的 PowerShell 窗口）：

```powershell
# 验证 MSVC 工具链
where link.exe
# 应显示: C:\Program Files\Microsoft Visual Studio\2022\...\link.exe

# 验证 CMake
cmake --version
# 应显示: cmake version 3.x.x

# 验证 NASM（可选）
nasm --version
# 应显示: NASM version 2.x.x

# 验证 Rust 工具链
rustc --version
cargo --version
```

### 步骤 5：构建 ACSA UI 版本

```powershell
cd ACSA\o_sovereign_rust

# 清理之前的构建缓存
cargo clean

# 构建 UI 版本
cargo build --release --features ui

# 或构建完整版本
cargo build --release --features full
```

**⏱️ 首次编译预计耗时：10-30 分钟**（取决于 CPU 性能和网速）

**为什么这么慢？** 请参考 [编译时间与优化](#编译时间与优化) 章节。

### 步骤 6：运行 UI 版本

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

### Q6: PowerShell 脚本报错 "未预期的 '}' 语汇基元" / "缺少 '}'"？

**A:** 这是 **Windows PowerShell 5.1** 的 UTF-8 编码解析问题（PowerShell 7 无此问题）。

**症状**：
```
位於 C:\...\quick-start.ps1:125 字元:1
+ }
+ ~
運算式或陳述式中有未預期的 '}' 語彙基元。
```

**根本原因**：
- 脚本包含中文和 Unicode 字符（✓ ✗ ⚠）
- 文件保存为 **UTF-8 无 BOM**
- Windows PowerShell 5.1 会按 ANSI codepage 误解码
- 解析器读错字符 → 括号对不上 → 连锁报错

**解决方案 A：重新保存为 UTF-8 with BOM**（推荐）

```powershell
# 按字节加 BOM（最稳定）
$path = ".\quick-start.ps1"
$bytes = [System.IO.File]::ReadAllBytes($path)

# 去除已有 BOM（如果有）
if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
  $bytes = $bytes[3..($bytes.Length-1)]
}

# 写回：BOM + 原始内容
[System.IO.File]::WriteAllBytes($path, @(0xEF,0xBB,0xBF) + $bytes)
```

**解决方案 B：使用 PowerShell 7**（推荐）

```powershell
# 安装 PowerShell 7
winget install --id Microsoft.PowerShell --source winget

# 然后用 pwsh 运行
pwsh -ExecutionPolicy Bypass -File .\quick-start.ps1
```

**解决方案 C：临时执行策略 + 忽略编码问题**

```powershell
# 设置临时执行策略
Set-ExecutionPolicy Bypass -Scope Process

# 直接运行（如果内容已正确）
.\quick-start.ps1
```

**参考**：
- [Microsoft Learn: PowerShell 5.1 UTF-8 解析问题](https://learn.microsoft.com/en-us/answers/questions/3850223/powershell-5-1-parser-bug-failure-to-parse-utf-8)
- [Microsoft Learn: 文件编码说明](https://learn.microsoft.com/en-us/powershell/scripting/dev-cross-plat/vscode/understanding-file-encoding)

### Q7: 如何切换界面语言？UI 为什么是英文？

**A:** ACSA 包含完整的 i18n 国际化模块（支持中文、英文、日文、韩文），但 **目前 Desktop 和 TUI 界面尚未集成语言切换功能**。

**当前状态**：
- ✅ **i18n 模块已实现**：`src/core/i18n.rs`
- ✅ **支持 4 种语言**：简体中文（zh-CN）、英文（en-US）、日文（ja-JP）、韩文（ko-KR）
- ❌ **UI 集成待完成**：Desktop 和 TUI 界面暂未调用 i18n 模块

**临时解决方案**（开发者）：

如果你需要在代码中使用 i18n，可以这样调用：

```rust
use acsa_core::{I18n, Language};

// 创建中文 i18n 实例
let i18n = I18n::new(Language::ChineseSimplified);

// 获取翻译
let welcome = i18n.t(TranslationKey::Welcome);
println!("{}", welcome); // 输出：欢迎使用 ACSA

// 切换语言
i18n.set_language(Language::EnglishUS);
let welcome_en = i18n.t(TranslationKey::Welcome);
println!("{}", welcome_en); // 输出：Welcome to ACSA
```

**未来计划**：
- 🔄 在 Desktop/TUI 界面添加语言选择菜单
- 🔄 记住用户语言偏好设置
- 🔄 支持系统语言自动检测

**代码位置**：
- i18n 模块：`o_sovereign_rust/src/core/i18n.rs`
- 测试代码：同文件末尾（包含所有语言测试）

## 编译时间与优化

### 为什么 Rust 编译这么慢？

很多开发者第一次编译 Rust 项目时都会有这样的感受：**"我是在浪费时间吗？"**

这句话说中了所有 C/C++/Rust 开发者的心声。在编程界有一个经典的笑话：

> **"为什么 C++ 工程师薪水比较高？因为一半的时间都在等编译，这叫『精神抚慰金』。"**

但更精确地说，这不只是"浪费时间"，而是 **"将开发者的等待，转换为程序的运行效率"**。

### 编译慢的核心原因

#### 1. **编译时做了超繁重的体力活**

像 Python 或 JavaScript 这种语言，编译器几乎不检查什么。但 Rust 或 C++ 在编译时会进行：

- **LLVM 优化**：为了让你的程序跑得跟飞的一样快，编译器会反复尝试上百种方法来优化汇编语言（Assembly）
- **Rust 的借用检查（Borrow Checker）**：Rust 会在编译时扫描所有的内存安全规则，确保你不会有 Data Race 或空指针
- **泛型展开（Monomorphization）**：如果你写了一个泛型函数，编译器会为每一种你用到的类型生成一份独立的代码，这会导致代码量暴增

#### 2. **静态链接（Static Linking）的代价**

你刚才编译的 `aws-lc-sys` 为什么要 CMake 和 NASM？

因为 Rust 倾向于把所有依赖库都直接编译进一个二进制文件里。这样你的程序放到别台计算机不用装一堆 DLL 就能跑，但代价就是编译时要现场 **"从头开始组装整台飞机"**。

#### 3. **C 依赖的跨语言编译**

像 `aws-lc-sys`、`ring`、`rustls` 这些加密库，底层用 C/C++ 写的，需要：
- ✅ C/C++ 编译器（MSVC）
- ✅ CMake 构建系统
- ✅ NASM 汇编器（性能优化）

每次构建都要编译这些 C 代码，然后链接到 Rust 代码中。

### 如何在 2025 年节省编译时间？

既然你已经开始开发 Rust，这里有几个 **老手必备的技巧**：

#### 1. **不要每次都 --release**

```powershell
# ❌ 开发时不要这样做（慢 5-10 倍）
cargo build --release

# ✅ 开发时用 Debug 模式
cargo build

# ✅ 只在最终发布时用 release
cargo build --release
```

**原因**：`--release` 会开启最极限的优化，编译时间慢 5-10 倍，但运行速度快 2-10 倍。开发时不需要。

#### 2. **使用 cargo check**

```powershell
# ✅ 只检查语法，不生成二进制文件（极快）
cargo check

# ✅ 检查特定 feature
cargo check --features ui
```

**速度对比**：
- `cargo check`：5-10 秒
- `cargo build`：1-5 分钟
- `cargo build --release`：10-30 分钟

#### 3. **使用 sccache（分布式编译缓存）**

sccache 是 Mozilla 开发的编译缓存工具，可以极大加速重复编译。

```powershell
# 安装 sccache
cargo install sccache

# 配置 Rust 使用 sccache
$env:RUSTC_WRAPPER = "sccache"

# 现在编译会自动缓存
cargo build
```

**效果**：
- 首次编译：20 分钟
- 清理后再编译：2 分钟（缓存命中）

#### 4. **增量编译（Incremental Compilation）**

Rust 默认开启，但可以确认：

```powershell
# 确保开启增量编译（默认已开启）
$env:CARGO_INCREMENTAL = "1"
cargo build
```

**效果**：只重新编译修改过的部分，而不是整个项目。

#### 5. **并行编译（使用多核）**

```powershell
# 使用所有 CPU 核心（默认已开启）
cargo build -j 8  # 8 核并行

# 或者让 Cargo 自动检测
cargo build  # 默认使用所有核心
```

#### 6. **跳过不需要的依赖**

```powershell
# 只构建需要的 features
cargo build --no-default-features --features "core,ui"

# 跳过文档和测试
cargo build --release --workspace --exclude "*"
```

#### 7. **使用 mold 链接器（Linux）或 lld（Windows）**

在 `.cargo/config.toml` 中配置：

```toml
[target.x86_64-pc-windows-msvc]
linker = "rust-lld"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**效果**：链接速度提升 2-5 倍。

#### 8. **换台电脑（现实但残酷）**

C 家族编译非常吃：
- ✅ **CPU 多核性能**：16 核以上最佳
- ✅ **SSD 读写速度**：NVMe SSD 比 SATA SSD 快 3 倍
- ✅ **内存**：16GB 以上，32GB 更好

**2025 年推荐配置**：
- CPU: AMD Ryzen 9 / Intel i9（16+ 核心）
- RAM: 32GB DDR5
- SSD: 1TB NVMe Gen4

### 编译时间对比表

| 操作 | Debug 模式 | Release 模式 |
|------|-----------|-------------|
| 首次完整构建 | 5-10 分钟 | 15-30 分钟 |
| 增量编译（修改一个文件） | 5-30 秒 | 1-5 分钟 |
| cargo check | 5-10 秒 | N/A |
| 使用 sccache 后的清理重建 | 1-2 分钟 | 3-5 分钟 |

### 总结

你刚才经历的 **"安装 CMake → 安装 NASM → 修复环境变量"**，是 C 家族 **"痛苦"的顶峰——环境配置**。

一旦环境通了，之后的开发大多只需要等 `cargo build`。虽然等待很烦，但换来的是：

✅ **运行速度**：比 Java 或 Python 快上几十倍
✅ **内存占用**：极低（MB 级别 vs GB 级别）
✅ **安全性**：编译时就消灭了 90% 的 bug
✅ **可移植性**：一个二进制文件，到处运行

这就是所谓的 **"前期痛苦，后期享福"**。

## 技术背景

### 为什么 Windows 编译这么麻烦？

1. **MSVC vs GNU**：Windows 默认使用 MSVC ABI，而 Linux/macOS 使用 GCC/Clang
2. **动态链接库**：Windows 的 DLL 系统与 Unix 的 .so 不同
3. **TLS 实现**：`rustls` 使用的 `aws-lc-sys` 需要 CMake 和 C++ 编译器
4. **WebView2**：Dioxus 0.7 desktop 依赖 Windows 特有的 WebView2 组件

### Dioxus 0.7 架构变更

**重要变化（2025年12月）：**

1. **TUI 后端移除**：Dioxus 从 0.5 开始移除了 TUI 支持（[Issue #2620](https://github.com/DioxusLabs/dioxus/issues/2620)）
   - `dioxus-tui` 包已不存在
   - 官方推荐使用 [Ratatui](https://github.com/ratatui/ratatui) 作为 TUI 替代

2. **Signal 系统重写**：
   - `use_signal` 必须通过 `use dioxus::prelude::*` 引入
   - 基于 generational-box 的新实现
   - Launch API 简化：`launch(App)` 替代 `dioxus::desktop::launch(App)`

3. **ACSA 架构**：
   - **desktop.rs**：使用 Dioxus 0.7（WebView2 桌面 GUI）
   - **tui.rs**：使用 Ratatui 0.29 + Crossterm 0.28（终端 TUI）

### dx-cli 已知问题

根据 GitHub Issues [#4877](https://github.com/DioxusLabs/dioxus/issues/4877) 和 [#4878](https://github.com/DioxusLabs/dioxus/issues/4878)，dx-cli 0.7.0 在 Windows 11 上存在链接器问题：

- 错误代码：`exit code: 1120`（符号未定义）
- 影响版本：dx-cli 0.7.0（2025年11月发布）
- 临时解决：降级到 0.7.0-rc.3（但推荐直接使用 `cargo build`）

## 参考链接

**官方文档：**
- [Rust MSVC 工具链安装指南](https://rust-lang.github.io/rustup/installation/windows-msvc.html)
- [Visual Studio Build Tools 下载](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
- [CMake 官方下载](https://cmake.org/download/)
- [Dioxus 0.7 迁移指南](https://dioxuslabs.com/learn/0.7/migration/to_07/)
- [Ratatui 官方文档](https://ratatui.rs/)

**相关 Issues：**
- [dx-cli 0.7.0 Windows 编译失败 #4878](https://github.com/DioxusLabs/dioxus/issues/4878)
- [dx-cli 0.7.0 桌面版构建失败 #4877](https://github.com/DioxusLabs/dioxus/issues/4877)
- [Dioxus TUI 状态和路线图 #2620](https://github.com/DioxusLabs/dioxus/issues/2620)
- [link.exe 未找到解决方案](https://users.rust-lang.org/t/link-exe-not-found-error-when-building-rust-on-windows-11-msvc-target/133232)

**社区讨论：**
- [Windows 11 编译配置](https://github.com/DioxusLabs/dioxus/discussions/4249)
- [AWS-LC-SYS CMake 依赖问题](https://github.com/DioxusLabs/dioxus/issues/3309)
- [Ratatui vs tui-rs #1311](https://github.com/DioxusLabs/dioxus/issues/1311)

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
**适用版本：** ACSA 0.1.0, Dioxus 0.7, Ratatui 0.29
**测试环境：** Windows 11 Pro, Visual Studio 2022 Build Tools
