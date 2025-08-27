// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio::process::{Command};
use std::process::Stdio;
use std::path::PathBuf;

use std::fs;
use tokio::io::{BufReader, AsyncBufReadExt};
use tauri::{command, Manager};
use tauri::State;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use log::{info, warn, error};
use regex::Regex;
use std::sync::Arc;

// 引入Rust后端库
use audit_backend::{AuditService, TauriAuditConfig, TimePointService, TimePointQueryRequest, TimePointQueryResult, FundPoolQueryRequest, FundPoolQueryResult};

// 引入模块化命令
mod commands;

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{BOOL, HWND},
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
};

#[cfg(target_os = "windows")]
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

// Windows深色主题设置函数
#[cfg(target_os = "windows")]
fn set_window_theme(window: &tauri::Window, dark_mode: bool) {
    let handle = window.raw_window_handle();
    if let RawWindowHandle::Win32(win32_handle) = handle {
        let hwnd = HWND(win32_handle.hwnd as isize);
        let dark_mode_flag: BOOL = BOOL(if dark_mode { 1 } else { 0 });
        
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &dark_mode_flag as *const _ as *const _,
                std::mem::size_of::<BOOL>() as u32,
            );
        }
    }
}

// Tauri命令：设置窗口主题
#[command]
async fn set_window_dark_mode(window: tauri::Window, dark_mode: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        set_window_theme(&window, dark_mode);
        info!("Window theme set to: {}", if dark_mode { "dark" } else { "light" });
    }
    Ok(())
}

// 数据类型定义
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditConfig {
    pub algorithm: String,
    pub input_file: String,
    pub output_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub output_files: Vec<String>,
    // 新增：分析统计信息
    pub statistics: Option<AnalysisStatistics>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisStatistics {
    pub total_records: u32,
    pub processing_time: u64,  // 毫秒
    pub validation_errors: u32,
    pub validation_fixes: u32,
    pub algorithm: String,
    pub input_file_name: String,
    pub input_file_size: u64,
    pub output_file_size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimePointQuery {
    pub file_path: String,
    pub row_number: u32,
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryHistory {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub file_path: String,
    pub row_number: u32,
    pub algorithm: String,
    pub result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub default_algorithm: String,
    pub auto_export: bool,
    pub max_history: usize,
    pub language: String,
    pub theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub exists: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessStatus {
    pub running: bool,
    pub command: Option<String>,
    pub progress: Option<f32>,
    pub message: Option<String>,
    pub output_log: Vec<String>,
    pub process_id: Option<u32>, // 添加进程ID字段
}

// 应用状态管理
pub struct AppState {
    pub query_history: Mutex<Vec<QueryHistory>>,
    pub current_process: Mutex<ProcessStatus>,
    pub app_config: Mutex<AppConfig>,
    pub audit_service: Arc<AuditService>,  // 添加Rust后端服务
}

// Tauri命令：获取可用算法列表
#[command]
async fn get_algorithms() -> Result<Vec<String>, String> {
    Ok(vec!["FIFO".to_string(), "BALANCE_METHOD".to_string()])
}

// Tauri命令：运行Rust后端审计分析（新增）
#[command]
async fn run_rust_audit(config: AuditConfig, state: State<'_, AppState>) -> Result<AuditResult, String> {
    info!("Starting Rust audit with algorithm: {}, input: {}", config.algorithm, config.input_file);
    
    // 步骤0: 并发控制 - 检查是否已有分析在运行
    {
        let process_status = state.current_process.lock().await;
        if process_status.running {
            warn!("Analysis already running, rejecting new request");
            return Err("分析正在进行中，请等待当前分析完成后再试".to_string());
        }
    }
    
    // 步骤1: 简化初始化，保留现有日志（如文件选择记录）
    {
        let mut process_status = state.current_process.lock().await;
        let existing_logs = process_status.output_log.clone(); // 保留现有日志
        *process_status = ProcessStatus {
            running: true,
            command: Some(format!("rust_audit_{}", config.algorithm)),
            progress: Some(0.0),
            message: Some("开始分析...".to_string()),
            output_log: existing_logs, // 保留现有日志而不是清空
            process_id: None,
        };
    }
    
    // 步骤2: 使用一个更简单的解决方案
    // 在分析开始时就设置一个标记，让前端轮询能获取到实时日志
    
    let tauri_config = TauriAuditConfig {
        algorithm: config.algorithm.clone(),
        input_file: config.input_file.clone(),
        output_file: config.output_file.clone(),
    };
    
    // 步骤3: 创建服务并执行分析，使用共享状态机制
    let service = AuditService::new().with_suppress_output(false);
    
    // 步骤3.1: 并行执行分析和实时日志同步
    let state_clone = state.inner().clone();
    let service_clone = Arc::new(service);
    let service_for_analysis = service_clone.clone();
    let service_for_sync = service_clone.clone();
    
    // 分析任务
    let analysis_task = async move {
        service_for_analysis.run_audit_for_gui(tauri_config).await
    };
    
    // 同步任务
    let sync_task = async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        let mut last_count = 0;
        
        loop {
            interval.tick().await;
            
            let current_logs = service_for_sync.get_output_logs();
            if current_logs.len() > last_count {
                let mut process_status = state_clone.current_process.lock().await;
                process_status.output_log = current_logs.clone();
                println!("🔍 实时同步: 更新了 {} 条日志 (新增 {} 条)", 
                    current_logs.len(), current_logs.len() - last_count);
                last_count = current_logs.len();
            }
        }
    };
    
    // 并行执行：分析完成时自动取消同步任务
    let result = tokio::select! {
        analysis_result = analysis_task => {
            println!("🔍 分析任务完成");
            analysis_result
        },
        _ = sync_task => {
            // 这个分支不应该执行
            return Err("同步任务意外完成".to_string());
        }
    };
    
    // 最后一次同步确保所有日志都被获取
    let final_logs = service_clone.get_output_logs();
    if !final_logs.is_empty() {
        let mut process_status = state.current_process.lock().await;
        process_status.output_log = final_logs;
    }
    
    // 步骤4: 转换结果并重置状态
    let final_result = match result.success {
        true => {
            {
                let mut process_status = state.current_process.lock().await;
                process_status.output_log.push(format!("[{}] ✅ {}分析完成", 
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    match config.algorithm.as_str() {
                        "FIFO" => "FIFO算法",
                        "BALANCE_METHOD" => "差额计算法",
                        _ => "审计"
                    }
                ));
                process_status.running = false;
                process_status.command = None;
                process_status.progress = Some(100.0);
                process_status.message = Some("分析完成".to_string());
            }
            
            // 收集统计信息
            let input_file_metadata = std::fs::metadata(&config.input_file).ok();
            let output_file_metadata = if !result.output_files.is_empty() {
                std::fs::metadata(&result.output_files[0]).ok()
            } else {
                None
            };
            
            let statistics = if let Some(ref data) = result.data {
                AnalysisStatistics {
                    total_records: data.transaction_count as u32,
                    processing_time: (data.processing_time * 1000.0) as u64, // 转换为毫秒
                    validation_errors: 0, // TODO: 从validation result中获取
                    validation_fixes: 0,  // TODO: 从validation result中获取
                    algorithm: config.algorithm.clone(),
                    input_file_name: std::path::Path::new(&config.input_file)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("未知文件")
                        .to_string(),
                    input_file_size: input_file_metadata.map(|m| m.len()).unwrap_or(0),
                    output_file_size: output_file_metadata.map(|m| m.len()),
                }
            } else {
                // 如果没有数据，使用默认值
                AnalysisStatistics {
                    total_records: 0,
                    processing_time: 0,
                    validation_errors: 0,
                    validation_fixes: 0,
                    algorithm: config.algorithm.clone(),
                    input_file_name: std::path::Path::new(&config.input_file)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("未知文件")
                        .to_string(),
                    input_file_size: input_file_metadata.map(|m| m.len()).unwrap_or(0),
                    output_file_size: output_file_metadata.map(|m| m.len()),
                }
            };
            
            AuditResult {
                success: true,
                message: result.message,
                data: result.data.map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null)),
                output_files: result.output_files,
                statistics: Some(statistics),
                error: None,
            }
        }
        false => {
            {
                let mut process_status = state.current_process.lock().await;
                process_status.output_log.push(format!("[{}] ❌ {}分析失败: {}", 
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    match config.algorithm.as_str() {
                        "FIFO" => "FIFO算法",
                        "BALANCE_METHOD" => "差额计算法",
                        _ => "审计"
                    },
                    result.message
                ));
                process_status.running = false;
                process_status.command = None;
                process_status.progress = None;
                process_status.message = Some("分析失败".to_string());
            }
            
            AuditResult {
                success: false,
                message: "分析失败".to_string(),
                data: None,
                output_files: vec![],
                statistics: None,
                error: Some(result.message),
            }
        }
    };
    
    Ok(final_result)
}

// Tauri命令：运行审计分析（使用Rust后端）
#[command]
async fn run_audit(config: AuditConfig, state: State<'_, AppState>) -> Result<AuditResult, String> {
    // 直接调用Rust后端实现，复用上面的逻辑
    return run_rust_audit(config, state).await;
}

// Tauri命令：时点查询（旧Python版本，保留作为备用）
#[command]
async fn time_point_query_python(query: TimePointQuery, state: State<'_, AppState>) -> Result<QueryResult, String> {
    info!("Time point query: file={}, row={}, algorithm={}", query.file_path, query.row_number, query.algorithm);
    
    let python_exe = find_python_executable();
    let project_root = get_project_root()?;
    let script_path = project_root.join("src").join("services").join("query_cli.py");
    
    let mut cmd = Command::new(&python_exe);
    cmd.current_dir(&project_root)
        .env("PYTHONIOENCODING", "utf-8")  // 设置UTF-8编码
        .env("PYTHONLEGACYWINDOWSSTDIO", "utf-8")  // Windows兼容性
        .arg("-u")  // 无缓冲模式，立即输出
        .arg(script_path)
        .arg("--file")
        .arg(&query.file_path)
        .arg("--row")
        .arg(&query.row_number.to_string())
        .arg("--algorithm")
        .arg(&query.algorithm)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // 初始化时点查询状态
    {
        let mut process_status = state.current_process.lock().await;
        
        if !process_status.output_log.is_empty() {
            process_status.output_log.push(format!("[{}] ===== 开始时点查询 =====", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            ));
        }
        
        process_status.output_log.push(format!("[{}] 🔍 执行时点查询: 第{}行", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), query.row_number
        ));
        process_status.output_log.push(format!("[{}] 📁 文件: {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), 
            query.file_path.split(&['/', '\\'][..]).last().unwrap_or(&query.file_path)
        ));
        process_status.output_log.push(format!("[{}] 🔧 算法: {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), 
            match query.algorithm.as_str() {
                "FIFO" => "FIFO先进先出算法",
                "BALANCE_METHOD" => "差额计算法",
                _ => &query.algorithm
            }
        ));
    }
    
    let result = match cmd.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();
            
            // 采用与资金分析完全一致的方式：在主线程中实时处理stderr
            let mut stderr_reader = BufReader::new(stderr);
            let stdout = stdout;
            
            let mut stdout_lines = Vec::new();
            let mut stderr_lines = Vec::new();
            
            // 异步任务只负责收集stdout，不更新状态
            let stdout_handle = tokio::spawn(async move {
                let mut stdout_reader = BufReader::new(stdout);
                let mut lines = Vec::new();
                loop {
                    let mut line = String::new();
                    match stdout_reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            line = line.trim_end().to_string();
                            if !line.is_empty() {
                                lines.push(line);
                            }
                        }
                        Err(e) => {
                            error!("Error reading stdout line: {}", e);
                            break;
                        }
                    }
                }
                lines
            });
            
            // 主线程实时读取stderr并更新日志 - 与资金分析一致
            loop {
                let mut line = String::new();
                match stderr_reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let line_str = line.trim_end().to_string();
                        if !line_str.is_empty() {
                            stderr_lines.push(line_str.clone());
                            
                            // 实时更新进程状态 - 与资金分析完全一致的模式
                            {
                                let mut process_status = state.current_process.lock().await;
                                process_status.output_log.push(format!("[{}] {}", 
                                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), 
                                    line_str
                                ));
                                
                                // 限制日志长度，避免内存占用过多
                                if process_status.output_log.len() > 1000 {
                                    process_status.output_log.drain(0..100);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading stderr line: {}", e);
                        break;
                    }
                }
            }
            
            // 等待stdout收集完成
            if let Ok(stdout_result) = stdout_handle.await {
                stdout_lines = stdout_result;
            }
            
            // 等待子进程完成
            let exit_status = child.wait().await;
            let stdout_output = stdout_lines.join("\n");
            let _stderr_output = stderr_lines.join("\n");  // 加前缀避免unused warning
            
            match exit_status {
                Ok(status) if status.success() => {
                    info!("Time point query completed successfully");
                    let parsed_data = parse_query_output(&stdout_output);
                    info!("解析后的数据: {:?}", parsed_data.is_some());
                    QueryResult {
                        success: true,
                        data: parsed_data,
                        message: "查询完成".to_string(), // 简化消息，详细日志已实时显示
                    }
                }
                Ok(_) => {
                    warn!("Time point query failed with non-zero exit code");
                    QueryResult {
                        success: false,
                        data: None,
                        message: "查询失败，请查看日志了解详情".to_string(),
                    }
                }
                Err(e) => {
                    error!("Failed to wait for time point query process: {}", e);
                    QueryResult {
                        success: false,
                        data: None,
                        message: format!("进程等待失败: {}", e),
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to spawn time point query process: {}", e);
            QueryResult {
                success: false,
                data: None,
                message: format!("进程启动失败: {}", e),
            }
        }
    };
    
    // 添加到查询历史
    if result.success {
        let history_entry = QueryHistory {
            id: generate_id(),
            timestamp: Utc::now(),
            file_path: query.file_path.clone(),
            row_number: query.row_number,
            algorithm: query.algorithm.clone(),
            result: Some(result.message.clone()),
        };
        
        let mut history = state.query_history.lock().await;
        history.push(history_entry);
        
        // 保持历史记录数量限制
        let config = state.app_config.lock().await;
        let max_history = config.max_history;
        drop(config);
        
        if history.len() > max_history {
            let len = history.len();
            history.drain(0..len - max_history);
        }
    }
    
    Ok(result)
}

// Tauri命令：检查系统环境
#[command]
async fn check_system_env() -> Result<serde_json::Value, String> {
    println!("check_system_env 命令被调用");
    
    // 检测是否为开发环境
    let is_dev_mode = cfg!(debug_assertions);
    println!("开发模式: {}", is_dev_mode);
    
    // 检查临时目录访问权限
    let temp_dir_available = match std::env::temp_dir().metadata() {
        Ok(_) => true,
        Err(_) => false,
    };
    
    // 检查工作目录权限
    let work_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let work_dir_writable = match std::fs::create_dir_all(&work_dir.join("temp_analysis_results")) {
        Ok(_) => true,
        Err(e) => {
            if is_dev_mode {
                println!("Dev mode: Cannot create temp_analysis_results directory: {}", e);
                println!("Working directory: {}", work_dir.display());
            }
            false
        }
    };
    
    // 在开发环境中放宽检查要求
    let file_system_ok = if is_dev_mode {
        temp_dir_available // 开发环境只需要临时目录可用
    } else {
        temp_dir_available && work_dir_writable // 生产环境需要更严格的检查
    };
    
    // 检查内存情况（简单检查）
    let memory_available = true; // Rust自身能运行说明内存基本够用
    
    // 系统架构信息
    let os_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    
    // 环境模式信息
    let env_mode = if is_dev_mode { "开发模式" } else { "生产模式" };
    let backend_version = if is_dev_mode { "v2.0.0-Dev" } else { "v2.0.0" };
    
    let result = serde_json::json!({
        "system_available": file_system_ok && memory_available,
        "file_system_access": file_system_ok,
        "temp_directory_access": temp_dir_available,
        "work_directory_writable": work_dir_writable,
        "memory_available": memory_available,
        "system_info": os_info,
        "work_directory": work_dir.to_string_lossy(),
        "backend_engine": format!("Rust Native Backend ({})", env_mode),
        "backend_version": backend_version,
        "is_dev_mode": is_dev_mode
    });
    
    println!("系统环境检查结果: {:?}", result);
    Ok(result)
}

// Tauri命令：获取查询历史
#[command]
async fn get_query_history(state: State<'_, AppState>) -> Result<Vec<QueryHistory>, String> {
    let history = state.query_history.lock().await;
    Ok(history.clone())
}

// Tauri命令：清空查询历史
#[command]
async fn clear_query_history(state: State<'_, AppState>) -> Result<(), String> {
    let mut history = state.query_history.lock().await;
    history.clear();
    info!("Query history cleared");
    Ok(())
}

// Tauri命令：停止当前分析
#[command]
async fn stop_analysis(state: State<'_, AppState>) -> Result<bool, String> {
    let mut process_status = state.current_process.lock().await;
    
    if process_status.running {
        process_status.output_log.push(format!("[{}] ⏹️ 用户停止分析", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));
        
        // 尝试终止Python进程
        let mut process_killed = false;
        if let Some(process_id) = process_status.process_id {
            process_status.output_log.push(format!("[{}] 🔄 正在终止Python进程 (PID: {})...", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), process_id
            ));
            
            // 在Windows上使用taskkill命令终止进程
            match Command::new("taskkill")
                .arg("/F")  // 强制终止
                .arg("/PID") 
                .arg(process_id.to_string())
                .output()
                .await
            {
                Ok(output) => {
                    if output.status.success() {
                        process_killed = true;
                        process_status.output_log.push(format!("[{}] ✅ Python进程已成功终止", 
                            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
                        ));
                    } else {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        process_status.output_log.push(format!("[{}] ⚠️ 无法终止Python进程: {}", 
                            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), error_msg
                        ));
                    }
                }
                Err(e) => {
                    process_status.output_log.push(format!("[{}] ❌ 终止进程时发生错误: {}", 
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), e
                    ));
                }
            }
        }
        
        // 重置状态
        process_status.running = false;
        process_status.command = None;
        process_status.progress = Some(0.0);  // 重置进度条
        process_status.process_id = None;     // 清除进程ID
        process_status.message = Some(if process_killed { 
            "分析已停止，进程已终止".to_string() 
        } else { 
            "分析已停止".to_string() 
        });
        
        info!("Analysis stopped by user - Process termination: {}", process_killed);
        Ok(true)
    } else {
        Ok(false)
    }
}

// Tauri命令：清空分析日志
#[command]
async fn clear_analysis_log(state: State<'_, AppState>) -> Result<(), String> {
    let mut process_status = state.current_process.lock().await;
    
    if !process_status.running {
        process_status.output_log.clear();
        process_status.output_log.push(format!("[{}] 📝 日志已清空", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));
        info!("Analysis log cleared");
        Ok(())
    } else {
        Err("无法在分析进行中清空日志".to_string())
    }
}

// Tauri命令：删除历史记录项
#[command]
async fn delete_query_history_item(id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let mut history = state.query_history.lock().await;
    if let Some(pos) = history.iter().position(|item| item.id == id) {
        history.remove(pos);
        info!("Deleted query history item: {}", id);
        Ok(true)
    } else {
        Ok(false)
    }
}

// Tauri命令：获取进程状态
#[command]
async fn get_process_status(state: State<'_, AppState>) -> Result<ProcessStatus, String> {
    let status = state.current_process.lock().await;
    Ok((*status).clone())
}

// Tauri命令：获取应用配置
#[command]
async fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.app_config.lock().await;
    Ok((*config).clone())
}

// Tauri命令：更新应用配置
#[command]
async fn update_app_config(new_config: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.app_config.lock().await;
    *config = new_config;
    info!("App config updated");
    Ok(())
}

// Tauri命令：获取文件信息
#[command]
async fn get_file_info(path: String) -> Result<FileInfo, String> {
    let file_path = PathBuf::from(&path);
    let exists = file_path.exists();
    
    if !exists {
        return Ok(FileInfo {
            path,
            name: file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            size: 0,
            modified: Utc::now(),
            exists: false,
        });
    }
    
    let metadata = fs::metadata(&file_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    let modified = metadata.modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Invalid modification time: {}", e))?;
    
    Ok(FileInfo {
        path,
        name: file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string(),
        size: metadata.len(),
        modified: DateTime::from_timestamp(modified.as_secs() as i64, 0)
            .unwrap_or_else(|| Utc::now()),
        exists: true,
    })
}

// Tauri命令：导出查询结果
#[command]
async fn export_query_result(
    query_id: String, 
    output_path: String, 
    state: State<'_, AppState>
) -> Result<bool, String> {
    let history = state.query_history.lock().await;
    
    if let Some(query) = history.iter().find(|h| h.id == query_id) {
        let export_data = serde_json::json!({
            "query_info": {
                "id": query.id,
                "timestamp": query.timestamp,
                "file_path": query.file_path,
                "row_number": query.row_number,
                "algorithm": query.algorithm
            },
            "result": query.result
        });
        
        fs::write(&output_path, serde_json::to_string_pretty(&export_data).unwrap())
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        
        info!("Exported query result to: {}", output_path);
        Ok(true)
    } else {
        Ok(false)
    }
}

// Tauri命令：验证文件路径
#[command]
async fn validate_file_path(path: String) -> Result<bool, String> {
    let file_path = PathBuf::from(path);
    Ok(file_path.exists() && file_path.is_file())
}

// 辅助函数：查找Python可执行文件
fn find_python_executable() -> PathBuf {
    // 按优先级查找Python
    let candidates = vec!["python", "python3", "py"];
    
    for candidate in candidates {
        if let Ok(path) = which::which(candidate) {
            return path;
        }
    }
    
    // 如果都找不到，返回默认的python
    PathBuf::from("python")
}

// 辅助函数：获取项目根目录
fn get_project_root() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    
    // 开发模式：从tauri-app目录返回上级目录
    // 生产模式：可能需要不同的逻辑
    let mut path = exe_path.parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf();
    
    // 尝试找到项目根目录（包含src目录的目录）
    for _ in 0..5 { // 最多向上查找5级
        if path.join("src").join("main.py").exists() {
            return Ok(path);
        }
        if let Some(parent) = path.parent() {
            path = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    // 如果找不到，返回当前目录的上级目录
    Ok(PathBuf::from(".."))
}

// 辅助函数：生成唯一ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("id_{}", timestamp)
}

// 辅助函数：解析Python输出中的文件列表
fn extract_output_files(output: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in output.lines() {
        if line.contains("输出文件:") || line.contains("Output file:") {
            if let Some(file_path) = line.split_once(":").map(|(_, path)| path.trim()) {
                files.push(file_path.to_string());
            }
        }
    }
    files
}

// 辅助函数：解析查询输出为JSON
fn parse_query_output(output: &str) -> Option<serde_json::Value> {
    info!("开始解析查询输出，总字符数: {}", output.len());
    info!("Python stdout输出内容:\n{}", output);
    
    let lines: Vec<&str> = output.lines().collect();
    info!("输出共 {} 行", lines.len());
    
    // 查找JSON标记块
    let mut json_start_idx = None;
    let mut json_end_idx = None;
    
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "JSON_RESULT_START" {
            json_start_idx = Some(i + 1);
            info!("找到JSON_RESULT_START标记，位置: 第{}行", i);
        } else if line.trim() == "JSON_RESULT_END" {
            json_end_idx = Some(i);
            info!("找到JSON_RESULT_END标记，位置: 第{}行", i);
            break;
        }
    }
    
    info!("JSON标记索引 - 开始: {:?}, 结束: {:?}", json_start_idx, json_end_idx);
    
    // 如果找到标记，解析标记之间的JSON
    if let (Some(start), Some(end)) = (json_start_idx, json_end_idx) {
        if start < end && start < lines.len() {
            let json_line = lines[start];
            info!("准备解析JSON行: {}", json_line);
            match serde_json::from_str(json_line.trim()) {
                Ok(json) => {
                    info!("JSON解析成功");
                    return Some(json);
                }
                Err(e) => {
                    error!("JSON解析失败: {}", e);
                }
            }
        } else {
            error!("JSON标记索引无效: start={}, end={}, lines.len()={}", start, end, lines.len());
        }
    } else {
        error!("未找到有效的JSON标记对");
    }
    
    // 兼容性：尝试从输出中提取单行JSON数据（旧格式）
    warn!("尝试使用兼容性模式解析JSON");
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with('{') && line.trim().ends_with('}') {
            info!("找到潜在JSON行 (第{}行): {}", i, line);
            match serde_json::from_str(line.trim()) {
                Ok(json) => {
                    info!("兼容性模式JSON解析成功");
                    return Some(json);
                }
                Err(e) => {
                    warn!("兼容性模式JSON解析失败: {}", e);
                }
            }
        }
    }
    
    error!("所有JSON解析方法都失败了");
    None
}

// 辅助函数：创建默认配置
fn create_default_config() -> AppConfig {
    AppConfig {
        default_algorithm: "FIFO".to_string(),
        auto_export: false,
        max_history: 100,
        language: "zh".to_string(),
        theme: "light".to_string(),
    }
}

// 辅助函数：创建应用状态
fn create_app_state() -> AppState {
    AppState {
        query_history: Mutex::new(Vec::new()),
        current_process: Mutex::new(ProcessStatus {
            running: false,
            command: None,
            progress: None,
            message: None,
            output_log: Vec::new(),
            process_id: None,
        }),
        app_config: Mutex::new(create_default_config()),
        audit_service: Arc::new(AuditService::new()),  // 添加Rust审计服务
    }
}

// 辅助函数：限制进度值为2位小数
fn round_progress(progress: f32) -> f32 {
    // 使用更严格的精度控制方法
    // 先转换为字符串，再解析回f32以确保精度
    let formatted = format!("{:.2}", progress);
    formatted.parse::<f32>().unwrap_or(progress)
}

// 辅助函数：从输出行解析进度百分比
fn parse_progress_from_line(line: &str) -> f32 {
    // 1. 解析新格式的处理进度 "⏳ 处理进度: 1,000/9,799 (10.2%)"
    if let Ok(re) = Regex::new(r"处理进度:\s*([\d,]+)/([\d,]+)\s*\((\d+\.?\d*)%\)") {
        if let Some(captures) = re.captures(line) {
            if let Some(percent_str) = captures.get(3) {
                if let Ok(percent) = percent_str.as_str().parse::<f32>() {
                    // 先对输入的百分比进行精度控制
                    let percent_rounded = round_progress(percent);
                    // 处理阶段占35%-88%，基于实际时间分布(53%)
                    let progress = 35.0 + (percent_rounded * 0.53);
                    return round_progress(progress); // 限制为2位小数
                }
            }
        }
    }
    
    // 2. 解析简单的处理进度格式 "处理进度: X/Y"
    if let Ok(re) = Regex::new(r"处理进度:\s*([\d,]+)/([\d,]+)") {
        if let Some(captures) = re.captures(line) {
            if let (Some(current_str), Some(total_str)) = (captures.get(1), captures.get(2)) {
                // 移除逗号分隔符
                let current_clean = current_str.as_str().replace(",", "");
                let total_clean = total_str.as_str().replace(",", "");
                
                if let (Ok(current), Ok(total)) = (
                    current_clean.parse::<f32>(), 
                    total_clean.parse::<f32>()
                ) {
                    if total > 0.0 {
                        let data_progress = (current / total) * 100.0;
                        let data_progress_rounded = round_progress(data_progress);
                        let progress = 35.0 + (data_progress_rounded * 0.53); // 35% + (进度 * 53%)
                        return round_progress(progress); // 限制为2位小数
                    }
                }
            }
        }
    }
    
    // 3. 精确匹配特定关键词并映射到进度阶段
    // 初始化和启动阶段 (0-35%)
    if line.contains("🚀 启动算法:") && (line.contains("FIFO") || line.contains("BALANCE_METHOD")) {
        return 5.0;  // 算法启动信息
    } else if line.contains("📊 开始数据预处理") {
        return 10.0;
    } else if line.contains("✅ 数据预处理完成") {
        return 25.0;  // 数据预处理占20%时间
    } else if line.contains("🔍 开始流水完整性验证") {
        return 26.0;
    } else if line.contains("✅ 流水完整性验证通过") {
        return 30.0;  // 流水验证占5%时间
    } else if line.contains("🔎 开始数据验证") {
        return 31.0;
    } else if line.contains("✅ 数据验证通过") {
        return 33.0;  // 数据验证占5%时间
    } else if line.contains("💰 计算初始余额") {
        return 34.0;  // 瞬间完成
    } else if line.contains("🚀 开始") && line.contains("资金追踪分析") {
        return 35.0; // 开始数据处理阶段
    } else if line.contains("📋 总共需要处理") && line.contains("条交易记录") {
        return 35.0; // 开始数据处理
    // 数据处理完成阶段 (88-100%)  
    } else if line.contains("✅ 所有") && line.contains("条交易记录处理完成") {
        return 88.0;  // 数据处理完成，占53%时间
    } else if line.contains("📈 生成分析结果") {
        return 90.0;
    } else if line.contains("💾 保存分析结果到:") {
        return 95.0;
    } else if line.contains("📋 生成场外资金池记录:") {
        return 98.0;
    } else if line.contains("✅") && (line.contains("算法分析完成") || line.contains("FIFO算法分析完成") || line.contains("BALANCE_METHOD算法分析完成")) {
        return 100.0;
    }
    
    0.0 // 默认返回0，表示没有进度更新
}

// 辅助函数：从输出行提取显示消息
fn extract_message_from_line(line: &str) -> String {
    // 移除时间戳和日志级别前缀
    let mut cleaned = if let Ok(re) = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d+ - \w+ - ") {
        re.replace(line, "").to_string()
    } else {
        line.to_string()
    };
    
    // 移除Python输出中的百分比，避免与前端显示重复
    // 匹配格式： "⏳ 处理进度: 2,000/9,799 (20.4%)" -> "⏳ 处理进度: 2,000/9,799"
    if let Ok(re) = Regex::new(r"\s*\([\d.]+%\)") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    
    // 如果行太长，截断显示（安全处理UTF-8字符边界）
    if cleaned.chars().count() > 80 {
        let truncated: String = cleaned.chars().take(77).collect();
        format!("{}...", truncated)
    } else {
        cleaned
    }
}

// 已迁移到 commands::query_fund_pool

// Tauri命令：打开本地文件
#[command]
async fn open_file(file_path: String) -> Result<(), String> {
    info!("Attempting to open file: {}", file_path);
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let result = Command::new("cmd")
            .args(&["/C", "start", "", &file_path])
            .spawn();
            
        match result {
            Ok(_) => {
                info!("Successfully opened file: {}", file_path);
                Ok(())
            },
            Err(e) => {
                error!("Failed to open file {}: {}", file_path, e);
                Err(format!("Failed to open file: {}", e))
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // 对于非Windows系统，使用其他方法
        use std::process::Command;
        let result = if cfg!(target_os = "macos") {
            Command::new("open").arg(&file_path).spawn()
        } else {
            // Linux或其他Unix系统
            Command::new("xdg-open").arg(&file_path).spawn()
        };
        
        match result {
            Ok(_) => {
                info!("Successfully opened file: {}", file_path);
                Ok(())
            },
            Err(e) => {
                error!("Failed to open file {}: {}", file_path, e);
                Err(format!("Failed to open file: {}", e))
            }
        }
    }
}

fn main() {
    // 初始化日志
    env_logger::init();
    
    // 创建应用状态
    let app_state = create_app_state();
    
    info!("Starting FIFO Audit Desktop Application");
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_algorithms,
            run_audit,
            run_rust_audit,  // 新增Rust后端命令
            commands::time_point_query_rust,
            check_system_env,
            get_query_history,
            clear_query_history,
            delete_query_history_item,
            stop_analysis,
            clear_analysis_log,
            get_process_status,
            get_app_config,
            update_app_config,
            get_file_info,
            export_query_result,
            validate_file_path,
            set_window_dark_mode,
            commands::query_fund_pool,
            open_file  // 新增打开文件命令
        ])
        .setup(|app| {
            info!("Application setup completed");
            
            // 初始化Windows窗口主题（默认浅色）
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_window("main") {
                    set_window_theme(&window, false);
                }
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}