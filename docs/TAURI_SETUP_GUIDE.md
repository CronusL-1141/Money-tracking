# Tauri开发环境安装指南

## 📋 环境要求
- Windows 11 系统
- Node.js 18+ 
- Rust 1.70+
- Tauri CLI
- VS Code (推荐)

## 🔧 安装步骤

### 1. 安装Node.js
1. 访问 [Node.js官网](https://nodejs.org/)
2. 下载并安装 **LTS版本** (推荐18.x+)
3. 验证安装：
   ```powershell
   node --version
   npm --version
   ```

### 2. 安装Rust
1. 访问 [Rust官网](https://rustup.rs/)
2. 下载并运行 `rustup-init.exe`
3. 选择默认安装选项
4. 重启命令行工具
5. 验证安装：
   ```powershell
   rustc --version
   cargo --version
   ```

### 3. 安装Tauri CLI
```powershell
# 使用npm安装
npm install -g @tauri-apps/cli

# 或使用cargo安装
cargo install tauri-cli
```

### 4. 安装Windows构建工具
```powershell
# 安装Microsoft C++ Build Tools
# 通过Visual Studio Installer安装"C++ build tools"工作负载
```

### 5. 验证完整环境
```powershell
node --version    # 应显示 v18.x.x+
npm --version     # 应显示 9.x.x+
rustc --version   # 应显示 rustc 1.70+
cargo --version   # 应显示 cargo 1.70+
tauri --version   # 应显示 tauri-cli版本
```

## 🚀 快速开始

安装完成后，可以运行以下命令初始化项目：

```powershell
cd C:\Users\cronu\OneDrive\Desktop\法巴农银工作资料\审计
npm run tauri:init
```

## 📖 参考资源
- [Tauri官方文档](https://tauri.app/v1/guides/)
- [Rust学习资源](https://doc.rust-lang.org/book/)
- [React + TypeScript指南](https://react-typescript-cheatsheet.netlify.app/)

## ⚠️ 常见问题
- 如果遇到权限问题，以管理员身份运行命令行
- Windows Defender可能会误报，需要添加白名单
- 确保PATH环境变量正确配置