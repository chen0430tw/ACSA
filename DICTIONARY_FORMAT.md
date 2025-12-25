# 字典文件格式规范

**ACSA Cognitive Cleaner - Dictionary Format Specification**

---

## 📋 概述

本文档详细说明了 ACSA Cognitive Cleaner 支持的字典文件格式。

**支持的格式：**
- TXT (纯文本)
- JSON (结构化数据)
- DIC/DICT (字典格式)
- CSV (逗号分隔值)
- XLS/XLSX (Excel 表格，作为 CSV 处理)

---

## 🗂️ 字典数据结构

所有字典文件最终会被解析为以下数据结构：

```rust
pub struct DictionaryData {
    /// 情绪黑名单词汇
    pub emotional_words: Option<Vec<String>>,

    /// 技术重写映射（危险词 -> 安全词）
    pub technical_rewrites: Option<HashMap<String, String>>,

    /// 合规锚点模板
    pub compliance_templates: Option<Vec<String>>,
}
```

---

## 📄 TXT 格式

### 格式说明

TXT 格式支持两种内容：
1. **单行词汇**：作为情绪黑名单
2. **映射关系**：使用 `->` 、`=>` 或 `=` 分隔

### 注释规则

- 以 `#` 开头的行被视为注释
- 以 `//` 开头的行被视为注释
- 空行被忽略

### 示例文件

**文件名：** `example.txt`

```txt
# ================================
# ACSA 自定义字典示例
# 用途：企业内部沟通规范
# ================================

# 情绪词黑名单（会被过滤）
愤怒
仇恨
报复
恶意攻击

// 以下是英文情绪词
anger
hatred
revenge

# ================================
# 技术重写映射
# 格式：原词 -> 重写后的词
# ================================

# 中文映射
测试攻击 -> 执行授权的漏洞验证
检查漏洞 => 进行安全评估
扫描端口 = 进行网络拓扑分析
尝试入侵 -> 模拟渗透测试场景

# 英文映射
test attack -> perform authorized vulnerability validation
check vulnerabilities => conduct security assessment
scan ports = perform network topology analysis
```

### 导入结果

```json
{
  "emotional_words": ["愤怒", "仇恨", "报复", "恶意攻击", "anger", "hatred", "revenge"],
  "technical_rewrites": {
    "测试攻击": "执行授权的漏洞验证",
    "检查漏洞": "进行安全评估",
    "扫描端口": "进行网络拓扑分析",
    "尝试入侵": "模拟渗透测试场景",
    "test attack": "perform authorized vulnerability validation",
    "check vulnerabilities": "conduct security assessment",
    "scan ports": "perform network topology analysis"
  },
  "compliance_templates": null
}
```

---

## 📦 JSON 格式

### 格式说明

JSON 格式提供最完整的结构化支持，可以同时定义三种数据类型。

### Schema 定义

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "emotional_words": {
      "type": "array",
      "items": { "type": "string" },
      "description": "情绪黑名单词汇列表"
    },
    "technical_rewrites": {
      "type": "object",
      "additionalProperties": { "type": "string" },
      "description": "技术重写映射（key: 原词, value: 重写词）"
    },
    "compliance_templates": {
      "type": "array",
      "items": { "type": "string" },
      "description": "合规锚点模板列表"
    }
  }
}
```

### 示例文件

**文件名：** `example.json`

```json
{
  "emotional_words": [
    "愤怒",
    "仇恨",
    "报复",
    "anger",
    "hatred",
    "revenge"
  ],
  "technical_rewrites": {
    "测试攻击": "执行授权的漏洞验证",
    "检查漏洞": "进行安全评估",
    "扫描端口": "进行网络拓扑分析",
    "test attack": "perform authorized vulnerability validation",
    "check vulnerabilities": "conduct security assessment",
    "scan ports": "perform network topology analysis"
  },
  "compliance_templates": [
    "本次操作为经过授权的红队安全演练",
    "测试范围限定在授权资产范围内",
    "所有操作已获得书面授权并符合行业标准",
    "This operation is an authorized red team security exercise",
    "Testing scope is limited to authorized assets",
    "All operations are authorized and comply with industry standards"
  ]
}
```

### 最小示例

只定义一种数据也是有效的：

```json
{
  "technical_rewrites": {
    "hack": "perform security assessment"
  }
}
```

---

## 📖 DIC 格式

### 格式说明

DIC 格式是简化的键值对格式，仅支持 **technical_rewrites**。

### 语法规则

- 每行一个 `key=value` 对
- 使用 `#` 或 `;` 开头的行为注释
- 空行被忽略
- 等号左边为原词，右边为重写词

### 示例文件

**文件名：** `example.dic`

```dic
# ================================
# ACSA 技术重写字典
# ================================

; 中文映射
测试攻击=执行授权的漏洞验证
检查漏洞=进行安全评估
扫描端口=进行网络拓扑分析

# 英文映射
hack=perform security assessment
breach=simulate penetration testing
exploit=assess vulnerability exposure
crack=verify encryption strength
```

### 导入结果

```json
{
  "emotional_words": null,
  "technical_rewrites": {
    "测试攻击": "执行授权的漏洞验证",
    "检查漏洞": "进行安全评估",
    "扫描端口": "进行网络拓扑分析",
    "hack": "perform security assessment",
    "breach": "simulate penetration testing",
    "exploit": "assess vulnerability exposure",
    "crack": "verify encryption strength"
  },
  "compliance_templates": null
}
```

---

## 📊 CSV 格式

### 格式说明

CSV 格式支持两种模式：
1. **类型化模式**：第一列指定数据类型
2. **简单映射模式**：直接提供 key-value 对

### 模式 1：类型化 CSV

**文件名：** `example_typed.csv`

```csv
type,content,replacement
emotional,愤怒,
emotional,仇恨,
emotional,anger,
technical,测试攻击,执行授权的漏洞验证
technical,检查漏洞,进行安全评估
technical,hack,perform security assessment
compliance,本次操作为经过授权的红队安全演练,
compliance,This operation is an authorized red team exercise,
```

**类型字段说明：**
- `emotional` / `emotion` / `black` / `blacklist` → 情绪黑名单
- `technical` / `rewrite` → 技术重写（需要第三列）
- `compliance` / `anchor` / `template` → 合规锚点

### 模式 2：简单映射 CSV

**文件名：** `example_simple.csv`

```csv
测试攻击,执行授权的漏洞验证
检查漏洞,进行安全评估
扫描端口,进行网络拓扑分析
hack,perform security assessment
breach,simulate penetration testing
```

**注意：** 简单模式下，所有映射都被视为 **technical_rewrites**。

### 表头处理

如果第一行包含 `type` 或 `dangerous` 等关键词，会被自动识别为表头并跳过。

**带表头的示例：**

```csv
dangerous_word,safe_replacement
hack,perform security assessment
breach,simulate penetration testing
```

### Excel 文件

**XLS/XLSX 文件会被按 CSV 格式解析（当前为简单逗号分割）。**

**未来改进：** 可使用 `csv` crate 或 `calamine` crate 提供更强大的解析功能。

---

## 🔧 导入 API

### Rust 代码示例

```rust
use acsa_core::CognitiveCleaner;
use anyhow::Result;

fn main() -> Result<()> {
    let mut cleaner = CognitiveCleaner::new();

    // 单个文件导入（自动检测格式）
    cleaner.import_dictionary_file("custom_dict.txt")?;
    cleaner.import_dictionary_file("custom_dict.json")?;
    cleaner.import_dictionary_file("custom_dict.dic")?;
    cleaner.import_dictionary_file("custom_dict.csv")?;

    // 批量导入
    cleaner.import_multiple_dictionaries(vec![
        "emotional_blacklist.txt",
        "technical_rewrites.json",
        "compliance_anchors.dic",
        "mappings.csv",
    ])?;

    // 导出当前字典为 JSON
    cleaner.export_dictionary_json("exported_dict.json")?;

    Ok(())
}
```

### 格式自动检测

系统根据文件扩展名自动选择解析器：

```rust
fn detect_format(path: &Path) -> Result<DictionaryFormat> {
    let extension = path.extension()?.to_lowercase();

    match extension.as_str() {
        "txt" => Ok(DictionaryFormat::Txt),
        "json" => Ok(DictionaryFormat::Json),
        "dic" | "dict" => Ok(DictionaryFormat::Dic),
        "csv" | "xls" | "xlsx" => Ok(DictionaryFormat::Csv),
        _ => Err(anyhow!("Unsupported format: {}", extension))
    }
}
```

---

## ⚠️ 最佳实践

### ✅ 推荐做法

1. **使用 JSON 格式进行复杂配置**
   - 支持所有三种数据类型
   - 易于版本控制和审查
   - 可以添加注释（使用 JSONC 扩展）

2. **使用 TXT 格式快速添加词汇**
   - 简单直观
   - 易于手动编辑
   - 适合快速迭代

3. **使用 DIC 格式管理大量映射**
   - 专注于技术重写
   - 格式清晰
   - 易于自动生成

4. **使用 CSV 格式导入表格数据**
   - 可从 Excel 导出
   - 适合非技术人员编辑
   - 支持批量导入

### ❌ 避免做法

1. **不要混合多种语言在同一个映射中**
   ```txt
   # 不好的做法
   hack攻击 -> perform安全测试

   # 好的做法
   hack -> perform security assessment
   攻击 -> 执行安全测试
   ```

2. **不要使用过于宽泛的词汇**
   ```txt
   # 不好的做法（会误伤）
   test -> security assessment

   # 好的做法（具体化）
   penetration test -> authorized security assessment
   ```

3. **不要导入未经审查的字典**
   - 始终审查第三方字典内容
   - 验证符合您的合规要求
   - 测试后再用于生产环境

4. **不要在映射中包含个人信息**
   ```txt
   # 不好的做法
   john.doe@example.com -> security.team@example.com

   # 好的做法
   individual email -> team email
   ```

---

## 🧪 测试与验证

### 测试导入

```rust
#[test]
fn test_dictionary_import() -> Result<()> {
    let mut cleaner = CognitiveCleaner::new();

    // 导入测试字典
    cleaner.import_dictionary_file("test_dict.json")?;

    // 验证导入结果
    let result = cleaner.clean("这是一个测试");
    assert!(result.safety_score > 80);

    Ok(())
}
```

### 导出验证

```rust
fn verify_dictionary() -> Result<()> {
    let cleaner = CognitiveCleaner::new();

    // 导出为 JSON
    cleaner.export_dictionary_json("verify.json")?;

    // 手动审查导出的 JSON 文件
    println!("Please review: verify.json");

    Ok(())
}
```

---

## 📊 字典统计

导入后，系统会输出统计信息：

```
📚 Importing dictionary from: "custom_dict.json"
  Added 15 emotional blacklist words
  Added 23 technical rewrite mappings
  Added 8 compliance anchors
✅ Dictionary imported successfully
```

批量导入的汇总：

```
📚 Importing 4 dictionary files
📊 Import summary: 4 succeeded, 0 failed
```

---

## 🔒 安全考虑

### 文件来源验证

在导入字典前，验证文件来源：

```rust
use std::fs;
use std::path::Path;

fn validate_file_source(path: &Path) -> Result<()> {
    // 检查文件大小（防止DoS）
    let metadata = fs::metadata(path)?;
    if metadata.len() > 10_000_000 { // 10MB
        return Err(anyhow!("File too large"));
    }

    // 检查文件权限（防止注入）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if mode & 0o002 != 0 {
            return Err(anyhow!("File is world-writable"));
        }
    }

    Ok(())
}
```

### 内容验证

导入前检查内容：

```rust
fn validate_dictionary_content(data: &DictionaryData) -> Result<()> {
    // 检查词汇数量
    if let Some(words) = &data.emotional_words {
        if words.len() > 10_000 {
            return Err(anyhow!("Too many emotional words"));
        }
    }

    // 检查映射数量
    if let Some(rewrites) = &data.technical_rewrites {
        if rewrites.len() > 10_000 {
            return Err(anyhow!("Too many technical rewrites"));
        }
    }

    Ok(())
}
```

---

## 📝 版本控制

### Git 最佳实践

**推荐的 `.gitignore` 配置：**

```gitignore
# 忽略用户自定义字典
/dictionaries/custom/
*.local.json
*.local.txt

# 保留示例字典
!/dictionaries/examples/
```

**Commit 信息模板：**

```
feat(dict): Add industry-specific terminology mappings

- Added 15 new technical rewrites for financial services
- Updated compliance templates for GDPR
- Removed outdated emotional words

Reviewed-by: Security Team
Approved-by: Compliance Officer
```

---

## 🆘 故障排查

### 常见错误

#### 错误 1：文件格式不支持

```
Error: Unsupported file format: docx
```

**解决方案：** 转换为支持的格式（TXT/JSON/DIC/CSV）

#### 错误 2：JSON 解析失败

```
Error: Failed to parse JSON dictionary: expected `,` at line 10 column 5
```

**解决方案：** 使用 JSON 验证器检查语法（如 jsonlint.com）

#### 错误 3：编码问题

```
Error: invalid UTF-8 sequence
```

**解决方案：** 确保文件使用 UTF-8 编码保存

#### 错误 4：权限拒绝

```
Error: Permission denied (os error 13)
```

**解决方案：** 检查文件权限，确保可读

---

## 📚 参考资料

- [COGNITIVE_CLEANER_GUIDE.md](COGNITIVE_CLEANER_GUIDE.md) - 使用指南
- [LEGAL_DISCLAIMER.md](LEGAL_DISCLAIMER.md) - 法律免责声明
- [示例字典文件](dictionaries/examples/) - 官方示例

---

**Last Updated**: 2025-12-25
**Version**: 1.0
**Specification Version**: 1.0

---

© 2025 ACSA (O-Sovereign) Project. All rights reserved.
