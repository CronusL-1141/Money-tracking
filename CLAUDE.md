# CLAUDE.md

这是涉案资金追踪分析系统的Claude Code工作指南。

## 项目概述

**涉案资金追踪分析系统** - 司法审计工具，用于检测公款挪用、职务侵占等经济犯罪行为。

### 当前状态（2025年8月29日）
- **当前版本**: v3.3.4 - 项目标准化完成，版本信息统一
- **架构状态**: Rust后端引擎稳定，Tauri桌面应用完善
- **开发完成度**: 文件状态管理系统完成，算法验证通过
- **技术栈**: Python CLI + Tauri (React前端 + Rust壳)
- **目标**: 完全迁移到Rust后端 + Tauri

### 分支管理
- **main分支**: Python生产版本，不要修改
- **rust-backend分支**: 当前工作分支，所有开发在此进行
- **工作目录**: `C:\Users\TUF\Desktop\资金追踪`

## 常用命令

### 开发环境
```bash
# 启动GUI开发服务器
cd tauri-app && npm run tauri:dev

# Rust后端检查
cd rust-backend && cargo check --lib

# 运行Python版本（已移除，现使用Rust后端）
# python src/main.py -a FIFO -i 流水.xlsx
```

### 支持的算法
- **FIFO**: 先进先出算法
- **BALANCE_METHOD**: 差额计算法
- 测试文件: `流水.xlsx`

## 技术架构

### 当前架构（已实现）
```
GUI: Tauri → Rust库直接调用 → Excel输出
```

### 旧架构（已废弃）
```
GUI: Tauri → Shell.Command → Python CLI → Excel输出
```

### 核心模块（Rust后端）
```
rust-backend/src/
├── data_models/           # 数据结构
├── utils/                 # 工具层（Excel处理、数据验证）
├── algorithms/            # 算法层（FIFO、BalanceMethod）
├── services/              # 服务层（GUI交互）
└── bin/                   # CLI应用层（暂时禁用）
```

### GUI组件（Tauri）
```
tauri-app/src/
├── pages/                 # 页面（审计分析、时点查询、设置）
├── components/            # 组件（历史记录、时间选择器）
├── contexts/              # 状态管理（全局状态、跨页面同步）
├── utils/                 # 工具（文件状态管理、防重复机制）
└── services/              # 服务（系统初始化、Rust后端调用）
```

## 开发重点

### 已完成 ✅
- 算法层：FIFO和BalanceMethod算法100%正确
- 工具层：Excel处理、数据验证修复完成
- GUI优化：跨页面文件同步、防重复日志机制
- 测试验证：9,799条真实数据验证通过
- **文件状态管理系统**：智能检测、自动同步、实时监控

### 进行中 🔄
- 服务层完善：AuditService协调层优化
- 性能测试：大数据量处理基准测试

### 下一步 ⏳
- Rust后端完全集成到Tauri
- 移除Python依赖，实现纯Rust方案
- CLI应用层重新启用

## 技术约定

### 数据处理原则
1. **数据不变性**: 源Excel文件只读，修复在内存进行
2. **流水线处理**: 读取 → 验证修复 → 算法分析 → 输出
3. **清洁数据**: 业务算法只处理经过验证的数据

### GUI设计模式
1. **全局状态**: 使用AppStateContext管理跨页面状态
2. **防重复**: FileDropManager全局单例防重复文件处理
3. **用户体验**: 文件选择跨页面同步，避免重复操作

### 开发工具
- **Shell**: PowerShell 7
- **语言**: 中文回复和注释
- **测试数据**: 流水.xlsx

## 重要文件路径

### 核心文件
- Rust库入口: `rust-backend/src/lib.rs`
- Tauri配置: `tauri-app/src-tauri/tauri.conf.json`
- 主数据文件: `流水.xlsx`

### 关键新增文件
- 全局防重复: `tauri-app/src/utils/fileDropManager.ts`
- 历史记录管理: `tauri-app/src/utils/analysisHistoryManager.ts`
- 状态管理: `tauri-app/src/contexts/AppStateContext.tsx`
- 系统服务: `tauri-app/src/services/systemService.ts`
- 分析历史面板: `tauri-app/src/components/AnalysisHistoryPanel.tsx`

## 开发注意事项

1. **响应语言**: 使用中文
2. **任务跟踪**: 使用TodoWrite工具追踪进度
3. **文件创建**: 优先编辑现有文件，避免创建不必要文档
4. **数据安全**: 源数据绝对不修改，处理在内存副本进行
5. **功能范围**: 专注单一算法分析，不实现算法对比功能
6. **工作分支**: 始终在rust-backend分支工作

---

## 文件状态管理系统（v3.3.3新增）

### 系统特性
- **智能检测**：自动识别文件是否被外部删除或恢复
- **自动同步**：应用启动时自动检查所有历史记录的文件状态
- **实时监控**：查看历史记录时动态检测文件存在性
- **用户交互**：手动刷新按钮，支持主动文件状态更新

### 核心功能模块

#### AnalysisHistoryManager
```typescript
// 文件状态检测和更新
updateRecordFileStatus(record)       // 检查单个记录文件状态
syncAllRecordsFileStatus()          // 批量同步所有记录状态  
getHistoryWithRealTimeStatus()      // 获取实时状态的历史记录
```

#### SystemService  
```typescript
// 系统初始化服务
initialize()                        // 应用启动时环境检查+文件状态同步
```

#### AnalysisHistoryPanel
- **刷新按钮**：手动触发文件状态检测，带旋转动画反馈
- **视觉反馈**：删除的文件显示横线样式，透明度降低
- **智能按钮**：已删除文件的相关操作按钮自动禁用

### 使用场景
1. **应用启动**：自动检测所有历史文件状态，确保数据一致性
2. **页面加载**：每次查看历史记录都实时检测文件状态
3. **手动刷新**：用户可主动点击刷新按钮更新文件状态
4. **外部操作**：系统能检测到文件被外部程序删除并相应更新

---

**记住**: 这是一个从Python向Rust迁移的项目，当前处于文件状态管理系统完成阶段，下一步是Rust后端完全集成。