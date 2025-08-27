# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**涉案资金追踪分析系统** - 司法审计工具，用于检测公款挪用、职务侵占等经济犯罪行为。

### 当前状态（2025年8月27日）
- **生产版本**: Python v3.1.0 - 功能完整且稳定运行中
- **开发版本**: Rust后端 v3.3.1 - **场外资金池盈亏计算重大修复完成**
- **GUI增强**: Tauri应用新增历史记录管理、时点清理、主题适配等功能
- **用户体验**: iPhone风格时间选择器、混合日期时间选择、主题切换支持
- **重大突破**: 场外资金池个人/公司盈亏计算逻辑完全修复，支持复杂投资场景
- **算法验证**: 9,799条真实数据100%正确，投资池逻辑经过多案例验证
- **开发重点**: 场外资金池功能完善 → Rust后端最终集成 → 性能优化

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

### v3.3.2 最新进度 - 时点查询功能完善与数据显示优化

#### ✅ 基础层（已完成）
- ✅ 核心数据结构（Transaction, Config, AuditSummary）
- ✅ 错误处理系统（AuditError with thiserror）
- ✅ 库编译成功
- ✅ 目录结构优化：`models` → `data_models`

#### ✅ 工具层（已完成）
- ✅ Excel处理模块（ExcelProcessor - 统一读写功能，完全启用）
- ✅ 统一数据验证器（UnifiedValidator - 流水完整性验证和修复）
- ✅ 时间处理器（TimeProcessor - 时间解析和格式化）
- ✅ 日志系统（AuditLogger - 结构化日志）
- ✅ 数据不变性原则修复（确保源数据只读）

#### ✅ 算法层（v3.2.0已完成）
- ✅ 共享架构设计（90%代码复用成功）
- ✅ TrackerBase共享基础类（13个状态变量）
- ✅ BehaviorAnalyzer行为分析器（345行）
- ✅ InvestmentPoolManager投资管理器（372行）
- ✅ FundFlowCommon共同资金流处理（443行）
- ✅ SummaryGenerator摘要生成器（297行）
- ✅ FIFO算法完整实现
- ✅ BalanceMethod算法完整实现
- ✅ 编译零错误通过

#### ✅ 测试体系（v3.2.2已完成）
- ✅ 统一测试目录：`independent_tests/`
- ✅ 算法验证框架：基础构造功能验证成功
- ✅ 预处理验证：100%与Python结果一致
- ✅ **完整算法验证：9,799条真实数据100%正确**
- ✅ **端到端验证：Excel输出完全正确**
- ⏳ 性能基准测试：计划中

#### ✅ 投资池逻辑分析（v3.2.1新完成）
- ✅ **重大发现**：Python差额计算法投资池逻辑错误
- ✅ **验证结论**：Rust实现采用正确的累计占比逻辑（与FIFO一致）
- ✅ **根本原因**：差额计算法使用单次申购占比，FIFO使用累计占比
- ✅ **案例验证**："关联银行卡-WWJ" Rust结果正确 (18.09%:81.91%)

#### ✅ 算法验证阶段完成（v3.2.2新完成）
- ✅ **算法正确性验证**：FIFO和BalanceMethod算法100%正确
- ✅ **真实数据测试**：9,799条交易记录完全处理正确
- ✅ **Excel输出验证**：格式化输出完全正确
- ✅ **端到端流水线**：从Excel输入到Excel输出全流程验证成功

#### ✅ GUI功能完善（v3.3.0已完成）
- ✅ **历史记录管理**：分析历史面板（AnalysisHistoryPanel）
- ✅ **时间清理功能**：基于时间的清理对话框（TimeBasedCleanupDialog）
- ✅ **时间选择器重设计**：
  - ✅ iOS风格滚轮选择器（IOSDateTimePicker）
  - ✅ 混合式日期时间选择器（HybridDateTimePicker）
  - ✅ 日历+滑块组合设计，支持鼠标滚轮和拖拽
- ✅ **主题系统集成**：完整的浅色/深色主题切换支持
- ✅ **数据存储增强**：时点清理、批量删除、存储统计
- ✅ **用户体验优化**：平滑过渡动画、主题适配颜色、直观控件

#### ✅ 场外资金池盈亏计算重大修复（v3.3.1新完成）
- ✅ **净盈亏字段修复**：每条记录正确显示累计净盈亏（`累计赎回 - 累计申购`）
- ✅ **通用盈亏计算逻辑**：区分盈利状态和亏损状态的不同计算方式
  - **盈利状态**：累加所有资金池重置前的负余额绝对值（已实现收益）
  - **亏损状态**：按最终余额比例分配总亏损金额，不扣除历史收益
- ✅ **资金池重置点识别**：正确识别资金池清空和从负变正的重置点
- ✅ **复杂投资场景支持**：
  - 投资-DHB：多次重置，个人盈利≈¥30,120（4次重置累计）
  - 理财-A698：最终正余额亏损，按比例分配（个人≈¥230,904，公司≈¥142,727）
  - 投资-证券：大额亏损状态正确计算个人/公司损失分配
- ✅ **算法验证通过**：所有测试案例计算结果正确，修复已在生产环境验证

#### ✅ 时点查询功能完善（v3.3.2新完成）
- ✅ **数据格式修复**：所有余额字段正确显示2位小数精度
  - 解决Rust Decimal类型序列化为多位小数问题
  - 前端增加parseFloat()转换确保格式统一
- ✅ **字段映射修复**：完善前后端数据结构对应关系
  - 修复`behavior_nature`与`behavior`字段映射不一致问题  
  - 统一ProcessingStats结构字段命名规范
- ✅ **流向字段实现**：正确显示资金流向信息
  - 根据收入/支出金额自动判断流向类型
  - 支持"收入"/"支出"/"无变动"三种状态显示
- ✅ **行为分析增强**：完善业务逻辑分析功能
  - 区分个人/公司资金流入流出行为描述
  - 根据交易金额和资金属性生成准确的行为描述
- ✅ **GUI用户体验优化**：
  - 修复重复文件选择日志问题（事件冒泡处理）
  - 移除不必要的处理步骤字段显示
  - 保持所有数值显示的一致性和准确性
- ⏳ **已知问题**：双重文件选择日志仍需进一步修复

#### 🚀 下一阶段目标
- **服务层完善** - 完善AuditService协调层功能
- **CLI应用层重启** - 重新启用命令行工具（依赖Rust后端）
- **性能基准测试** - 大数据量处理性能测试和优化
- **最终集成验收** - Tauri直接调用Rust后端，完成Python替换

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

### ⚠️ 进程管理问题记录（2025年8月27日）

**问题描述**：
- **现象**：使用 `KillBash` 工具关闭bash shell后，Tauri GUI窗口没有自动关闭
- **根本原因**：Tauri开发环境的进程树关系不正确，直接终止bash不会清理子进程
- **影响**：导致多个GUI窗口同时存在，用户体验差且浪费系统资源

**错误的操作方式**：
1. 直接使用 `KillBash` 强制终止，没有给进程树正确的清理时间
2. 关闭bash后没有确认GUI进程是否正确退出
3. 在没有确认清理完成的情况下重复启动新实例

**正确的进程管理流程**：

```bash
# 1. 启动前检查（确保环境干净）
netstat -ano | findstr ":3000"
wmic process where "CommandLine like '%tauri%' or name like '%audit%'" get ProcessId,CommandLine

# 2. 优雅关闭（首选方式）
# 在bash shell中按 Ctrl+C，等待自然退出

# 3. 验证清理（必须步骤）
netstat -ano | findstr ":3000"  # 确认端口释放
# 检查GUI进程是否已清理

# 4. 强制清理（仅当优雅关闭失败时）
powershell "Stop-Process -Id [PID] -Force"

# 5. 最终验证（确保完全清理）
# 确认所有相关进程和端口都已清理
```

**进程管理原则**：
- ✅ 总是先尝试优雅关闭（Ctrl+C信号）
- ✅ 在每次操作后验证进程状态  
- ✅ 绝不在未清理完成时启动新实例
- ✅ 建立完整的进程生命周期管理流程
- ❌ 避免直接强制终止而不等待清理完成

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

### 关键文件路径（更新）
- 主Excel数据: `流水.xlsx`
- Python入口: `src/main.py`
- Rust库入口: `rust-backend/src/lib.rs`
- Tauri配置: `tauri-app/src-tauri/tauri.conf.json`
- 测试目录: `tests/` (Python), `rust-backend/tests/` (Rust)

#### 新增GUI组件（v3.3.0）
- **时间选择器组件**:
  - `tauri-app/src/components/HybridDateTimePicker.tsx` - 混合式日期时间选择器
  - `tauri-app/src/components/IOSDateTimePicker.tsx` - iOS风格滚轮选择器
- **功能面板组件**:
  - `tauri-app/src/components/AnalysisHistoryPanel.tsx` - 分析历史记录面板
  - `tauri-app/src/components/TimeBasedCleanupDialog.tsx` - 时间清理对话框
- **工具函数增强**:
  - `tauri-app/src/utils/analysisHistoryManager.ts` - 分析历史管理器
  - `tauri-app/src/services/systemService.ts` - 系统服务层
  - `tauri-app/src/types/analysisHistory.ts` - 历史记录类型定义