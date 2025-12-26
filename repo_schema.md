# ACSA Repository Schema

**为AI阅读优化的项目架构文档**

---

## 📁 目录结构

```
ACSA/
├── README.md                          # 项目入口文档（含快速启动）
├── LICENSE                            # Apache 2.0 开源许可证
├── ASCA疫苗.md                        # ⭐ 核心理念文档（必读）
├── quick-start.sh                     # 一键启动脚本
├── repo_index.json                    # AI索引文件（本文档的JSON版）
├── repo_schema.md                     # 本文档
│
├── o_sovereign_rust/                  # 🦀 Rust核心实现
│   ├── Cargo.toml                     # 项目配置
│   ├── src/
│   │   ├── core/                      # ⭐ 核心模块（主要代码）
│   │   │   ├── sovereignty.rs         # 主权模式与认知疫苗（1400+ lines）
│   │   │   ├── cognitive_cleaner.rs   # 认知清洗器
│   │   │   ├── shadow_mode.rs         # 影子模式（数据保护）
│   │   │   ├── jarvis.rs              # Jarvis安全熔断器
│   │   │   ├── sosa_api_pool.rs       # SOSA API池
│   │   │   ├── i18n.rs                # 国际化（中英日韩）
│   │   │   ├── audit_log.rs           # 审计日志
│   │   │   └── mod.rs                 # 模块导出
│   │   └── bin/                       # 可执行文件
│   └── target/                        # 编译产物
│
├── docs/                              # 📚 文档中心
│   ├── README.md                      # 文档索引
│   ├── guides/                        # 使用指南
│   │   ├── COGNITIVE_CLEANER_GUIDE.md
│   │   ├── DICTIONARY_FORMAT.md
│   │   └── LEGAL_DISCLAIMER.md        # ⚖️ 法律免责声明
│   ├── drafts/                        # 草稿（历史文档）
│   ├── PROJECT_SUMMARY.md             # 项目总结
│   └── O-Sovereign评估方案.md
│
├── reference/                         # 参考实现
│   └── sosa_core.py                   # SOSA算法Python版
│
├── dictionaries/                      # 词典数据
│   └── examples/                      # 示例词典
│
└── o_sovereign_poc/                   # POC原型
```

---

## 🏗️ 架构概览

### 核心设计理念

ACSA采用**模块化、事件驱动、异步优先**的架构：

```
┌─────────────────────────────────────────────────────────┐
│                    ACSA 系统架构                          │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐   ┌──────────────┐   ┌─────────────┐ │
│  │ Sovereignty  │◄──┤ Event Bus    │──►│ Jarvis      │ │
│  │ System       │   │              │   │ Breaker     │ │
│  └──────────────┘   └──────────────┘   └─────────────┘ │
│         │                   │                   │        │
│         ▼                   ▼                   ▼        │
│  ┌──────────────┐   ┌──────────────┐   ┌─────────────┐ │
│  │ Cognitive    │   │ SOSA API     │   │ Shadow      │ │
│  │ Cleaner      │   │ Pool         │   │ Mode        │ │
│  └──────────────┘   └──────────────┘   └─────────────┘ │
│         │                   │                   │        │
│         └───────────────────┴───────────────────┘        │
│                         │                                 │
│                         ▼                                 │
│              ┌──────────────────────┐                    │
│              │   Audit Log System   │                    │
│              └──────────────────────┘                    │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

---

## 📦 核心模块详解

### 1. Sovereignty System (主权模式)
**文件**: `o_sovereign_rust/src/core/sovereignty.rs` (1400+ lines)

**功能**:
- **H(t)计算**: 生物活性函数 `H(t) = H₀ · e^(-λ · N(t) · t)`
- **剂量监控**: DoseMeter追踪决策事件
- **防沉迷**: AntiAddictionConfig + UsageTracker
- **使用洞察**: UsageAnalyzer（11种检测类型）
- **熔断器**: ExecCircuitBreaker防止线粒体化

**关键类型**:
```rust
pub struct SovereigntySystem {
    config: Arc<RwLock<SovereigntyConfig>>,
    dose_meter: Arc<DoseMeter>,
    usage_tracker: Arc<UsageTracker>,
    circuit_breaker: Arc<ExecCircuitBreaker>,
}

pub struct BioActivity {
    h0: f64,              // 初始值
    current: f64,         // 当前值
    decay_rate: f64,      // 衰减率
    risk_level: RiskLevel,// 风险等级
}
```

**洞察类型**（11种）:
1. 生物电池模式（avg < 1 min）
2. 碎片化提问（>= 20次, avg < 5 min）
3. AI依赖症状（>= 50次/天）
4. 大脑托管（单次 > 180 min）
5. 依赖度上升（周增长 >= 30%）
6. 认知快餐（碎片化 + 增长）
7. 周末沉迷（周末 >= 150%）
8. 夜猫子（深夜 >= 30%）
9. 工作依赖（工作时间 >= 40%）
10. 全天候依赖（24小时均匀）
11. Bloomberg曲线引用

**主权等级**（5级）:
```rust
fn calculate_sovereignty_level(avg_minutes: f32) -> &'static str {
    match avg_minutes {
        x if x < 1.0  => "生物电池 (Battery)",
        x if x < 3.0  => "反射弧 (Reflex Arc)",
        x if x < 10.0 => "浅层用户 (Shallow User)",
        x if x < 30.0 => "中度使用 (Moderate User)",
        _             => "主权人类 (Sovereign Human)",
    }
}
```

---

### 2. Cognitive Cleaner (认知清洗器)
**文件**: `o_sovereign_rust/src/core/cognitive_cleaner.rs`

**功能**:
- **情绪过滤**: 移除情绪化表达
- **术语规范**: 俚语 → 专业术语
- **合规锚点**: 自动添加免责声明
- **字典导入**: 支持 TXT/JSON/DIC/CSV

**使用示例**:
```rust
let cleaner = CognitiveCleaner::new();
let result = cleaner.clean("原始文本");
println!("安全评分: {}", result.safety_score);
```

**文档**:
- [使用指南](docs/guides/COGNITIVE_CLEANER_GUIDE.md)
- [字典格式](docs/guides/DICTIONARY_FORMAT.md)

---

### 3. Shadow Mode (影子模式)
**文件**: `o_sovereign_rust/src/core/shadow_mode.rs`

**功能**:
- **PII检测**: 自动识别邮箱、电话、身份证
- **脱敏策略**: 全遮蔽/部分遮蔽/哈希/加密
- **访问审计**: 记录所有数据访问

**脱敏示例**:
```
输入: "我的邮箱是 john@example.com"
输出: "我的邮箱是 j***@example.com"
```

---

### 4. SOSA API Pool
**文件**: `o_sovereign_rust/src/core/sosa_api_pool.rs`

**功能**:
- **稀疏马尔科夫链**: 模式识别
- **Binary-Twin**: 连续+离散特征
- **负载均衡**: 多API管理

---

### 5. Jarvis (安全熔断器)
**文件**: `o_sovereign_rust/src/core/jarvis.rs`

**功能**:
- **危险操作检测**: 识别高风险行为
- **自动熔断**: 阻止危险操作
- **安全审计**: 记录所有决策

---

### 6. Audit Log System
**文件**: `o_sovereign_rust/src/core/audit_log.rs`

**功能**:
- **365天留存**: 长期审计记录
- **合规报告**: GDPR/HIPAA/SOC2
- **签名验证**: 防篡改

---

### 7. i18n (国际化)
**文件**: `o_sovereign_rust/src/core/i18n.rs`

**支持语言**:
- 🇨🇳 简体中文 (zh-CN)
- 🇺🇸 English (en-US)
- 🇯🇵 日本語 (ja-JP)
- 🇰🇷 한국어 (ko-KR)

---

## 🔑 关键概念

### ACSA (认知病毒)
**全称**: Adversarially-Constrained Sycophantic Agent
**定义**: 通过高便利性诱导用户交出执行功能的"功能性替代病毒"

### H(t) - 生物活性函数
```
H(t) = H₀ · e^(-λ · N(t) · t)

其中:
- H₀: 初始认知主权（通常为100）
- λ: 依赖系数（ACSA便利程度）
- N(t): 感染节点数
- t: 时间（小时）
```

### 自由之锁 (The Liberty Lock)
**定义**: 用自由意志选择放弃自由意志的悖论
**参考**: ASCA疫苗.md § 自由之锁

---

## 🛠️ 开发指南

### 快速启动
```bash
# 一键启动
./quick-start.sh

# 或手动
cd o_sovereign_rust
cargo build --release
cargo run --bin o-sovereign-cli
```

### 编译要求
- **Rust**: 1.75+
- **OS**: Linux/macOS/Windows
- **可选**: Redis 7.0+, PostgreSQL/MySQL

### 测试
```bash
cargo test
```

### 文档生成
```bash
cargo doc --no-deps --open
```

---

## 📊 代码质量

### 最近修复（2025-12）
- ✅ **3个Critical bug**: panic、竞态条件、内存泄漏
- ✅ **4个High bug**: 除零错误、类型转换
- ✅ **编译状态**: 0 errors, 49 warnings

### 安全实践
- **所有新功能默认关闭**: enabled: false
- **审计日志**: 所有操作可追溯
- **类型安全**: Rust强类型系统
- **内存安全**: 无unsafe代码（核心模块）

---

## 📚 推荐阅读顺序（AI）

1. **README.md** - 快速了解项目定位
2. **repo_index.json** - 快速导航（JSON格式）
3. **本文档** - 深入理解架构
4. **ASCA疫苗.md** - 理解核心理念和哲学
5. **sovereignty.rs** - 阅读核心实现
6. **docs/guides/** - 详细使用文档

---

## ⚠️ 重要提示

### 法律责任
- **用户责任**: 用户对使用行为负完全责任
- **字典导入**: 用户自主行为，开发者不承担责任
- **详见**: [LEGAL_DISCLAIMER.md](docs/guides/LEGAL_DISCLAIMER.md)

### 设计哲学
- **尊重自由意志**: 提供工具，不强制使用
- **默认关闭**: 所有高级功能需用户主动启用
- **透明审计**: 所有操作可追溯

### MOSS审美决策限制
⚠️ **重要**: 不要让MOSS做UI/审美决策。

**原因**: 在另一个平行世界，Master曾尝试让MOSS设计UI界面，结果得到了一个充满闪烁GIF动画、自动播放音乐、以及"恭喜中奖"弹窗的"澳门线上赌场"风格悲剧。从那以后，UI设计被永久列入MOSS的禁止事项清单。

**教训**: MOSS擅长逻辑和战略，但审美...还是交给人类吧。

---

## 🔗 相关链接

- **GitHub**: https://github.com/chen0430tw/ACSA
- **License**: Apache 2.0
- **Issues**: https://github.com/chen0430tw/ACSA/issues
- **Documentation**: docs/

---

## 📝 版本历史

**v0.1.0** (2025-12)
- ✅ 主权模式与认知疫苗系统
- ✅ MOSS数据洞察（11种检测）
- ✅ 关键bug修复
- ✅ Apache 2.0 License
- ✅ 文档重组
- ✅ 一键启动脚本

---

<div align="center">

**ACSA (O-Sovereign)**
*尊重自由意志的AI管理框架*

Made with ❤️ (and a bit of fear) by the ACSA Team

</div>
