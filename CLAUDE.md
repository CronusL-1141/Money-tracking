# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**涉案资金追踪分析系统** - 司法审计工具，用于检测公款挪用、职务侵占等经济犯罪行为。

### 当前状态（2024年8月23日）
- **生产版本**: Python v3.1.0 - 功能完整且稳定运行中
- **开发中**: Rust后端迁移项目 - 工具层完成，**专注算法层逐层完善**
- **GUI状态**: Tauri应用通过Shell调用Python CLI，尚未集成Rust后端
- **开发重点**: 基础→工具→算法 三层逐层完善，不急于实现应用层

### 技术栈
- **当前生产**: Python 3.11+ (CLI) + Tauri (Rust壳 + React前端，调用Python)
- **迁移目标**: Rust后端 + Tauri (直接集成)
- **数据处理**: pandas → Rust原生实现 + 流水完整性修复算法
- **Excel处理**: openpyxl → calamine(读) + rust_xlsxwriter(写)

## 版本控制

### 分支结构
- **main**: 完整的Python实现，生产版本，功能完整且稳定
- **rust-backend**: Rust后端迁移开发分支（当前工作分支）
- **trial-portable**: 便携版尝试分支

### 当前工作分支
```bash
# 检查当前分支
git branch

# 当前应该在 rust-backend 分支
# 如果不在，切换到正确分支：
git checkout rust-backend
```

### 重要提醒
- **main分支**: 不要修改，这是稳定的Python生产版本
- **rust-backend分支**: 在此分支进行所有Rust迁移工作
- **合并策略**: Rust迁移完成后才合并回main

## 常用命令

### Python版本（main分支生产）
```bash
# FIFO算法分析
python src/main.py -a FIFO -i 流水.xlsx

# 差额计算法分析  
python src/main.py -a BALANCE_METHOD -i 流水.xlsx

# 时点查询
python src/services/query_cli.py -f 流水.xlsx -r 100 -a BALANCE_METHOD

# 运行测试
python -m pytest tests/ -v
```

### Rust后端（rust-backend分支开发）
```bash
# 进入Rust后端目录
cd rust-backend

# 检查编译
cargo check --lib

# 构建库
cargo build --release

# 运行测试（当前需完善）
cargo test

# 注意：二进制程序暂时禁用（Cargo.toml中已注释）
```

### Tauri GUI
```bash
cd tauri-app

# 开发模式（使用Python后端）
npm run tauri:dev

# 构建发布版
npm run tauri:build

# 前端开发服务器
npm run dev
```

## 高层架构

### 当前架构（main分支生产环境）
```
用户界面
├── CLI: Python直接执行
└── GUI: Tauri → Shell.Command → Python CLI → 返回结果

数据流:
Excel文件 → Python(pandas) → 算法处理 → Excel/CSV输出
```

### 目标架构（rust-backend分支迁移后）
```
用户界面
├── CLI: Rust二进制
└── GUI: Tauri → Rust库直接调用 → 返回结果

数据流:
Excel文件 → Rust(calamine读取) → 数据验证修复 → 算法处理 → Excel输出(rust_xlsxwriter)
```

### 支持的算法
- **FIFO（先进先出）**: 按时间顺序追踪资金流向
- **BALANCE_METHOD（差额计算法）**: 基于余额变化计算

**重要**: 不需要实现算法对比功能，每次分析只使用单一算法。

## Rust迁移状态（rust-backend分支）

### 当前进度 - 逐层完善阶段

#### ✅ 基础层（已完成）
- ✅ 核心数据结构（Transaction, Config, AuditSummary）
- ✅ 错误处理系统（AuditError with thiserror）
- ✅ 库编译成功

#### ✅ 工具层（已完成）
- ✅ Excel处理模块（ExcelProcessor - 统一读写功能）
- ✅ 统一数据验证器（UnifiedValidator - 流水完整性验证和修复）
- ✅ 时间处理器（TimeProcessor - 时间解析和格式化）
- ✅ 日志系统（AuditLogger - 结构化日志）
- ✅ 数据不变性原则修复（确保源数据只读）

#### 🔄 算法层（当前重点）
- 🔄 FIFO算法完善（核心逻辑已有，需完善细节）
- 🔄 BalanceMethod算法完善（核心逻辑已有，需完善细节）
- ⏳ FlowAnalyzer重新设计（功能分离）
- ⏳ TrackerFactory实现（算法工厂）
- ⏳ 算法层集成测试
- ⏳ 与Python版本结果对比验证

#### ⏸️ 暂缓的层次（等算法层完成后）
- **服务层**（AuditService等） - 为GUI交互设计，当前阶段不需要
- **应用层**（CLI/GUI） - 依赖算法层完成
- **Tauri集成** - 依赖服务层完成

## 核心设计原则

### 数据处理流水线（关键）
```
原始Excel文件（只读，永不修改）
    ↓
[1] Excel读取层 (ExcelProcessor::read_transactions)
    ↓
原始Transaction数据（内存中）
    ↓
[2] 数据验证和修复层 (UnifiedValidator)
    ├─ 基础验证：必需列、数据格式
    ├─ 流水完整性验证：余额连贯性检查
    └─ 自动修复：同时间交易重排序（贪心算法）
    ↓
清洁Transaction数据（修复后，用于后续处理）
    ↓
[3] 业务算法层 (FIFO/BalanceMethod)
    ├─ 资金流向分析
    ├─ 挪用垫付计算
    └─ 投资产品处理
    ↓
分析结果数据（AuditSummary + 增强Transaction）
    ↓
[4] 结果输出层 (ExcelProcessor::export_analysis_results)
```

### 数据不变性保证
1. **源文件保护**：原始Excel文件在整个过程中保持只读，绝不修改
2. **内存副本处理**：所有验证和修复都在内存副本上进行
3. **数据流单向性**：数据只能向下游传递，不能回流修改上游
4. **修复数据标准**：只有经过完整性验证和修复的数据才能进入业务算法
5. **清洁数据保证**：业务逻辑只处理经过验证修复的清洁数据

### 模块职责分离
- **ExcelProcessor**: 只负责IO，不修改业务数据
- **UnifiedValidator**: 专职数据质量保证和修复
- **Algorithm层**: 只处理已验证的清洁数据
- **Service层**: 协调各层，不直接处理数据

## 技术决策记录

### Excel处理策略（已更新）
- **问题**: xlsxwriter Rust版本API不兼容
- **决策**: 统一ExcelProcessor，calamine读取 + rust_xlsxwriter写入
- **原因**: 提供完整Excel读写功能，支持格式化输出

### 数据验证策略（新增）
- **移除**: 大额交易验证、日期范围验证（用户明确要求）
- **核心**: 流水完整性验证和自动修复算法
- **修复**: 贪心策略重排序同时间交易，保证余额连贯性
- **原则**: 一边验证一边修复，不修改源数据

### 依赖管理
- **已移除**: argminmax（编译错误）、brotli、memmap2（暂不需要）
- **核心依赖**: calamine(0.26), rust_xlsxwriter(0.76), rust_decimal, chrono, tokio

### 算法实现
- FIFO: 使用VecDeque实现队列
- BalanceMethod: 直接余额计算
- 资金池: HashMap管理多个投资产品
- 数据修复: 贪心搜索 + 余额连贯性检查

## 开发注意事项

### 关键约定
1. **响应语言**: 使用中文回复和注释
2. **Shell环境**: PowerShell 7（Windows）
3. **文件创建**: 避免创建不必要的文档文件，优先修改现有文件
4. **测试数据**: 使用"流水.xlsx"作为标准测试文件
5. **功能范围**: 不实现算法对比功能，专注单一算法分析
6. **工作分支**: 始终在rust-backend分支工作，不要修改main分支
7. **数据不变性**: 源数据文件绝对不能修改，所有处理在内存副本进行
8. **数据质量**: 业务算法只处理经过验证和修复的清洁数据

### 开发原则（重要发现）
9. **逐层完善**: 基础层→工具层→算法层，每层完善后再进入下一层
10. **架构理解**: Services层是GUI交互层，不是当前开发重点
11. **阶段专注**: 当前专注算法层完善，不急于实现CLI/GUI应用层
12. **功能验证**: 每层完成后需与Python版本对比验证正确性

### 代码风格
- Python: PEP 8，类型注解
- Rust: 标准rustfmt配置
- 避免添加过多注释，代码应自解释

### 关键文件路径
- 主Excel数据: `流水.xlsx`
- Python入口: `src/main.py`
- Rust库入口: `rust-backend/src/lib.rs`
- Tauri配置: `tauri-app/src-tauri/tauri.conf.json`
- 测试目录: `tests/` (Python), `rust-backend/tests/` (Rust)

## 问题追踪

### 当前已知问题（rust-backend分支）
1. ✅ AuditService数据不变性问题（已修复）
2. FIFO算法细节需完善（与Python版本对比）
3. BalanceMethod算法细节需完善
4. TrackerFactory算法工厂尚未实现
5. FlowAnalyzer功能分离尚未重新设计
6. 算法层集成测试覆盖不完整

## 最新架构设计（Rust版本）

### 模块层次结构
```
rust-backend/src/
├── lib.rs                    # 库入口
├── errors/                   # 错误处理（基础层）
│   └── mod.rs
├── models/                   # 数据模型（基础层）
│   ├── transaction.rs        # ✅ 交易记录
│   ├── config.rs            # ✅ 配置管理
│   ├── audit_summary.rs     # ✅ 审计摘要
│   └── fund_pool.rs         # ✅ 资金池管理
├── utils/                    # 工具层（已基本完成）
│   ├── excel_processor.rs    # ✅ 统一Excel读写
│   ├── unified_validator.rs  # ✅ 数据验证修复
│   ├── time_processor.rs     # ✅ 时间处理
│   └── logger.rs            # ✅ 日志系统
├── algorithms/               # 算法层（当前重点）
│   ├── fifo_tracker.rs      # 🔄 FIFO算法（需完善细节）
│   ├── balance_method_tracker.rs # 🔄 差额计算法（需完善细节）
│   ├── flow_analyzer.rs     # ⏳ 流向分析器（需重新设计）
│   └── tracker_factory.rs   # ⏳ 算法工厂（待实现）
├── services/                 # 服务层（暂缓，GUI交互层）
│   ├── audit_service.rs     # ⏸️ 主审计服务（为GUI设计）
│   └── integration_processor.rs # ⏸️ 集成处理器（为GUI设计）
└── bin/                      # 应用层（暂缓，依赖算法层）
    └── cli.rs               # ⏸️ 命令行工具（等算法层完成）
```

### 数据流向图
```
用户输入(流水.xlsx)
    ↓
┌─────────────────────────────┐
│   ExcelProcessor::read      │  [工具层]
│   - calamine读取           │
│   - 创建Transaction向量     │
└─────────────┬───────────────┘
              ↓
┌─────────────────────────────┐
│   UnifiedValidator          │  [工具层]
│   - 必需列验证             │
│   - 流水完整性检查         │
│   - 贪心算法自动修复       │
│   - 返回清洁数据           │
└─────────────┬───────────────┘
              ↓
┌─────────────────────────────┐
│   FIFO/BalanceMethod        │  [算法层]
│   - 资金流向分析           │
│   - 挪用垫付计算           │
│   - 投资产品处理           │
└─────────────┬───────────────┘
              ↓
┌─────────────────────────────┐
│   AuditService             │  [服务层]
│   - 协调各层               │
│   - 生成审计摘要           │
└─────────────┬───────────────┘
              ↓
┌─────────────────────────────┐
│   ExcelProcessor::export    │  [工具层]
│   - rust_xlsxwriter写入     │
│   - 格式化结果输出         │
└─────────────────────────────┘
    ↓
输出文件(分析结果.xlsx)
```

### 重要提醒
- **分支管理**: 确保在rust-backend分支工作，main分支保持稳定
- rust-backend目录是**今天新建**的项目，不要被旧文档误导
- Python版本（main分支）是**当前生产版本**，功能完整
- 迁移是**渐进式**的，不要急于删除Python代码
- **不需要算法对比功能**，每个分析任务使用单一算法即可
- **数据不变性原则**: 源文件只读，修复在内存进行，业务算法只处理清洁数据