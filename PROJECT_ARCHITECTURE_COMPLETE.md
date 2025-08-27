# 涉案资金追踪分析系统 - 架构文档

> **版本**: v3.3.2  
> **更新时间**: 2025年8月27日  
> **状态**: GUI优化完成，Rust后端集成进行中

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

## 📁 目录结构

### 项目根目录
```
资金追踪/
├── src/                    # Python生产版本
├── rust-backend/           # Rust后端开发
├── tauri-app/             # GUI应用
├── tests/                 # Python测试
├── independent_tests/     # 独立验证测试
└── 流水.xlsx              # 主要测试数据
```

### Rust后端结构
```
rust-backend/src/
├── data_models/           # 数据模型
│   ├── transaction.rs     # 交易记录
│   ├── audit_summary.rs   # 审计摘要
│   └── fund_pool.rs       # 资金池管理
├── utils/                 # 工具层
│   ├── excel_processor.rs # Excel读写
│   ├── unified_validator.rs # 数据验证修复
│   └── time_processor.rs  # 时间处理
├── algorithms/            # 算法层
│   ├── fifo_tracker.rs    # FIFO算法
│   └── balance_method_tracker.rs # 差额计算法
├── services/              # 服务层
│   └── audit_service.rs   # 审计服务协调
└── lib.rs                 # 库入口
```

### GUI应用结构
```
tauri-app/src/
├── pages/                 # 页面组件
│   ├── AuditPage.tsx      # 资金分析页面
│   ├── TimePointQueryPage.tsx # 时点查询页面
│   └── SettingsPage.tsx   # 设置页面
├── components/            # 复用组件
│   ├── AnalysisHistoryPanel.tsx # 历史记录面板
│   └── HybridDateTimePicker.tsx # 时间选择器
├── contexts/              # 状态管理
│   └── AppStateContext.tsx # 全局状态
├── utils/                 # 工具函数
│   ├── fileDropManager.ts # 防重复机制
│   └── analysisHistoryManager.ts # 历史记录
└── types/                 # 类型定义
```

## 🔧 技术栈

### 后端技术
- **Rust**: 核心业务逻辑
- **calamine**: Excel读取
- **rust_xlsxwriter**: Excel写入
- **rust_decimal**: 精确数值计算
- **chrono**: 时间处理

### 前端技术
- **Tauri**: 桌面应用框架
- **React 18**: UI框架
- **TypeScript**: 类型安全
- **Material-UI**: 组件库
- **React i18next**: 国际化

### Python生产版本
- **pandas**: 数据处理
- **openpyxl**: Excel操作
- **argparse**: 命令行接口

## 🚀 核心功能

### 算法支持
1. **FIFO（先进先出）**: 按时间顺序追踪资金流向
2. **差额计算法**: 基于余额变化计算挪用金额

### 主要特性
- ✅ **数据验证修复**: 自动修复流水完整性问题
- ✅ **场外资金池**: 投资产品盈亏计算
- ✅ **时点查询**: 任意时点的资金状态查询
- ✅ **历史记录**: 分析历史管理和清理
- ✅ **跨页面同步**: 全局文件状态管理

### 用户体验优化
- ✅ **文件拖拽**: 支持Excel文件拖拽选择
- ✅ **防重复机制**: 全局单例防重复处理
- ✅ **主题切换**: 浅色/深色主题支持
- ✅ **响应式设计**: 适配不同屏幕尺寸

## 📊 数据流程

### 核心数据管道
```
Excel输入 → 数据验证修复 → 算法分析 → 结果输出
    │           │              │          │
    │           │              │          └─ Excel格式化输出
    │           │              └─ 投资池盈亏计算
    │           └─ 流水完整性检查和自动修复
    └─ calamine读取，数据不可变原则
```

### 状态管理流程
```
用户操作 → AppStateContext → 页面状态更新
    │                            │
    │                            └─ 跨页面文件同步
    └─ FileDropManager全局防重复
```

## 🧪 测试验证

### 算法验证状态
- ✅ **FIFO算法**: 9,799条真实数据100%正确
- ✅ **差额计算法**: 多案例验证通过
- ✅ **投资池逻辑**: 复杂投资场景修复完成
- ✅ **端到端验证**: Excel输出完全正确

### 性能指标
- **数据处理速度**: 50,000+ 条/秒
- **内存使用**: 优化的内存管理
- **响应时间**: GUI操作即时响应

## 🔄 开发状态

### 已完成 ✅
- 工具层：Excel处理、数据验证修复
- 算法层：FIFO和差额计算法
- GUI层：用户体验优化、跨页面同步
- 测试验证：真实数据验证通过

### 进行中 🔄
- 服务层：AuditService协调层完善
- 集成测试：Rust后端与Tauri集成

### 计划中 ⏳
- Python依赖移除：完全迁移到Rust
- 性能优化：大数据量处理基准测试
- CLI重启：基于Rust的命令行工具

## 💻 开发环境

### 必需工具
- **Rust**: 1.70+
- **Node.js**: 18+
- **Python**: 3.11+ (生产版本)
- **PowerShell**: 7.0+

### 常用命令
```bash
# 启动开发环境
cd tauri-app && npm run tauri:dev

# Rust后端检查
cd rust-backend && cargo check --lib

# Python生产版本测试
python src/main.py -a FIFO -i 流水.xlsx
```

---

**项目状态**: GUI优化完成，正在进行Rust后端最终集成阶段。