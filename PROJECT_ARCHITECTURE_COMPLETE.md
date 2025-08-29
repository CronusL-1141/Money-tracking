# 涉案资金追踪分析系统 - 完整架构文档

> **版本**: v3.3.3  
> **更新时间**: 2025年8月27日  
> **状态**: 文件状态管理系统完成，GUI功能完善，Rust后端集成进行中

## 🏗️ 系统架构概览

### 当前架构（生产环境）
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Tauri GUI     │    │  Shell Command  │    │  Python Backend │
│  React + TS     │───▶│   Process       │───▶│   FIFO/Balance  │
│  用户交互界面    │    │   调用中介       │    │   核心算法      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                         ┌─────────────────┐
                         │  Excel 输出     │
                         │  分析结果       │
                         └─────────────────┘
```

### 目标架构（开发中）
```
┌─────────────────┐    ┌─────────────────┐
│   Tauri GUI     │    │  Rust Backend   │
│  React + TS     │───▶│   直接调用       │
│  用户交互界面    │    │   FIFO/Balance  │
└─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │  Excel 输出     │
                       │  分析结果       │
                       └─────────────────┘
```

## 📁 项目目录结构

### 完整项目结构
```
资金追踪/
├── 📁 src/                          # [已删除] Python版本已废弃
│
├── 🦀 rust-backend/                 # Rust后端 (v3.3.3开发中)
│   ├── src/
│   │   ├── lib.rs                   # 库入口
│   │   ├── data_models/             # 数据结构
│   │   │   ├── mod.rs
│   │   │   ├── transaction.rs
│   │   │   ├── config.rs
│   │   │   ├── audit_summary.rs
│   │   │   └── offsite_pool_record.rs
│   │   ├── utils/                   # 工具层
│   │   │   ├── mod.rs
│   │   │   ├── excel_processor.rs   # Excel处理
│   │   │   ├── unified_validator.rs # 数据验证
│   │   │   └── time_processor.rs    # 时间处理
│   │   ├── algorithms/              # 算法层
│   │   │   ├── mod.rs
│   │   │   ├── fifo_tracker.rs      # FIFO算法
│   │   │   ├── balance_method_tracker.rs # 差额计算法
│   │   │   └── shared/              # 共享组件
│   │   ├── services/                # 服务层
│   │   │   └── mod.rs
│   │   └── bin/                     # CLI应用 (暂时禁用)
│   └── Cargo.toml
│
├── 📱 tauri-app/                    # GUI应用 (v3.3.3)
│   ├── src/
│   │   ├── App.tsx                  # 主应用组件
│   │   ├── main.tsx                 # React入口
│   │   ├── pages/                   # 页面组件
│   │   │   ├── HomePage.tsx         # 首页
│   │   │   ├── AuditPage.tsx        # 审计分析页面
│   │   │   ├── TimePointQueryPage.tsx # 时点查询页面
│   │   │   └── SettingsPage.tsx     # 设置页面
│   │   ├── components/              # 可复用组件
│   │   │   ├── layout/
│   │   │   ├── ErrorBoundary.tsx
│   │   │   └── AnalysisHistoryPanel.tsx # 分析历史面板
│   │   ├── contexts/                # React上下文
│   │   │   └── AppStateContext.tsx  # 全局状态管理
│   │   ├── services/                # 前端服务
│   │   │   └── systemService.ts     # 系统服务
│   │   ├── utils/                   # 工具函数
│   │   │   ├── analysisHistoryManager.ts # 历史记录管理
│   │   │   └── fileDropManager.ts   # 文件拖拽管理
│   │   └── types/                   # 类型定义
│   │       └── analysisHistory.ts   # 历史记录类型
│   │
│   └── src-tauri/                   # Rust桌面应用壳
│       ├── src/main.rs              # Tauri接口
│       ├── Cargo.toml
│       └── tauri.conf.json
│
├── 📄 tests/                        # Python测试
├── 🔬 independent_tests/            # 独立验证测试
├── 📊 流水.xlsx                     # 主要测试数据 (9,799条记录)
├── 📄 CLAUDE.md                     # 项目工作指南
└── 📄 PROJECT_ARCHITECTURE_COMPLETE.md # 本文档
```

## 🎯 核心功能模块

### 1. GUI层 (Tauri + React)

#### 主要页面
- **AuditPage.tsx**: 审计分析主页面，支持文件选择、算法配置、进度显示
- **TimePointQueryPage.tsx**: 时点查询页面，支持行号输入、回车键查询
- **SettingsPage.tsx**: 系统设置页面
- **HomePage.tsx**: 系统首页和导航

#### 核心组件
- **AnalysisHistoryPanel.tsx**: 分析历史记录面板
  - 历史记录展示和管理
  - 文件状态实时检测
  - 删除、另存为、打开功能
  - 手动刷新按钮
- **AppStateContext.tsx**: 全局状态管理
  - 跨页面文件状态同步
  - 全局配置管理
- **ErrorBoundary.tsx**: 错误边界处理

#### 工具模块
- **analysisHistoryManager.ts**: 历史记录管理器
  - 分析记录的CRUD操作
  - 文件状态检测和同步
  - 统计信息生成
- **fileDropManager.ts**: 文件拖拽管理器
  - 全局单例模式
  - 防重复文件处理
  - 跨页面文件状态同步

### 2. 服务层 (Services)

#### SystemService.ts
- **环境检查**: 系统可用性检测
- **文件状态同步**: 应用启动时自动检测文件状态
- **Rust后端调用**: 提供统一的后端接口

#### 历史记录管理
- **本地存储**: localStorage持久化
- **文件状态跟踪**: 实时检测文件存在性
- **批量操作**: 支持批量删除和清理

### 3. 算法层 (Rust Backend)

#### 核心算法
- **FIFO (先进先出)**: rust-backend/src/algorithms/fifo_tracker.rs
- **BALANCE_METHOD (差额计算法)**: rust-backend/src/algorithms/balance_method_tracker.rs
- **共享架构**: algorithms/shared/ 目录下的通用组件

#### 数据处理
- **Excel处理**: utils/excel_processor.rs (calamine + rust_xlsxwriter)
- **数据验证**: utils/unified_validator.rs (流水完整性验证)
- **时间处理**: utils/time_processor.rs

## 🔄 文件状态管理系统 (v3.3.3新增)

### 系统特性
- **智能检测**: 自动识别文件是否被外部删除或恢复
- **自动同步**: 应用启动时自动检查所有历史记录的文件状态
- **实时监控**: 查看历史记录时动态检测文件存在性
- **用户交互**: 手动刷新按钮，支持主动文件状态更新

### 核心架构
```
应用启动
    │
    ▼
SystemService.initialize()
    │
    ├─ 检查系统环境
    └─ 自动同步文件状态
        │
        ▼
AnalysisHistoryManager.syncAllRecordsFileStatus()
    │
    ├─ 批量检查所有历史记录
    ├─ 并发检测文件存在性 (Promise.all)
    ├─ 更新删除/恢复状态
    └─ 保存到本地存储
        │
        ▼
用户查看历史记录
    │
    ▼
AnalysisHistoryPanel
    │
    ├─ 实时检测文件状态 (getHistoryWithRealTimeStatus)
    ├─ 视觉反馈 (删除文件横线显示、透明度降低)
    ├─ 智能按钮禁用 (已删除文件操作按钮自动禁用)
    └─ 手动刷新功能 (带旋转动画反馈)
```

### 数据结构扩展
```typescript
interface OutputFile {
  name: string;
  path: string;
  size?: number;
  deleted?: boolean;      // 文件删除状态
  deleteError?: string;   // 删除错误信息
}

interface OffsitePoolFile {
  name: string;
  path: string;
  size?: number;
  deleted?: boolean;      // 文件删除状态
  deleteError?: string;   // 删除错误信息
}
```

### 核心方法
- **updateRecordFileStatus()**: 检查单个记录文件状态
- **syncAllRecordsFileStatus()**: 批量同步所有记录状态
- **getHistoryWithRealTimeStatus()**: 获取实时状态历史记录

## 🛠️ 技术栈

### 前端技术栈
- **React 18**: 用户界面框架
- **TypeScript**: 类型安全的JavaScript
- **Material-UI (MUI)**: UI组件库
- **Tauri**: 跨平台桌面应用框架
- **React Router**: 路由管理
- **React Context**: 状态管理

### 后端技术栈
- **Rust**: 系统编程语言
- **calamine**: Excel文件读取
- **rust_xlsxwriter**: Excel文件写入
- **serde**: 序列化/反序列化
- **tokio**: 异步运行时
- **chrono**: 时间处理

### 开发工具
- **Vite**: 前端构建工具
- **ESLint + Prettier**: 代码规范
- **Cargo**: Rust包管理器
- **PowerShell 7**: 主要开发Shell

## 🚀 当前开发状态

### 已完成 ✅
- **算法层**: FIFO和BalanceMethod算法100%正确，通过9,799条真实数据验证
- **工具层**: Excel处理、数据验证修复完成
- **GUI优化**: 跨页面文件同步、防重复日志机制
- **文件状态管理系统**: 智能检测、自动同步、实时监控
- **用户体验增强**: 时点查询回车键支持、历史记录管理、视觉反馈优化

### 进行中 🔄
- **服务层完善**: AuditService协调层优化
- **Rust后端集成**: 将GUI直接调用Rust库，移除Python依赖
- **性能测试**: 大数据量处理基准测试

### 下一步 ⏳
- **完全迁移到Rust**: 移除Python依赖，实现纯Rust方案
- **CLI应用层重启**: 重新启用命令行工具
- **性能优化**: 大数据处理性能调优
- **生产部署**: 最终版本打包和发布

## 🔧 开发环境配置

### 必需工具
- **Rust**: 1.70+
- **Node.js**: 18+
- **Python**: 3.11+ (生产版本兼容)
- **PowerShell**: 7.0+

### 常用命令
```bash
# 启动GUI开发环境
cd tauri-app && npm run tauri:dev

# Rust后端检查
cd rust-backend && cargo check --lib

# Python版本已废弃，现使用Rust后端
# python src/main.py -a FIFO -i 流水.xlsx

# 前端开发服务器
cd tauri-app && npm run dev
```

### 项目初始化
```bash
# 克隆项目
git clone <repository-url>
cd 资金追踪

# 安装前端依赖
cd tauri-app && npm install

# 检查Rust环境
cd rust-backend && cargo check

# 验证Python环境
python --version
python -m pip list
```

## 📊 数据处理流程

### 完整处理管道
```
Excel文件输入 (.xlsx)
    │
    ▼
[1] Excel读取层 (ExcelProcessor::read_transactions)
    │ • calamine读取
    │ • 数据类型转换
    │ • 初步验证
    ▼
[2] 数据验证和修复层 (UnifiedValidator)
    │ • 必需列检查
    │ • 流水完整性验证
    │ • 贪心算法修复
    │ • 时间序列整理
    ▼
[3] 算法处理层 (FIFO/BalanceMethod)
    │ • 资金流向分析
    │ • 挪用垫付计算
    │ • 投资产品处理
    │ • 行为模式识别
    ▼
[4] 结果输出层 (rust_xlsxwriter)
    │ • 分析结果格式化
    │ • Excel报告生成
    │ • 场外资金池记录
    │ • 统计摘要生成
    ▼
输出文件和历史记录保存
```

## 🔒 数据安全原则

### 数据不变性保证
1. **源文件保护**: 原始Excel文件在整个过程中保持只读，绝不修改
2. **内存副本处理**: 所有验证和修复都在内存副本上进行  
3. **数据流单向性**: 数据只能向下游传递，不能回流修改上游
4. **修复数据标准**: 只有经过完整性验证和修复的数据才能进入业务算法
5. **清洁数据保证**: 业务逻辑只处理经过验证修复的清洁数据

### 文件管理安全
- **状态跟踪**: 实时监控文件存在性，防止外部删除导致的数据不一致
- **错误处理**: 完善的异常捕获和用户反馈机制
- **备份机制**: 历史记录持久化存储，防止数据丢失

---

**项目状态**: v3.3.3文件状态管理系统完成，GUI功能完善，正在进行Rust后端最终集成阶段。

**开发重点**: 下一阶段将专注于移除Python依赖，实现完全的Rust后端直接调用，提升系统性能和部署便利性。