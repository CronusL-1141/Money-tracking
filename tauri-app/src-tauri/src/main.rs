/*
 * FLUX资金追踪分析系统 v3.3.4
 * Copyright (c) 2025 刘光浚
 * 开发完成日期: 2025年8月28日
 */

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 移除了Python进程相关导入 - 现在使用Rust后端
use std::path::PathBuf;

use std::fs;
// 移除了Python输出读取相关导入
use tauri::{command, Manager};
use tauri::State;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use log::{info, warn, error};
use regex::Regex;
use std::sync::Arc;

// 引入Rust后端库
use flux_backend::{AuditService, TauriAuditConfig, TimePointService, TimePointQueryRequest, TimePointQueryResult};

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
    pub last_full_query: Mutex<Option<(String, String)>>, // (file_path, algorithm) 用于缓存判定
    pub time_point_service: Mutex<Option<flux_backend::services::TimePointService>>, // 时点查询服务（支持缓存）
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

// 移除了Python备用的时点查询函数 - 现在完全使用Rust后端

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
    
    // 检查工作目录权限 - 使用用户数据目录而不是当前工作目录
    let work_dir = if is_dev_mode {
        // 开发模式：使用当前项目目录
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    } else {
        // 生产模式：使用用户文档目录，避免权限问题
        dirs::document_dir()
            .map(|d| d.join("FLUX Analysis System"))
            .unwrap_or_else(|| {
                // 如果无法获取文档目录，使用用户主目录
                dirs::home_dir()
                    .map(|h| h.join("FLUX Analysis System"))
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
            })
    };
    
    let work_dir_writable = match std::fs::create_dir_all(&work_dir.join("temp_analysis_results")) {
        Ok(_) => true,
        Err(e) => {
            println!("Cannot create temp_analysis_results directory: {}", e);
            println!("Working directory: {}", work_dir.display());
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
        
        // Rust后端分析停止（无需终止外部进程）
        process_status.output_log.push(format!("[{}] ⚡ Rust分析已停止", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));
        
        // 重置状态
        process_status.running = false;
        process_status.command = None;
        process_status.progress = Some(0.0);  // 重置进度条
        process_status.process_id = None;     // 清除进程ID
        process_status.message = Some("分析已停止".to_string());
        
        info!("Rust backend analysis stopped by user");
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

// 移除了Python相关的辅助函数 - 现在完全使用Rust后端

// 辅助函数：生成唯一ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("id_{}", timestamp)
}

// 移除了Python输出解析相关函数 - 现在完全使用Rust后端

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
        last_full_query: Mutex::new(None), // 初始化缓存状态
        time_point_service: Mutex::new(None), // 时点查询服务延迟初始化
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

/// 获取文件统计信息
#[tauri::command]
async fn get_file_stats(file_path: String) -> Result<serde_json::Value, String> {
    use std::fs;
    use serde_json::json;
    
    match fs::metadata(&file_path) {
        Ok(metadata) => {
            let modified = metadata.modified()
                .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0);
                
            Ok(json!({
                "size": metadata.len(),
                "modified": modified * 1000, // 转换为毫秒
                "is_dir": metadata.is_dir(),
                "is_file": metadata.is_file()
            }))
        }
        Err(e) => Err(format!("无法获取文件信息: {}", e))
    }
}

/// 获取应用目录路径
#[tauri::command]
async fn get_app_directory() -> Result<String, String> {
    use std::env;
    
    // 获取当前工作目录
    match env::current_dir() {
        Ok(dir) => Ok(dir.to_string_lossy().to_string()),
        Err(e) => Err(format!("无法获取应用目录: {}", e))
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
            commands::clear_query_cache,
            commands::export_fund_pools_excel,  // 新增Excel导出命令
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
            open_file,  // 新增打开文件命令
            get_file_stats,
            get_app_directory
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