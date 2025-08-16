# 🔄 用户更新机制完整配置指南

## 📋 用户更新流程概览

### 🎯 **用户端更新体验**

#### **自动更新（推荐体验）**
```
用户启动应用
    ↓
后台自动检查更新（延迟3秒）
    ↓
发现新版本 → 弹出更新对话框
    ↓
显示版本信息和更新内容
    ↓
用户点击"立即更新" → 开始下载
    ↓
显示下载进度 → 安装完成
    ↓
提示重启 → 用户确认 → 自动重启
    ↓
✅ 更新完成，使用新版本
```

#### **手动更新**
```
用户打开设置页面
    ↓
点击"检查更新"按钮
    ↓
立即检查并显示结果
    ↓
按提示完成更新流程
```

### 🌐 **联网要求说明**

| 功能 | 是否需要网络 | 说明 |
|-----|-------------|------|
| ✅ **核心功能** | ❌ **不需要** | 数据处理、审计分析、Excel导入导出完全离线 |
| ✅ **日常使用** | ❌ **不需要** | 所有业务功能本地运行 |
| 🔄 **检查更新** | ✅ **需要** | 从服务器获取版本信息 |
| 📥 **下载更新** | ✅ **需要** | 下载新版本安装包 |
| ⚙️ **更新设置** | ❌ **不需要** | 可以关闭自动更新功能 |

**结论：软件可以完全离线使用，只有更新功能需要网络连接**

---

## 🏗️ **更新服务器配置**

### **方案1：GitHub Releases（推荐）**

#### **优势**
```
✅ 完全免费
✅ 全球CDN加速
✅ 高可用性和稳定性
✅ 自动化CI/CD集成
✅ 版本管理和回滚支持
✅ 无需自建服务器
```

#### **配置步骤**

##### **1. 创建GitHub仓库**
```bash
# 在GitHub上创建仓库
仓库名: fifo-audit-system
URL: https://github.com/YOUR_USERNAME/fifo-audit-system
```

##### **2. 更新配置文件**
修改 `src-tauri/tauri.conf.json`:
```json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://github.com/YOUR_USERNAME/fifo-audit-system/releases/latest/download/"
    ],
    "dialog": true,
    "pubkey": "YOUR_PUBLIC_KEY_HERE"
  }
}
```

##### **3. 生成更新签名密钥**
```powershell
# 安装Tauri CLI
npm install -g @tauri-apps/cli

# 生成密钥对
tauri signer generate -w ~/.tauri/myapp.key

# 获取公钥（配置到tauri.conf.json）
tauri signer sign -w ~/.tauri/myapp.key --password YOUR_PASSWORD
```

##### **4. 配置GitHub Actions**
我们已经创建的 `.github/workflows/release.yml` 会：
- 自动构建新版本
- 生成更新签名
- 上传到GitHub Releases
- 创建更新清单文件

### **方案2：自建更新服务器**

#### **服务器要求**
```
✅ HTTPS支持（SSL证书）
✅ 静态文件托管能力
✅ 支持JSON API响应
✅ 足够的带宽和存储
```

#### **文件结构**
```
更新服务器目录/
├── latest.json          # 版本信息
├── v2.0.0/
│   ├── app-update.sig   # 更新签名
│   └── FIFO资金追踪审计系统.exe
├── v2.0.1/
│   ├── app-update.sig
│   └── FIFO资金追踪审计系统.exe
└── changelog/
    ├── v2.0.0.md
    └── v2.0.1.md
```

##### **latest.json 格式**
```json
{
  "version": "2.0.1",
  "notes": "修复重要bug，性能优化",
  "pub_date": "2025-01-13T10:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldUTE1wZz...",
      "url": "https://yourserver.com/updates/v2.0.1/FIFO资金追踪审计系统.exe"
    }
  }
}
```

---

## 🚀 **发布新版本流程**

### **方法1：自动化发布（推荐）**

#### **触发发布**
```bash
# 1. 更新版本号并创建标签
git tag v2.0.1
git push origin v2.0.1

# 2. GitHub Actions自动执行：
#    - 构建新版本
#    - 运行测试
#    - 生成更新签名
#    - 上传到GitHub Releases
#    - 通知用户有新版本
```

#### **发布内容**
GitHub Actions自动生成：
```
✅ 可执行文件: FIFO资金追踪审计系统_2.0.1.exe
✅ MSI安装包: FIFO资金追踪审计系统_2.0.1_x64_zh-CN.msi
✅ 更新签名文件: app-update.sig
✅ 版本信息: latest.json
✅ 发布说明: RELEASE_NOTES.md
✅ 文件哈希: SHA256SUMS.txt
```

### **方法2：手动发布**

#### **使用PowerShell脚本**
```powershell
# 执行发布脚本
.\scripts\release.ps1 -Version "2.0.1" -ReleaseNotes "修复bug，性能优化"

# 脚本自动完成：
# ✅ 更新版本号
# ✅ 构建应用程序
# ✅ 生成发布包
# ✅ 创建Git标签
# ✅ 生成发布说明
```

---

## 👥 **用户更新体验细节**

### **更新检查时机**
```
🕐 应用启动时：延迟3秒后自动检查
🔄 手动检查：设置页面"检查更新"按钮
⚙️ 定期检查：可配置每日/每周自动检查
```

### **更新提示界面**
```
┌─────────────────────────────────┐
│  🎉 发现新版本！                │
│                                 │
│  当前版本: v2.0.0               │
│  最新版本: v2.0.1               │
│                                 │
│  更新内容：                     │
│  • 修复数据处理bug              │
│  • 优化界面响应速度             │
│  • 新增快捷键支持               │
│                                 │
│  [ 立即更新 ]  [ 稍后提醒 ]    │
└─────────────────────────────────┘
```

### **下载进度显示**
```
┌─────────────────────────────────┐
│  📥 正在下载更新...             │
│                                 │
│  ████████████░░░░░░  75%        │
│                                 │
│  下载速度: 2.1 MB/s             │
│  剩余时间: 约 30 秒             │
│                                 │
│  [ 取消下载 ]                   │
└─────────────────────────────────┘
```

### **安装完成提示**
```
┌─────────────────────────────────┐
│  ✅ 更新安装完成！              │
│                                 │
│  新版本已准备就绪，需要重启     │
│  应用程序来使用新功能。         │
│                                 │
│  [ 立即重启 ]  [ 稍后重启 ]    │
└─────────────────────────────────┘
```

---

## ⚙️ **用户设置选项**

### **更新设置界面**
```
应用更新设置
├── ✅ 启动时自动检查更新
├── ✅ 发现新版本时显示通知
├── ⚙️ 更新频率: 每日/每周/手动
├── 🔄 当前版本: v2.0.0
└── [ 检查更新 ] 按钮
```

### **用户可控制选项**
```
✅ 开启/关闭自动更新
✅ 开启/关闭更新通知
✅ 设置更新检查频率
✅ 查看当前版本信息
✅ 手动触发更新检查
✅ 选择立即更新或稍后更新
```

---

## 🔧 **快速配置指南**

### **立即启用更新功能**

#### **第1步：创建GitHub仓库**
```bash
1. 访问 https://github.com/new
2. 仓库名: fifo-audit-system
3. 设为Public（或Private with GitHub Pro）
4. 创建仓库
```

#### **第2步：上传代码**
```bash
# 在项目目录执行
cd "C:\Users\cronu\OneDrive\Desktop\法巴农银工作资料\审计"
git init
git add .
git commit -m "Initial commit"
git remote add origin https://github.com/YOUR_USERNAME/fifo-audit-system.git
git push -u origin main
```

#### **第3步：配置更新地址**
修改 `tauri-app/src-tauri/tauri.conf.json`:
```json
"endpoints": [
  "https://github.com/YOUR_USERNAME/fifo-audit-system/releases/latest/download/"
]
```

#### **第4步：生成签名密钥**
```powershell
# 生成密钥（替换为实际路径）
npx @tauri-apps/cli signer generate -w ./update-key.key

# 获取公钥（复制到tauri.conf.json）
npx @tauri-apps/cli signer sign -w ./update-key.key --password YOUR_PASSWORD
```

#### **第5步：发布首个版本**
```bash
# 创建版本标签
git tag v2.0.0
git push origin v2.0.0

# GitHub Actions自动构建并发布
```

#### **第6步：测试更新功能**
```bash
1. 运行当前版本应用
2. 发布新版本（v2.0.1）
3. 重新启动应用
4. 验证自动更新提示
5. 测试下载和安装流程
```

---

## 📊 **更新统计和监控**

### **可追踪指标**
```
📈 版本分布情况
📥 下载成功率
⏱️ 更新安装时间
❌ 更新失败原因
🌍 地理分布统计
```

### **用户反馈渠道**
```
📧 自动错误报告
🐛 GitHub Issues
📝 用户调研
📞 技术支持邮箱
```

---

## 🎯 **总结**

### **✅ 用户更新体验**
1. **自动检查**：应用启动后自动检查更新
2. **友好提示**：清晰显示版本信息和更新内容
3. **一键更新**：点击按钮即可完成整个流程
4. **无缝升级**：后台下载，自动安装重启

### **✅ 开发者发布流程**
1. **推送标签**：`git tag v2.0.1 && git push origin v2.0.1`
2. **自动构建**：GitHub Actions自动处理
3. **用户通知**：已安装用户自动收到更新通知

### **✅ 联网要求**
- **日常使用**：完全离线
- **更新功能**：需要网络连接
- **用户可控**：可关闭自动更新

### **🔧 配置完成后的效果**
用户将享受到与Chrome、VS Code等专业软件相同的自动更新体验：
- 启动时自动检查
- 友好的更新提示
- 后台下载安装
- 一键重启升级

**现在您只需要按照配置指南设置GitHub仓库和签名密钥，就可以启用完整的自动更新功能！**