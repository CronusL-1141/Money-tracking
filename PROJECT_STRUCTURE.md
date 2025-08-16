# 项目结构说明

## 📁 整理后的项目结构

```
audit-system/  （审计系统根目录）
├── src/                    # 📦 源代码目录
│   ├── main.py            # 原始主程序
│   ├── main_new.py        # 新版主程序（支持双算法）
│   ├── config.py          # 系统配置文件
│   ├── debug_tool.py      # 调试工具
│   ├── requirements.txt   # Python依赖包
│   ├── core/             # 核心架构模块
│   │   ├── interfaces/   # 抽象接口
│   │   ├── trackers/     # 追踪器实现（FIFO+差额计算法）
│   │   └── factories/    # 工厂模式
│   ├── services/         # 服务层
│   │   ├── audit_service.py      # 审计分析服务
│   │   ├── time_point_query_service.py  # 时点查询服务
│   │   └── query_cli.py          # CLI接口
│   ├── utils/           # 工具类
│   │   ├── data_processor.py     # 数据处理器
│   │   ├── logger.py            # 日志工具
│   │   └── validators.py        # 验证器
│   └── models/          # 原始模型（备份兼容）
│       ├── fifo_tracker.py      # FIFO追踪器
│       ├── behavior_analyzer.py # 行为分析器
│       └── investment_manager.py # 投资管理器
│
├── tests/              # 🧪 测试代码目录
│   ├── test_dual_algorithm.py         # 双算法测试
│   ├── test_time_point_query.py       # 时点查询测试
│   ├── test_balance_method_fix.py     # 差额计算法测试
│   ├── test_user_scenario_comparison.py # 用户场景对比测试
│   ├── test_flow_integrity.py         # 流水完整性测试
│   ├── test_basic.py                  # 基础功能测试
│   └── check_*.py / diagnose_*.py     # 调试和诊断工具
│
├── data/              # 📊 数据文件目录
│   ├── input/         # 输入数据
│   │   ├── 流水.xlsx                 # 主要交易流水数据
│   │   └── 投资产品交易记录.xlsx      # 投资产品数据
│   └── output/        # 输出结果
│       ├── FIFO资金追踪结果.xlsx               # FIFO算法结果
│       ├── BALANCE_METHOD_资金追踪结果.xlsx     # 差额计算法结果
│       └── 投资产品交易记录_BALANCE_METHOD.xlsx # 投资产品结果
│
├── legacy/            # 🗂️ 备份代码目录
│   ├── main.py        # 原始主程序备份
│   ├── config.py      # 原始配置备份
│   └── models/        # 原始模型备份
│   └── utils/         # 原始工具备份
│
├── temp/              # 🗃️ 临时文件目录
│   └── 流水验证错误报告.txt    # 错误报告
│
├── logs/              # 📋 日志文件目录
│   ├── audit.log      # 审计日志
│   ├── audit_error.log # 错误日志
│   └── audit_detail.log # 详细日志
│
├── docs/              # 📖 文档目录
│   └── （文档文件）
│
├── 原始数据/           # 📁 原始数据备份
└── .cursor/           # IDE配置目录
```

## 🎯 主要改进

### ✅ 文件分类整理
- **源代码集中**: 所有Python源码移至`src/`目录
- **测试文件分离**: 所有测试文件移至`tests/`目录  
- **数据文件分类**: 输入/输出数据分别存放在`data/input/`和`data/output/`
- **备份完整**: 原始代码完整备份在`legacy/`目录

### ✅ 项目结构优化
- **模块化架构**: 核心代码按功能分层组织
- **清洁根目录**: 根目录只保留分类文件夹和必要文件
- **临时文件管理**: 临时文件和日志分别管理

### ✅ 开发友好
- **易于导航**: 清晰的目录结构便于代码查找
- **测试隔离**: 测试代码独立，不污染源码
- **版本管理**: 新旧代码共存，支持回滚

## 🚀 使用指南

### 运行主程序
```bash
# 运行新版主程序（推荐）
python src/main_new.py --algorithm BALANCE_METHOD --input data/input/流水.xlsx

# 运行原版主程序（兼容）
python src/main.py
```

### 时点查询
```bash
# CLI时点查询
python src/services/query_cli.py --file "data/input/流水.xlsx" --row 100 --algorithm BALANCE_METHOD

# 交互模式
python src/services/query_cli.py --file "data/input/流水.xlsx" --interactive
```

### 运行测试
```bash
# 运行双算法测试
python tests/test_dual_algorithm.py

# 运行时点查询测试
python tests/test_time_point_query.py
```

## 📋 注意事项

1. **路径调整**: 由于文件移动，导入路径可能需要调整
2. **配置更新**: 配置文件中的路径引用需要更新
3. **依赖安装**: 运行前确保安装`src/requirements.txt`中的依赖
4. **日志目录**: 确保`logs/`目录有写入权限

---
*整理日期: 2025-01-13*  
*整理目标: 创建清洁、模块化的项目结构*