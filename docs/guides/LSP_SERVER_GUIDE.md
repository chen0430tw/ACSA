# LSP (Language Server Protocol) 服务器指南

**ACSA LSP Server - 智能代码补全与诊断**

---

## 📋 目录

- [什么是 LSP](#什么是-lsp)
- [ACSA LSP 服务器概述](#acsa-lsp-服务器概述)
- [快速开始](#快速开始)
- [编辑器集成](#编辑器集成)
  - [VS Code](#vs-code-集成)
  - [Neovim](#neovim-集成)
  - [Emacs](#emacs-集成)
  - [其他编辑器](#其他编辑器)
- [功能说明](#功能说明)
- [配置选项](#配置选项)
- [故障排查](#故障排查)

---

## 什么是 LSP

**Language Server Protocol (LSP)** 是由 Microsoft 开发的标准化协议，用于在编辑器和语言服务器之间提供智能编程功能。

### 核心功能

- 📝 **代码补全**: 智能提示和自动补全
- 🔍 **定义跳转**: 跳转到定义和引用
- 🐛 **诊断**: 实时错误和警告
- 💡 **代码操作**: 快速修复和重构
- 📖 **悬停提示**: 文档和类型信息

---

## ACSA LSP 服务器概述

### 实现文件

- **核心**: `o_sovereign_rust/src/core/lsp_server.rs`
- **协议**: JSON-RPC 2.0 over stdio

### 支持的能力

ACSA LSP 服务器专注于 ACSA 项目特定的智能功能：

#### 1. **代码补全**
- ✅ ACSA API 函数补全
- ✅ 配置项补全
- ✅ 模块导入补全

#### 2. **诊断**
- ✅ 配置错误检测
- ✅ API 使用错误
- ✅ 最佳实践建议

#### 3. **文档**
- ✅ 悬停提示
- ✅ 函数签名帮助
- ✅ 示例代码

---

## 快速开始

### 1. 构建 LSP 服务器

```bash
cd o_sovereign_rust
cargo build --release --bin acsa-lsp-server
```

### 2. 测试服务器

```bash
# 启动 LSP 服务器（stdio 模式）
./target/release/acsa-lsp-server

# 服务器会等待 JSON-RPC 消息
```

### 3. 编辑器配置

根据您使用的编辑器，参考以下配置指南。

---

## 编辑器集成

### VS Code 集成

#### 方法 1: 使用配置文件

创建 `.vscode/settings.json`:

```json
{
  "acsa.lsp.enable": true,
  "acsa.lsp.serverPath": "/path/to/acsa/target/release/acsa-lsp-server",
  "acsa.lsp.trace.server": "verbose"
}
```

#### 方法 2: 创建 VS Code 扩展

创建 `acsa-vscode-extension/package.json`:

```json
{
  "name": "acsa-lsp",
  "displayName": "ACSA Language Support",
  "description": "ACSA LSP client for VS Code",
  "version": "0.1.0",
  "engines": {
    "vscode": "^1.75.0"
  },
  "activationEvents": [
    "onLanguage:rust",
    "onLanguage:toml"
  ],
  "main": "./out/extension.js",
  "contributes": {
    "configuration": {
      "type": "object",
      "title": "ACSA LSP",
      "properties": {
        "acsa.lsp.serverPath": {
          "type": "string",
          "default": "acsa-lsp-server",
          "description": "Path to ACSA LSP server executable"
        }
      }
    }
  }
}
```

创建 `acsa-vscode-extension/src/extension.ts`:

```typescript
import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const serverPath = workspace
    .getConfiguration('acsa.lsp')
    .get<string>('serverPath') || 'acsa-lsp-server';

  const serverOptions: ServerOptions = {
    command: serverPath,
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'rust' },
      { scheme: 'file', language: 'toml' },
    ],
  };

  client = new LanguageClient(
    'acsaLsp',
    'ACSA Language Server',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
```

---

### Neovim 集成

#### 使用 nvim-lspconfig

在 `~/.config/nvim/init.lua` 或 `~/.config/nvim/lua/lsp-config.lua` 中:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- 定义 ACSA LSP 配置
if not configs.acsa_lsp then
  configs.acsa_lsp = {
    default_config = {
      cmd = {'/path/to/acsa/target/release/acsa-lsp-server'},
      filetypes = {'rust', 'toml'},
      root_dir = lspconfig.util.root_pattern('Cargo.toml', '.git'),
      settings = {},
    },
  }
end

-- 启动 ACSA LSP
lspconfig.acsa_lsp.setup{}
```

#### 使用 coc.nvim

在 `~/.config/nvim/coc-settings.json`:

```json
{
  "languageserver": {
    "acsa": {
      "command": "/path/to/acsa/target/release/acsa-lsp-server",
      "filetypes": ["rust", "toml"],
      "rootPatterns": ["Cargo.toml", ".git"]
    }
  }
}
```

---

### Emacs 集成

#### 使用 lsp-mode

在 `~/.emacs.d/init.el`:

```elisp
(require 'lsp-mode)

;; 定义 ACSA LSP 客户端
(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection "/path/to/acsa/target/release/acsa-lsp-server")
  :major-modes '(rust-mode toml-mode)
  :server-id 'acsa-lsp))

;; 在 Rust 模式中启用
(add-hook 'rust-mode-hook #'lsp)
(add-hook 'toml-mode-hook #'lsp)
```

---

### 其他编辑器

#### Sublime Text

创建 `ACSA.sublime-settings`:

```json
{
  "clients": {
    "acsa-lsp": {
      "enabled": true,
      "command": ["/path/to/acsa/target/release/acsa-lsp-server"],
      "selector": "source.rust | source.toml"
    }
  }
}
```

#### Vim (with vim-lsp)

在 `~/.vimrc`:

```vim
if executable('acsa-lsp-server')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'acsa-lsp',
    \ 'cmd': {server_info->['/path/to/acsa/target/release/acsa-lsp-server']},
    \ 'allowlist': ['rust', 'toml'],
    \ })
endif
```

---

## 功能说明

### 1. 代码补全

ACSA LSP 提供智能补全：

```rust
use acsa_core::{ // <-- 触发补全
    SovereigntySystem,  // ✅ 自动补全
    DoseMeter,          // ✅ 自动补全
    ...
}
```

### 2. 诊断

实时检测错误：

```rust
let config = SovereigntyConfig {
    h0: -100.0,  // ❌ 错误: h0 必须为正数
};
```

### 3. 悬停提示

将鼠标悬停在函数上显示文档：

```rust
sovereignty.calculate_h_t()  // 显示: 计算生物活性函数 H(t)
```

### 4. 定义跳转

`Ctrl+Click` 跳转到定义：

```rust
use acsa_core::SovereigntySystem;
              // ^^^^^^^^^^^^^^^^ Ctrl+Click 跳转到定义
```

---

## 配置选项

### LSP 服务器配置

创建 `acsa-lsp.toml`:

```toml
[server]
# 日志级别
log_level = "info"

# 诊断延迟（毫秒）
diagnostic_delay = 500

[features]
# 启用代码补全
completion = true

# 启用诊断
diagnostics = true

# 启用悬停提示
hover = true

# 启用定义跳转
goto_definition = true

[completion]
# 补全触发字符
trigger_characters = [".", ":", ">"]

# 最大补全项数
max_items = 50

[diagnostics]
# 严重性级别
severity_levels = ["error", "warning", "info", "hint"]

# 启用最佳实践检查
best_practices = true
```

---

## 故障排查

### 问题 1: LSP 服务器未启动

**症状**: 编辑器没有补全和诊断

**解决方案**:
1. 检查服务器路径是否正确
2. 确认服务器可执行权限: `chmod +x acsa-lsp-server`
3. 手动运行服务器测试: `./acsa-lsp-server`

### 问题 2: 补全不工作

**症状**: 没有补全提示

**解决方案**:
1. 检查触发字符配置
2. 查看 LSP 日志: `:LspLog` (Neovim) 或 `Output > ACSA LSP` (VS Code)
3. 重启 LSP 服务器: `:LspRestart` (Neovim)

### 问题 3: 诊断延迟

**症状**: 错误提示出现缓慢

**解决方案**:
- 调整 `diagnostic_delay` 配置
- 减少 `max_items` 限制

### 问题 4: 高内存占用

**解决方案**:
- 限制索引的文件数量
- 增加垃圾回收频率
- 使用增量分析

---

## 开发 LSP 功能

### 添加新的诊断规则

```rust
use acsa_core::{LspDiagnostic, DiagnosticSeverity};

fn check_config_validity(config: &str) -> Vec<LspDiagnostic> {
    let mut diagnostics = Vec::new();

    // 检查配置错误
    if config.contains("h0: 0") {
        diagnostics.push(LspDiagnostic {
            range: Range { /* ... */ },
            severity: Some(DiagnosticSeverity::Error),
            message: "h0 不能为 0".to_string(),
            source: Some("acsa-lsp".to_string()),
        });
    }

    diagnostics
}
```

### 添加新的补全项

```rust
use acsa_core::{CompletionItem, CompletionItemKind};

fn provide_completion() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "SovereigntySystem".to_string(),
            kind: Some(CompletionItemKind::Class),
            detail: Some("主权系统".to_string()),
            documentation: Some("ACSA 主权模式核心系统".to_string()),
            ..Default::default()
        },
        // 更多补全项...
    ]
}
```

---

## 相关资源

### 官方文档
- [LSP 规范](https://microsoft.github.io/language-server-protocol/)
- [LSP 实现指南](https://microsoft.github.io/language-server-protocol/implementors/servers/)

### ACSA 文档
- [MCP 集成指南](MCP_INTEGRATION_GUIDE.md)
- [插件系统文档](../README.md)

### 编辑器文档
- [VS Code LSP 扩展](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)
- [Neovim LSP](https://neovim.io/doc/user/lsp.html)
- [Emacs lsp-mode](https://emacs-lsp.github.io/lsp-mode/)

---

<div align="center">

**ACSA LSP Server**
*Intelligent Code Assistance for ACSA Projects*

Made with ❤️ by the ACSA Team

[GitHub](https://github.com/chen0430tw/ACSA) • [文档](../../README.md) • [Issues](https://github.com/chen0430tw/ACSA/issues)

</div>
