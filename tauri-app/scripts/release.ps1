# FIFO资金追踪审计系统 - 专业发布脚本
# Copyright © 2025 审计团队

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    
    [Parameter(Mandatory=$false)]
    [string]$ReleaseNotes = "",
    
    [Parameter(Mandatory=$false)]
    [switch]$SkipTests = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$DryRun = $false
)

# 配置
$PROJECT_ROOT = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$TAURI_DIR = Join-Path $PROJECT_ROOT "tauri-app"
$BUILD_DIR = Join-Path $TAURI_DIR "src-tauri\target\release"
$DIST_DIR = Join-Path $PROJECT_ROOT "dist"

# 颜色输出函数
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Write-Success { param([string]$Message) Write-ColorOutput "✅ $Message" "Green" }
function Write-Info { param([string]$Message) Write-ColorOutput "ℹ️  $Message" "Cyan" }
function Write-Warning { param([string]$Message) Write-ColorOutput "⚠️  $Message" "Yellow" }
function Write-Error { param([string]$Message) Write-ColorOutput "❌ $Message" "Red" }

# 检查版本格式
if (-not ($Version -match "^v?\d+\.\d+\.\d+$")) {
    Write-Error "版本格式错误。请使用格式: 1.0.0 或 v1.0.0"
    exit 1
}

# 确保版本以v开头
if (-not $Version.StartsWith("v")) {
    $Version = "v$Version"
}

Write-Info "🚀 开始发布流程 - 版本: $Version"
Write-Info "📁 项目目录: $PROJECT_ROOT"

# 创建分发目录
if (-not (Test-Path $DIST_DIR)) {
    New-Item -Path $DIST_DIR -ItemType Directory -Force | Out-Null
    Write-Success "创建分发目录: $DIST_DIR"
}

try {
    # 1. 环境检查
    Write-Info "🔍 检查构建环境..."
    
    # 检查Node.js
    $nodeVersion = node --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Node.js 未安装或不在PATH中"
        exit 1
    }
    Write-Success "Node.js版本: $nodeVersion"
    
    # 检查Rust
    $rustVersion = rustc --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Rust 未安装或不在PATH中"
        exit 1
    }
    Write-Success "Rust版本: $rustVersion"
    
    # 检查Python
    $pythonVersion = python --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "Python 未找到，某些功能可能无法正常工作"
    } else {
        Write-Success "Python版本: $pythonVersion"
    }

    # 2. 更新版本号
    Write-Info "📝 更新版本信息..."
    
    Set-Location $TAURI_DIR
    
    # 更新package.json
    $packageJson = Get-Content "package.json" | ConvertFrom-Json
    $packageJson.version = $Version.TrimStart('v')
    $packageJson | ConvertTo-Json -Depth 10 | Set-Content "package.json"
    Write-Success "更新 package.json 版本: $($packageJson.version)"
    
    # 更新Cargo.toml
    $cargoToml = Get-Content "src-tauri\Cargo.toml" -Raw
    $cargoToml = $cargoToml -replace 'version = ".*"', "version = `"$($Version.TrimStart('v'))`""
    Set-Content "src-tauri\Cargo.toml" $cargoToml
    Write-Success "更新 Cargo.toml 版本"

    # 3. 运行测试 (可选)
    if (-not $SkipTests) {
        Write-Info "🧪 运行测试..."
        npm test
        if ($LASTEXITCODE -ne 0) {
            Write-Error "测试失败，发布中止"
            exit 1
        }
        Write-Success "所有测试通过"
    }

    # 4. 清理构建目录
    Write-Info "🧹 清理构建缓存..."
    if (Test-Path "src-tauri\target") {
        Remove-Item "src-tauri\target" -Recurse -Force
        Write-Success "清理构建目录"
    }
    
    npm run clean 2>$null
    Write-Success "清理前端构建缓存"

    # 5. 构建发布版本
    Write-Info "🏗️  开始构建发布版本..."
    
    if ($DryRun) {
        Write-Warning "干运行模式 - 跳过实际构建"
    } else {
        npm run tauri:build
        if ($LASTEXITCODE -ne 0) {
            Write-Error "构建失败"
            exit 1
        }
        Write-Success "构建完成"
    }

    # 6. 复制构建产物
    Write-Info "📦 复制构建产物..."
    
    $releaseDir = Join-Path $DIST_DIR $Version
    if (-not (Test-Path $releaseDir)) {
        New-Item -Path $releaseDir -ItemType Directory -Force | Out-Null
    }

    if (-not $DryRun) {
        # 复制主执行文件
        $exePath = Join-Path $BUILD_DIR "FIFO资金追踪审计系统.exe"
        if (Test-Path $exePath) {
            Copy-Item $exePath (Join-Path $releaseDir "FIFO资金追踪审计系统_$($Version.TrimStart('v')).exe")
            Write-Success "复制可执行文件"
        }

        # 复制MSI安装包
        $msiPath = Get-ChildItem "$BUILD_DIR\bundle\msi\*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($msiPath) {
            Copy-Item $msiPath.FullName (Join-Path $releaseDir "FIFO资金追踪审计系统_$($Version.TrimStart('v'))_x64_zh-CN.msi")
            Write-Success "复制MSI安装包"
        }

        # 计算文件哈希
        Write-Info "🔐 计算文件哈希..."
        Get-ChildItem $releaseDir | ForEach-Object {
            $hash = Get-FileHash $_.FullName -Algorithm SHA256
            "$($hash.Hash)  $($_.Name)" | Out-File (Join-Path $releaseDir "SHA256SUMS.txt") -Append
        }
        Write-Success "生成哈希文件"
    }

    # 7. 生成发布说明
    Write-Info "📄 生成发布说明..."
    
    $releaseNotesPath = Join-Path $releaseDir "RELEASE_NOTES.md"
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    
    $releaseContent = @"
# FIFO资金追踪审计系统 $Version

**发布日期**: $timestamp

## 📋 发布信息

- **版本号**: $Version
- **构建日期**: $timestamp
- **构建环境**: Windows $(Get-WmiObject Win32_OperatingSystem).Version
- **Node.js**: $nodeVersion
- **Rust**: $rustVersion

## 📦 包含文件

"@

    if (-not $DryRun) {
        Get-ChildItem $releaseDir -File | ForEach-Object {
            $size = [math]::Round($_.Length / 1MB, 2)
            $releaseContent += "- **$($_.Name)** - ${size}MB`n"
        }
    }

    if ($ReleaseNotes) {
        $releaseContent += "`n## 🔄 更新内容`n`n$ReleaseNotes`n"
    }

    $releaseContent += @"

## 🚀 安装方式

### MSI安装包 (推荐)
1. 下载 `FIFO资金追踪审计系统_$($Version.TrimStart('v'))_x64_zh-CN.msi`
2. 双击运行安装程序
3. 按照向导完成安装

### 便携版本
1. 下载 `FIFO资金追踪审计系统_$($Version.TrimStart('v')).exe`
2. 直接运行，无需安装

## 🔐 安全验证

请使用 SHA256SUMS.txt 文件验证下载文件的完整性：

``````
# PowerShell
Get-FileHash .\FIFO资金追踪审计系统_$($Version.TrimStart('v')).exe -Algorithm SHA256
``````

## 📞 技术支持

- GitHub: https://github.com/your-org/fifo-audit/releases/tag/$Version
- 邮箱: support@yourcompany.com

---
*由自动化发布流程生成*
"@

    Set-Content $releaseNotesPath $releaseContent -Encoding UTF8
    Write-Success "生成发布说明: $releaseNotesPath"

    # 8. Git标签 (如果在Git仓库中)
    if (Test-Path (Join-Path $PROJECT_ROOT ".git")) {
        Write-Info "🏷️  创建Git标签..."
        
        if (-not $DryRun) {
            git tag -a $Version -m "Release $Version"
            if ($LASTEXITCODE -eq 0) {
                Write-Success "创建Git标签: $Version"
                Write-Info "推送标签到远程仓库: git push origin $Version"
            } else {
                Write-Warning "Git标签创建失败，请手动创建"
            }
        } else {
            Write-Warning "干运行模式 - 跳过Git标签创建"
        }
    }

    # 9. 发布摘要
    Write-Success "🎉 发布流程完成！"
    Write-Info ""
    Write-Info "📋 发布摘要:"
    Write-Info "   版本: $Version"
    Write-Info "   构建时间: $timestamp"
    Write-Info "   发布目录: $releaseDir"
    
    if (-not $DryRun -and (Test-Path $releaseDir)) {
        Write-Info "   文件列表:"
        Get-ChildItem $releaseDir -File | ForEach-Object {
            $size = [math]::Round($_.Length / 1MB, 2)
            Write-Info "     - $($_.Name) (${size}MB)"
        }
    }
    
    Write-Info ""
    Write-Info "🚀 后续步骤:"
    Write-Info "   1. 测试发布版本"
    Write-Info "   2. 上传到GitHub Releases"
    Write-Info "   3. 更新官方网站"
    Write-Info "   4. 发送发布通知"

} catch {
    Write-Error "发布过程中发生错误: $($_.Exception.Message)"
    exit 1
} finally {
    Set-Location $PROJECT_ROOT
}

Write-Success "✨ 发布脚本执行完毕"