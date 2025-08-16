# FIFO资金追踪审计系统 - 独立版本构建脚本
# 解决Python环境依赖问题，创建完全独立的可执行文件

param(
    [Parameter(Mandatory=$false)]
    [switch]$CleanBuild = $false,
    
    [Parameter(Mandatory=$false)]
    [string]$OutputDir = "standalone_build"
)

# 配置
$PROJECT_ROOT = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$SRC_DIR = Join-Path $PROJECT_ROOT "src"
$TAURI_DIR = Join-Path $PROJECT_ROOT "tauri-app"
$BUILD_DIR = Join-Path $PROJECT_ROOT $OutputDir

# 颜色输出函数
function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}
function Write-Success { param([string]$Message) Write-ColorOutput "✅ $Message" "Green" }
function Write-Info { param([string]$Message) Write-ColorOutput "ℹ️  $Message" "Cyan" }
function Write-Warning { param([string]$Message) Write-ColorOutput "⚠️  $Message" "Yellow" }
function Write-Error { param([string]$Message) Write-ColorOutput "❌ $Message" "Red" }

Write-Info "🚀 开始构建完全独立的FIFO审计系统"
Write-Info "📁 项目目录: $PROJECT_ROOT"
Write-Info "📁 构建目录: $BUILD_DIR"

try {
    # 1. 环境检查
    Write-Info "🔍 检查构建环境..."
    
    # 检查Python
    $pythonVersion = python --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Python 未安装，无法构建独立版本"
        exit 1
    }
    Write-Success "Python版本: $pythonVersion"
    
    # 检查PyInstaller
    $pyinstallerVersion = pyinstaller --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Info "安装 PyInstaller..."
        pip install pyinstaller
        if ($LASTEXITCODE -ne 0) {
            Write-Error "PyInstaller 安装失败"
            exit 1
        }
    } else {
        Write-Success "PyInstaller版本: $pyinstallerVersion"
    }

    # 2. 创建构建目录
    if ($CleanBuild -and (Test-Path $BUILD_DIR)) {
        Write-Info "🧹 清理旧构建..."
        Remove-Item $BUILD_DIR -Recurse -Force
    }
    
    if (-not (Test-Path $BUILD_DIR)) {
        New-Item -Path $BUILD_DIR -ItemType Directory -Force | Out-Null
    }
    Write-Success "创建构建目录: $BUILD_DIR"

    # 3. 使用PyInstaller打包Python代码
    Write-Info "📦 使用PyInstaller打包Python核心..."
    
    $distDir = Join-Path $BUILD_DIR "python_core"
    $workDir = Join-Path $BUILD_DIR "pyinstaller_work"
    
    Set-Location $SRC_DIR
    
    # 创建PyInstaller规格文件
    $specContent = @"
# -*- mode: python ; coding: utf-8 -*-

block_cipher = None

a = Analysis(
    ['main_new.py'],
    pathex=['$SRC_DIR'],
    binaries=[],
    datas=[
        ('config.py', '.'),
        ('core', 'core'),
        ('services', 'services'),
        ('models', 'models'),
        ('utils', 'utils'),
    ],
    hiddenimports=[
        'pandas',
        'numpy',
        'openpyxl',
        'xlrd',
        'xlsxwriter',
        'datetime',
        'pathlib',
        'sys',
        'os',
        'argparse',
        'logging'
    ],
    hookspath=[],
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name='fifo_audit_core',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
"@
    
    Set-Content -Path "fifo_audit.spec" -Value $specContent -Encoding UTF8
    Write-Success "创建PyInstaller规格文件"
    
    # 执行PyInstaller打包
    pyinstaller --clean --distpath $distDir --workpath $workDir fifo_audit.spec
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Python核心打包失败"
        exit 1
    }
    Write-Success "Python核心打包完成"

    # 4. 修改Tauri配置以使用打包的Python
    Write-Info "🔧 修改Tauri配置..."
    
    $tauriConfig = Join-Path $TAURI_DIR "src-tauri\tauri.conf.json"
    $config = Get-Content $tauriConfig | ConvertFrom-Json
    
    # 添加Python可执行文件到资源
    if (-not $config.tauri.bundle.resources) {
        $config.tauri.bundle.resources = @()
    }
    
    $config.tauri.bundle.resources += @(
        "../$OutputDir/python_core/fifo_audit_core.exe"
    )
    
    # 添加外部二进制文件
    if (-not $config.tauri.bundle.externalBin) {
        $config.tauri.bundle.externalBin = @()
    }
    
    $config.tauri.bundle.externalBin += @(
        "fifo_audit_core.exe"
    )
    
    $config | ConvertTo-Json -Depth 20 | Set-Content $tauriConfig -Encoding UTF8
    Write-Success "Tauri配置已更新"

    # 5. 修改Rust代码以使用嵌入的Python
    Write-Info "🔧 修改Rust代码..."
    
    $rustMainPath = Join-Path $TAURI_DIR "src-tauri\src\main.rs"
    $rustBackup = "${rustMainPath}.backup"
    
    # 备份原始文件
    Copy-Item $rustMainPath $rustBackup -Force
    
    # 创建修改后的Rust代码
    $rustContent = Get-Content $rustMainPath -Raw
    
    # 替换find_python_executable函数
    $newPythonFunction = @"
// 辅助函数：查找Python可执行文件（使用嵌入版本）
fn find_python_executable() -> PathBuf {
    // 在生产环境中使用嵌入的Python
    let exe_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    
    let embedded_python = exe_dir.join("fifo_audit_core.exe");
    
    if embedded_python.exists() {
        info!("Using embedded Python: {:?}", embedded_python);
        return embedded_python;
    }
    
    // 开发环境回退到系统Python
    let candidates = vec!["python", "python3", "py"];
    
    for candidate in candidates {
        if let Ok(path) = which::which(candidate) {
            info!("Using system Python: {:?}", path);
            return path;
        }
    }
    
    warn!("No Python found, using default");
    PathBuf::from("python")
}
"@
    
    # 替换函数
    $rustContent = $rustContent -replace "(?s)// 辅助函数：查找Python可执行文件.*?^}", $newPythonFunction
    
    Set-Content $rustMainPath $rustContent -Encoding UTF8
    Write-Success "Rust代码已修改为使用嵌入Python"

    # 6. 构建Tauri应用
    Write-Info "🏗️  构建Tauri独立应用..."
    
    Set-Location $TAURI_DIR
    
    npm run tauri:build
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tauri应用构建失败"
        # 恢复原始Rust代码
        Copy-Item $rustBackup $rustMainPath -Force
        exit 1
    }
    Write-Success "Tauri应用构建完成"

    # 7. 创建最终分发包
    Write-Info "📦 创建最终分发包..."
    
    $releaseDir = Join-Path $BUILD_DIR "release"
    if (-not (Test-Path $releaseDir)) {
        New-Item -Path $releaseDir -ItemType Directory -Force | Out-Null
    }
    
    # 复制主可执行文件
    $tauriExe = Join-Path $TAURI_DIR "src-tauri\target\release\FIFO资金追踪审计系统.exe"
    if (Test-Path $tauriExe) {
        Copy-Item $tauriExe $releaseDir -Force
        Write-Success "复制主可执行文件"
    }
    
    # 复制Python核心
    $pythonCore = Join-Path $distDir "fifo_audit_core.exe"
    if (Test-Path $pythonCore) {
        Copy-Item $pythonCore $releaseDir -Force
        Write-Success "复制Python核心"
    }
    
    # 创建启动脚本（可选）
    $launchScript = @"
@echo off
echo 正在启动FIFO资金追踪审计系统...
echo.
"%~dp0FIFO资金追踪审计系统.exe"
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo 程序运行出现错误，错误代码: %ERRORLEVEL%
    echo 请检查系统要求或联系技术支持
    pause
)
"@
    
    Set-Content (Join-Path $releaseDir "启动.bat") $launchScript -Encoding ASCII
    Write-Success "创建启动脚本"

    # 8. 创建README
    $readmeContent = @"
# FIFO资金追踪审计系统 - 独立版本

## 📋 系统要求

- Windows 10/11 (64位)
- 内存: 至少 4GB RAM
- 磁盘空间: 至少 200MB 可用空间

## 🚀 使用方法

### 方式1: 直接运行
双击 `FIFO资金追踪审计系统.exe` 即可启动

### 方式2: 使用启动脚本
双击 `启动.bat` 启动（提供错误信息显示）

## ✅ 特点

- ✅ **完全独立**: 无需安装Python、Node.js等环境
- ✅ **免安装**: 解压即用，绿色软件
- ✅ **全功能**: 包含双算法、GUI界面、时点查询等所有功能
- ✅ **高性能**: 支持大数据量处理（50万行）

## 📁 文件说明

- `FIFO资金追踪审计系统.exe` - 主程序
- `fifo_audit_core.exe` - Python核心引擎（自动调用）
- `启动.bat` - 启动脚本（可选使用）
- `README.txt` - 本说明文件

## 🔧 使用步骤

1. **启动程序**: 双击exe文件
2. **选择文件**: 拖拽或点击选择Excel流水文件
3. **选择算法**: FIFO算法或差额计算法
4. **开始分析**: 点击开始分析按钮
5. **查看结果**: 分析完成后可导出Excel报告

## 📞 技术支持

如遇问题请联系技术支持团队。

## 📋 版本信息

- 版本: v2.0.0
- 构建日期: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
- 系统要求: Windows 10/11 64位
"@
    
    Set-Content (Join-Path $releaseDir "README.txt") $readmeContent -Encoding UTF8
    Write-Success "创建使用说明"

    # 9. 恢复原始配置
    Write-Info "🔄 恢复原始配置..."
    
    # 恢复Rust代码
    Copy-Item $rustBackup $rustMainPath -Force
    Remove-Item $rustBackup -Force
    
    # 恢复Tauri配置（移除临时修改）
    git checkout $tauriConfig 2>$null
    
    Write-Success "原始配置已恢复"

    # 10. 生成构建报告
    Write-Info "📊 生成构建报告..."
    
    $reportContent = @"
# 独立版本构建报告

## 构建信息
- 构建时间: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
- 构建类型: 完全独立版本
- Python版本: $pythonVersion
- 构建目录: $BUILD_DIR

## 生成文件
"@
    
    Get-ChildItem $releaseDir -File | ForEach-Object {
        $size = [math]::Round($_.Length / 1MB, 2)
        $reportContent += "`n- **$($_.Name)** - ${size}MB"
    }
    
    $reportContent += @"

## 技术细节
- 使用PyInstaller将Python代码打包为独立可执行文件
- Tauri应用调用嵌入的Python核心，而不是系统Python
- 包含所有必要的Python依赖和库文件
- 支持离线运行，无需任何外部环境

## 测试建议
1. 在干净的Windows系统上测试（无Python环境）
2. 测试文件拖拽和选择功能
3. 测试双算法分析功能
4. 测试时点查询功能
5. 测试Excel导出功能
"@
    
    Set-Content (Join-Path $BUILD_DIR "BUILD_REPORT.md") $reportContent -Encoding UTF8
    Write-Success "构建报告已生成"

    # 11. 最终输出
    Write-Success "🎉 独立版本构建完成！"
    Write-Info ""
    Write-Info "📋 构建摘要:"
    Write-Info "   构建目录: $BUILD_DIR"
    Write-Info "   发布目录: $releaseDir"
    Write-Info ""
    Write-Info "📁 生成文件:"
    Get-ChildItem $releaseDir -File | ForEach-Object {
        $size = [math]::Round($_.Length / 1MB, 2)
        Write-Info "     $($_.Name) (${size}MB)"
    }
    Write-Info ""
    Write-Info "🎯 测试步骤:"
    Write-Info "   1. 将 $releaseDir 目录复制到其他电脑"
    Write-Info "   2. 确保目标电脑没有Python环境"
    Write-Info "   3. 双击 FIFO资金追踪审计系统.exe"
    Write-Info "   4. 测试所有功能"
    Write-Info ""
    Write-Info "✨ 现在您拥有了完全独立的可执行文件！"

} catch {
    Write-Error "构建过程中发生错误: $($_.Exception.Message)"
    exit 1
} finally {
    Set-Location $PROJECT_ROOT
}