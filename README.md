# 涉案资金追踪分析系统

> **专业的司法审计工具 | Professional Judicial Audit Tool**

[![Python](https://img.shields.io/badge/Python-3.11+-blue.svg)](https://www.python.org/)
[![Tauri](https://img.shields.io/badge/Tauri-1.5+-green.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18+-61dafb.svg)](https://reactjs.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## 🎯 项目简介

涉案资金追踪分析系统是一套专业的司法审计工具，用于检测和分析公款挪用、职务侵占等经济犯罪行为。系统采用现代化的模块化架构，支持多种分析算法，提供命令行和GUI双重操作界面。

### ✨ 核心特性

- 🧮 **双算法引擎**: FIFO先进先出 + 差额计算法
- 🔍 **时点查询**: 查询任意时点的系统状态  
- 💰 **场外资金池**: 投资产品资金追踪与盈亏分析
- ✅ **流水完整性**: 自动检测修复数据问题
- 🖥️ **多界面支持**: CLI命令行 + GUI桌面应用
- 📊 **实时分析**: 支持实时日志输出和进度跟踪

## 🚀 快速开始

### 环境要求
- Python 3.11+
- Node.js 18+ (GUI应用)
- Rust 1.70+ (GUI应用)

### 安装依赖
```bash
# 安装Python依赖
cd src/
pip install -r requirements.txt

# 安装GUI依赖 (可选)
cd ../tauri-app/
npm install
```

### 基本使用

#### CLI版本
```bash
# FIFO算法分析
python src/main_new.py -a FIFO -i data/input/流水.xlsx

# 差额计算法分析
python src/main_new.py -a BALANCE_METHOD -i data/input/流水.xlsx

# 对比两种算法
python src/main_new.py --compare -i data/input/流水.xlsx

# 时点查询
python src/services/query_cli.py -f data/input/流水.xlsx -r 100 -a BALANCE_METHOD
```

#### GUI版本
```bash
cd tauri-app/
npm run tauri dev
```

## 📖 完整文档

- **📋 项目架构文档**: [PROJECT_ARCHITECTURE.md](PROJECT_ARCHITECTURE.md) ⭐
- **🚀 快速开始指南**: [QUICK_START.md](QUICK_START.md)
- **🧮 差额计算法详解**: [docs/balance_method_logic.md](docs/balance_method_logic.md)
- **🖥️ GUI应用指南**: [tauri-app/🚀启动GUI界面.md](tauri-app/🚀启动GUI界面.md)

## 🏗️ 项目结构

```
审计系统/
├── src/                          # Python源代码核心
│   ├── main_new.py              # 主程序入口 ⭐
│   ├── core/                    # 核心架构层
│   ├── services/                # 服务业务层
│   └── utils/                   # 工具模块层
├── tauri-app/                   # GUI桌面应用
│   ├── src-tauri/              # Rust后端
│   └── src/                    # React前端
├── tests/                       # 测试代码
├── data/                        # 数据文件
│   ├── input/                  # 输入数据
│   └── output/                 # 输出结果
└── docs/                        # 项目文档
```

## 🧪 运行测试

```bash
# 运行所有测试
python -m pytest tests/ -v

# 运行双算法对比测试
python -m pytest tests/test_dual_algorithm.py -v

# 运行时点查询测试
python -m pytest tests/test_time_point_query.py -v
```

## 📊 算法对比

| 特性 | FIFO算法 | 差额计算法 |
|------|----------|------------|
| **逻辑** | 先进先出队列 | 余额优先扣除 |
| **个人支出** | 按队列顺序 | 个人余额优先 |
| **公司支出** | 按队列顺序 | 公司余额优先 |
| **挪用计算** | 队列追溯 | 直接计算 |
| **性能** | 复杂 O(n) | 简单 O(1) |
| **精确度** | 高 | 中 |

## 🔧 输入数据格式

Excel文件必需包含以下列：

| 列名 | 数据类型 | 必填 | 说明 |
|------|----------|------|------|
| 交易日期 | 日期 | ✅ | 交易发生日期 |
| 交易时间 | 时间字符串 | ✅ | 具体交易时间 |
| 交易收入金额 | 数值 | ✅ | 资金流入金额 |
| 交易支出金额 | 数值 | ✅ | 资金流出金额 |
| 余额 | 数值 | ✅ | 交易后账户余额 |
| 资金属性 | 文本 | ✅ | 资金归属和性质 |

## 📈 输出结果

- **主分析结果**: `FIFO/BALANCE_METHOD_资金追踪结果.xlsx`
- **场外资金池记录**: `场外资金池记录_[算法].xlsx`
- **详细日志**: `logs/` 目录

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- **项目主页**: [GitHub Repository]
- **问题反馈**: [Issues]
- **更新日志**: [CHANGELOG.md]

---

**📅 最后更新**: 2025年1月  
**📦 当前版本**: v3.0.0  
**🏷️ 项目状态**: 活跃维护

> **💡 提示**: 首次使用建议阅读 [PROJECT_ARCHITECTURE.md](PROJECT_ARCHITECTURE.md) 了解完整架构。
