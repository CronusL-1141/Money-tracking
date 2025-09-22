# FLUX Financial Flow Analysis System

<div align="center">

**🌍 Language / 语言**: [中文](./README.md) | **English**

</div>

<div align="center">

![Version](https://img.shields.io/badge/version-v3.3.4-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![Language](https://img.shields.io/badge/language-Rust%20%2B%20TypeScript-orange)
![License](https://img.shields.io/badge/license-MIT-green)

**Professional Judicial Audit Tool - Detecting Misappropriation of Public Funds and Economic Crimes**

</div>

## 🚀 Quick Start

### 💾 Direct Download & Installation
Want to use it immediately? Download the installer directly:

**⬇️ [Download FLUX Financial Flow Analysis System_3.3.4_x64-setup.exe](./FLUX资金追踪分析系统_3.3.4_x64-setup.exe)**

- 📦 **File Size**: ~25MB
- 🖥️ **System Requirements**: Windows 10/11 x64
- ⚡ **Installation Time**: 1-2 minutes
- 🔧 **No Configuration**: Ready to use after installation, no additional dependencies required

---

## 📋 Project Overview

**FLUX Financial Flow Analysis System** is a desktop application specifically designed for judicial auditing. It can precisely analyze bank transaction data and identify patterns of fund misappropriation, embezzlement, and other economic crimes.

### 🎯 Core Features
- **📊 Intelligent Flow Analysis**: Supports both FIFO (First In First Out) and Balance Method algorithms
- **🔍 Precise Time-Point Queries**: Query financial status at any specific point in time
- **📈 Investment Product Tracking**: Track fund flows in investment pools and products
- **📋 History Management**: Smart analysis history management with file status detection
- **🌍 Multi-language Support**: Chinese/English interface switching
- **🌙 Theme Switching**: Support for light/dark theme modes

### 💼 Application Scenarios
- 🏛️ **Judicial Institutions**: Financial flow analysis for economic crime cases
- 🏢 **Audit Departments**: Corporate fund misappropriation detection
- 🏦 **Financial Institutions**: Internal risk control and compliance review
- 📊 **Accounting Firms**: Financial fraud investigation

---

## 🏗️ 系统架构

### 技术架构图
```
┌─────────────────────────────────────────────────────────────┐
│                    FLUX资金追踪分析系统                       │
│                   (纯Rust + Tauri架构)                      │
└─────────────────────────────────────────────────────────────┘

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   用户界面层     │    │   服务调用层     │    │   算法处理层     │
│   (Frontend)    │────│   (Bridge)      │────│   (Backend)     │
│                 │    │                 │    │                 │
│  React + TS     │    │  Tauri + Rust   │    │  Pure Rust      │
│  现代化GUI      │    │  直接调用       │    │  核心算法引擎    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  页面和组件      │    │  命令和接口      │    │  数据处理模块    │
│  • 首页         │    │  • 审计分析      │    │  • Excel处理    │
│  • 审计页面      │    │  • 时点查询      │    │  • 数据验证     │
│  • 时点查询      │    │  • 历史管理      │    │  • 算法计算     │
│  • 设置页面      │    │  • 文件操作      │    │  • 结果输出     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 🔄 数据处理流程
```
Excel文件输入 → 数据验证修复 → 算法分析 → 结果输出 → 历史记录保存
     ↓              ↓              ↓           ↓            ↓
  📄流水.xlsx   🔍完整性检查   🧮FIFO/差额法  📊Excel报告  💾本地存储
```

---

## 📱 功能页面详览

### 🏠 首页 (HomePage)
**功能**: 系统导航和项目概览
- 📊 **系统状态显示**: 显示当前系统版本和状态
- 🚀 **快速访问**: 提供到各功能页面的快速入口
- 📈 **使用统计**: 展示分析历史统计信息
- 🎨 **界面预览**: 系统主要功能的可视化展示

*[截图占位符 - 首页界面]*

### 📊 审计分析页面 (AuditPage)
**功能**: 核心分析功能界面
- 📁 **文件选择**: 支持拖拽和浏览器选择Excel文件
- ⚙️ **算法配置**: 选择FIFO或差额计算法
- ⏳ **实时进度**: 显示分析进度和状态信息
- 📋 **历史面板**: 智能历史记录管理，支持文件状态检测
- 🔄 **批处理**: 支持多文件批量分析

**核心特性**:
- ✅ 智能文件验证 (支持.xlsx格式检查)
- ✅ 实时进度反馈 (进度条 + 状态消息)
- ✅ 错误处理机制 (友好的错误提示)
- ✅ 防重复处理 (避免同一文件重复分析)

*[截图占位符 - 审计分析页面]*

### 🔍 时点查询页面 (TimePointQueryPage)
**功能**: 精确时点状态查询
- 🎯 **行号输入**: 输入特定行号查询该时点状态
- ⌨️ **回车支持**: 支持回车键快速查询
- 📊 **状态展示**: 显示该时点的完整财务状态
- 📈 **图表可视化**: 资金流向的图表展示
- 🔄 **算法切换**: 支持不同算法下的时点对比

**查询结果包含**:
- 💰 个人余额 / 公司余额
- 📈 累计挪用 / 垫付金额
- 🏦 投资产品状态
- 📊 资金占比分析

*[截图占位符 - 时点查询页面]*

### ⚙️ 设置页面 (SettingsPage)
**功能**: 系统个性化配置
- 🌍 **语言设置**: 中文/English界面切换
- 🌙 **主题模式**: 明亮主题 ↔ 暗黑主题
- 📁 **默认路径**: 设置文件输入/输出默认目录
- 🔧 **高级选项**: 算法参数调整和性能优化
- 📊 **数据管理**: 历史记录清理和导出功能

**主题切换效果**:
- 🌞 **明亮主题**: 经典白色背景，适合办公环境
- 🌚 **暗黑主题**: 护眼深色背景，适合长时间使用

*[截图占位符 - 设置页面 (明亮主题)]*

*[截图占位符 - 设置页面 (暗黑主题)]*

### 📋 历史记录管理 (AnalysisHistoryPanel)
**功能**: 智能分析历史管理
- 📝 **记录展示**: 显示所有历史分析记录
- 🔍 **智能检测**: 自动检测文件是否被外部删除
- 🔄 **状态同步**: 实时更新文件存在状态
- 📂 **快速操作**: 打开、另存为、删除等操作
- 🗑️ **批量清理**: 支持按时间范围批量删除记录

**文件状态管理**:
- ✅ **存在文件**: 正常显示，支持所有操作
- ❌ **已删除文件**: 横线显示，相关操作自动禁用
- 🔄 **自动刷新**: 支持手动刷新检测文件状态

*[截图占位符 - 历史记录面板]*

---

## 🌍 国际化支持

### 多语言切换
系统提供完整的中英文界面，用户可在设置页面随时切换：

| 功能 | 中文 | English |
|------|------|---------|
| 界面语言 | 简体中文 | English |
| 错误提示 | 中文消息 | English Messages |
| 报告输出 | 中文表头 | English Headers |
| 帮助文档 | 中文说明 | English Documentation |

*[截图占位符 - 中文界面]*

*[截图占位符 - 英文界面]*

---

## 🎨 主题系统

### 视觉主题对比

| 元素 | 明亮主题 | 暗黑主题 |
|------|----------|----------|
| 背景色 | 纯白 #FFFFFF | 深灰 #1a1a1a |
| 主色调 | 蓝色 #1976d2 | 青色 #00bcd4 |
| 文字色 | 深黑 #212121 | 浅白 #ffffff |
| 卡片色 | 浅灰 #f5f5f5 | 深灰 #2d2d2d |
| 按钮色 | 蓝色系 | 青色系 |

**切换动效**: 平滑过渡动画，提供流畅的视觉体验

*[截图占位符 - 主题对比图]*

---

## 📊 核心算法介绍

### 🔄 FIFO算法 (先进先出)
**原理**: 按照资金进入的时间顺序进行追踪分析
- ⚡ **适用场景**: 标准的资金流向分析
- 📈 **优势**: 逻辑清晰，易于理解和审核
- 🎯 **精度**: 适用于大多数常规审计案件

### ⚖️ 差额计算法 (Balance Method)
**原理**: 基于余额差额变化进行资金归属分析
- 🏆 **适用场景**: 复杂的资金混合情况
- 🔍 **优势**: 能处理更复杂的资金流向
- 📊 **精度**: 适用于高难度审计案件

---

## 🔧 开发和部署

### 开发环境
- **前端**: React 18 + TypeScript + Vite
- **后端**: Rust + Tauri
- **UI库**: Material-UI (MUI)
- **构建工具**: Cargo + npm

### 系统要求
- **操作系统**: Windows 10/11 (x64)
- **内存**: 最低4GB，推荐8GB+
- **存储**: 100MB可用空间
- **依赖**: 无需额外运行时

---

## 📞 支持与反馈

### 🐛 问题报告
如遇到问题，请提供以下信息：
- 系统版本和操作系统
- 详细的操作步骤
- 错误截图或日志文件
- 测试数据文件（如可提供）

### 📬 联系方式
- **GitHub Issues**: 技术问题和功能建议
- **电子邮件**: 企业用户支持
- **用户手册**: 详细操作指南

---

## 📄 许可证

本项目采用 MIT 许可证 - 详情请查看 [LICENSE](LICENSE) 文件。

---

<div align="center">

**🌟 如果这个项目对您有帮助，请给个Star支持！🌟**

Made with ❤️ by FLUX Team | Powered by Rust + Tauri

</div>