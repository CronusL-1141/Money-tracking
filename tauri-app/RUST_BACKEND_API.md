# Rust后端API文档

## 概述

Phase 4 完成了完整的Rust后端命令接口开发，实现了与Python核心逻辑的桥接和GUI所需的所有功能。

## 核心功能模块

### 1. 审计分析模块

#### `get_algorithms`
获取可用的算法列表
- **返回**: `Vec<String>` - 算法列表 ["FIFO", "BALANCE_METHOD"]

#### `run_audit`
执行审计分析
- **参数**: `AuditConfig` - 审计配置
  - `algorithm: String` - 算法类型
  - `input_file: String` - 输入文件路径
  - `output_file: Option<String>` - 输出文件路径（可选）
- **返回**: `AuditResult` - 审计结果
  - `success: bool` - 是否成功
  - `message: String` - 结果消息
  - `data: Option<Value>` - 结果数据
  - `output_files: Vec<String>` - 输出文件列表

### 2. 时点查询模块

#### `time_point_query`
执行时点查询
- **参数**: `TimePointQuery` - 查询配置
  - `file_path: String` - 文件路径
  - `row_number: u32` - 行号
  - `algorithm: String` - 算法类型
- **返回**: `QueryResult` - 查询结果
  - `success: bool` - 是否成功
  - `data: Option<Value>` - 查询数据
  - `message: String` - 结果消息

#### `get_query_history`
获取查询历史记录
- **返回**: `Vec<QueryHistory>` - 历史记录列表

#### `clear_query_history`
清空查询历史记录
- **返回**: `()` - 无返回值

#### `delete_query_history_item`
删除指定的历史记录项
- **参数**: `id: String` - 记录ID
- **返回**: `bool` - 是否成功删除

### 3. 应用状态管理模块

#### `get_process_status`
获取当前进程状态
- **返回**: `ProcessStatus` - 进程状态
  - `running: bool` - 是否正在运行
  - `command: Option<String>` - 当前命令
  - `progress: Option<f32>` - 进度百分比
  - `message: Option<String>` - 状态消息

#### `get_app_config`
获取应用配置
- **返回**: `AppConfig` - 应用配置
  - `default_algorithm: String` - 默认算法
  - `auto_export: bool` - 自动导出设置
  - `max_history: usize` - 最大历史记录数
  - `language: String` - 界面语言
  - `theme: String` - 界面主题

#### `update_app_config`
更新应用配置
- **参数**: `new_config: AppConfig` - 新的配置
- **返回**: `()` - 无返回值

### 4. 文件操作模块

#### `get_file_info`
获取文件信息
- **参数**: `path: String` - 文件路径
- **返回**: `FileInfo` - 文件信息
  - `path: String` - 文件路径
  - `name: String` - 文件名
  - `size: u64` - 文件大小
  - `modified: DateTime<Utc>` - 修改时间
  - `exists: bool` - 文件是否存在

#### `validate_file_path`
验证文件路径有效性
- **参数**: `path: String` - 文件路径
- **返回**: `bool` - 路径是否有效

#### `export_query_result`
导出查询结果到文件
- **参数**:
  - `query_id: String` - 查询ID
  - `output_path: String` - 输出路径
- **返回**: `bool` - 是否成功导出

### 5. 系统环境模块

#### `check_python_env`
检查Python环境状态
- **返回**: `serde_json::Value` - Python环境信息
  - `python_available: bool` - Python是否可用
  - `python_version: String` - Python版本
  - `python_path: String` - Python路径
  - `project_root: String` - 项目根目录

## 核心特性

### 状态管理
- **线程安全**: 使用 `tokio::sync::Mutex` 保证并发安全
- **实时状态**: 支持实时查询进程状态和进度
- **历史记录**: 自动管理查询历史，支持限制最大记录数

### 错误处理
- **统一错误格式**: 所有命令返回统一的错误信息
- **详细日志**: 使用 `log` 库记录详细的操作日志
- **优雅失败**: 确保错误不会导致应用崩溃

### Python桥接
- **自动发现**: 自动查找Python可执行文件
- **项目路径解析**: 智能定位Python脚本位置
- **参数传递**: 正确传递参数到Python脚本
- **输出解析**: 解析Python脚本的输出和错误信息

### 性能优化
- **异步处理**: 所有命令都是异步的，避免阻塞UI
- **进度跟踪**: 长时间运行的操作支持进度更新
- **资源清理**: 及时清理临时资源和状态

## 前端集成

### TypeScript类型定义
在 `src/types/rust-commands.ts` 中定义了完整的TypeScript接口，包括：
- 所有数据结构的类型定义
- `RustCommands` 类提供所有命令的调用方法
- 便捷的导出函数供直接使用

### 服务层集成
在 `src/services/pythonService.ts` 中更新了 `PythonService`：
- 完全使用新的Rust命令接口
- 保持向后兼容的API
- 增加了新功能的服务方法
- 统一的错误处理和日志记录

## 使用示例

### 基本审计分析
```typescript
import { RustCommands } from '../types/rust-commands';

// 运行FIFO算法
const result = await RustCommands.runAudit({
  algorithm: 'FIFO',
  input_file: '/path/to/input.xlsx',
  output_file: '/path/to/output.xlsx'
});

console.log('审计结果:', result);
```

### 时点查询
```typescript
// 执行时点查询
const queryResult = await RustCommands.timePointQuery({
  file_path: '/path/to/data.xlsx',
  row_number: 100,
  algorithm: 'FIFO'
});

// 获取查询历史
const history = await RustCommands.getQueryHistory();
```

### 状态监控
```typescript
// 检查进程状态
const status = await RustCommands.getProcessStatus();
if (status.running) {
  console.log(`正在执行: ${status.command}, 进度: ${status.progress}%`);
}
```

### 配置管理
```typescript
// 获取当前配置
const config = await RustCommands.getAppConfig();

// 更新配置
await RustCommands.updateAppConfig({
  ...config,
  default_algorithm: 'BALANCE_METHOD',
  theme: 'dark'
});
```

## 技术实现要点

### 依赖管理
- **Tauri 1.x**: 主框架
- **tokio**: 异步运行时
- **serde**: 序列化/反序列化
- **chrono**: 时间处理
- **log/env_logger**: 日志记录
- **which**: 查找可执行文件

### 架构设计
- **命令模式**: 每个功能都是独立的Tauri命令
- **状态管理**: 全局应用状态，支持并发访问
- **工厂模式**: Python脚本路径和参数的智能构建
- **观察者模式**: 进程状态的实时更新机制

### 安全考虑
- **路径验证**: 严格验证所有文件路径
- **参数过滤**: 过滤和转义所有Python脚本参数
- **权限控制**: 限制文件系统访问范围
- **错误隔离**: 防止Python错误影响Rust进程

## 后续扩展

Phase 4 为后续的前端开发(Phase 5)奠定了坚实的基础：
- 完整的API接口覆盖了所有GUI需求
- 类型安全的TypeScript集成
- 统一的错误处理和状态管理
- 可扩展的架构设计

下一步可以开始React前端界面的开发工作。