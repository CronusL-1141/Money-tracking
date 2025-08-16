@echo off
echo 正在启动FIFO资金追踪审计系统...
echo.
echo 请等待应用窗口出现...
echo 如果没有窗口出现，请检查任务栏
echo.

cd /d "%~dp0"
start "" "src-tauri\target\release\FIFO资金追踪审计系统.exe"

echo.
echo 应用已启动！
echo.
echo 测试清单:
echo 1. 查看应用窗口是否出现
echo 2. 点击"审计分析"页面
echo 3. 测试文件拖拽功能
echo 4. 点击"时点查询"页面
echo 5. 测试文件选择功能
echo.
pause