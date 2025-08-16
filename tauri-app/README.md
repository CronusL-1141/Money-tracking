# FIFO资金追踪审计系统 - 桌面版

基于 Tauri + React + TypeScript 构建的专业资金流向追踪分析桌面应用程序。

## 🚀 技术栈

- **前端**: React 18 + TypeScript + Material-UI
- **后端**: Rust (Tauri)
- **Python集成**: 复用现有的FIFO和差额计算法核心算法
- **构建工具**: Vite + Tauri CLI
- **国际化**: react-i18next (中文/英文)
- **主题**: Material-UI 主题系统 (浅色/深色/自动)

## 📋 主要功能

### 1. 双算法支持
- **FIFO算法**: 先进先出原则的资金流向追踪
- **差额计算法**: 基于余额优先的挪用检测算法

### 2. 核心模块
- 📊 **审计分析**: 完整的资金流向分析和报告生成
- 🔍 **时点查询**: 查询任意交易行的系统状态
- ⚙️ **设置管理**: 主题、语言、环境配置
- 🏠 **仪表盘**: 系统概览和快速操作

### 3. 技术特性
- 🖥️ **原生桌面应用**: 基于Tauri的跨平台支持
- 🔒 **离线运行**: 完全本地化处理，无需网络
- 🌍 **国际化**: 中英文界面切换
- 🎨 **主题系统**: 浅色/深色主题自动切换
- 📱 **响应式设计**: 适配不同屏幕尺寸
- ⚡ **高性能**: 支持处理50万行数据

## 🛠️ 开发环境设置

### 前提条件

1. **Node.js** (18.x+)
2. **Rust** (1.70+)
3. **Python** (3.8+) - 用于核心算法
4. **Tauri CLI**

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装Tauri CLI (如果没有)
npm install -g @tauri-apps/cli
```

### 开发命令

```bash
# 开发模式
npm run tauri:dev

# 构建应用
npm run tauri:build

# 仅前端开发
npm run dev

# 构建前端
npm run build
```

## 📁 项目结构

```
tauri-app/
├── src/                    # React 前端源码
│   ├── components/         # React 组件
│   │   ├── layout/        # 布局组件
│   │   └── common/        # 通用组件
│   ├── pages/             # 页面组件
│   ├── contexts/          # React Context
│   ├── services/          # 服务层
│   ├── types/             # TypeScript 类型
│   ├── locales/           # 国际化资源
│   ├── styles/            # 样式文件
│   └── utils/             # 工具函数
├── src-tauri/             # Rust 后端源码
│   ├── src/               # Rust 源码
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
├── public/                # 静态资源
├── package.json           # Node.js 依赖配置
├── tsconfig.json          # TypeScript 配置
└── vite.config.ts         # Vite 构建配置
```

## 🔧 配置说明

### Tauri 配置 (src-tauri/tauri.conf.json)

- 窗口设置: 1200x800 初始尺寸，最小 800x600
- 文件系统权限: 允许访问数据目录和用户文档
- Python执行权限: 允许调用系统Python环境

### 前端配置

- **路径别名**: `@/` 指向 `src/` 目录
- **主题系统**: 支持浅色/深色/自动主题
- **国际化**: 支持中文/英文动态切换
- **状态管理**: 使用React Context管理全局状态

## 🔌 Python 集成

应用通过Tauri命令调用现有Python脚本：

```rust
// 审计分析
invoke('run_audit', { 
  algorithm: 'FIFO', 
  input_file: 'data.xlsx' 
})

// 时点查询  
invoke('time_point_query', {
  file_path: 'data.xlsx',
  row_number: 100,
  algorithm: 'BALANCE_METHOD'
})
```

## 🌍 国际化支持

### 支持语言
- 🇨🇳 中文 (简体)
- 🇺🇸 英文

### 使用方式
```typescript
const { t } = useTranslation();
// 使用翻译
<Typography>{t('common.save')}</Typography>
```

## 🎨 主题系统

### 主题模式
- **浅色主题**: 适合日间使用
- **深色主题**: 适合夜间使用  
- **自动模式**: 跟随系统设置

### 自定义主题
```typescript
const { themeMode, setThemeMode } = useTheme();
setThemeMode('dark'); // 切换到深色主题
```

## 📦 构建和部署

### 开发构建
```bash
npm run tauri:dev
```

### 生产构建
```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 🔧 故障排除

### 常见问题

1. **Python环境问题**
   - 确保Python已安装并在PATH中
   - 检查Python版本兼容性 (3.8+)

2. **依赖安装失败**
   - 确保Node.js和Rust版本正确
   - 尝试清除缓存: `npm cache clean --force`

3. **构建失败**
   - 检查Tauri配置文件语法
   - 确保所有依赖已正确安装

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交Issue和Pull Request来改进项目。

## 📞 支持

如有问题，请通过以下方式联系：
- 提交GitHub Issue
- 查看文档: `docs/TAURI_SETUP_GUIDE.md`