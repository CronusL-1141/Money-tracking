# 🚀 快速开始指南

## 📁 项目结构（已整理）

```
审计系统/
├── src/           # 📦 源代码（核心功能）
├── tests/         # 🧪 测试文件（验证功能）
├── data/          # 📊 数据文件
│   ├── input/     # 输入数据：流水.xlsx, 投资产品交易记录.xlsx
│   └── output/    # 输出结果：分析结果文件
├── legacy/        # 🗂️ 原始代码备份
├── temp/          # 🗃️ 临时文件
├── logs/          # 📋 系统日志
└── docs/          # 📖 项目文档
```

## ⚡ 使用方法

### 1. 双算法分析（推荐）
```bash
# FIFO先进先出算法
python src/main_new.py --algorithm FIFO --input data/input/流水.xlsx

# 差额计算法（余额优先）
python src/main_new.py --algorithm BALANCE_METHOD --input data/input/流水.xlsx

# 对比两种算法结果
python src/main_new.py --compare --input data/input/流水.xlsx
```

### 2. 时点查询功能
```bash
# 查询第100行的系统状态
python src/services/query_cli.py --file data/input/流水.xlsx --row 100 --algorithm BALANCE_METHOD

# 启动交互模式
python src/services/query_cli.py --file data/input/流水.xlsx --interactive
```

### 3. 运行测试验证
```bash
# 验证双算法架构
python tests/test_dual_algorithm.py

# 验证时点查询
python tests/test_time_point_query.py

# 验证差额计算法修复
python tests/test_balance_method_fix.py
```

## 🎯 核心功能

### ✅ 已完成功能
1. **双算法支持**：FIFO + 差额计算法
2. **时点查询**：查询任意交易行的系统状态
3. **历史记录**：最多保存100条查询历史
4. **导出功能**：支持JSON/Excel格式
5. **CLI接口**：命令行和交互式操作
6. **完整测试**：所有功能验证通过

### 🔧 算法对比
| 特性 | FIFO算法 | 差额计算法 |
|------|----------|------------|
| **逻辑** | 先进先出队列 | 余额优先扣除 |
| **个人支出** | 按队列顺序 | 个人余额优先 |
| **公司支出** | 按队列顺序 | 公司余额优先 |
| **挪用计算** | 队列追溯 | 直接计算 |
| **性能** | 复杂 O(n) | 简单 O(1) |

## 📋 注意事项

1. **数据文件**：确保`data/input/`中有正确的Excel文件
2. **Python版本**：建议使用Python 3.11+
3. **依赖安装**：`pip install -r src/requirements.txt`
4. **路径问题**：已修复所有导入路径，支持从根目录运行

## 🎉 快速验证

```bash
# 1. 查看可用算法
python src/main_new.py --list-algorithms

# 2. 快速分析（使用差额计算法）
python src/main_new.py -a BALANCE_METHOD -i data/input/流水.xlsx

# 3. 查看结果
ls data/output/
```

---
*整理完成时间：2025-01-13*  
*项目状态：Phase 2完成，代码结构已优化* ✅