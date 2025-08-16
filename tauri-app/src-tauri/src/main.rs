// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs;
use std::io::{BufRead, BufReader};
use tauri::command;
use tauri::State;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use log::{info, warn, error};
use regex::Regex;

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
}

// 应用状态管理
pub struct AppState {
    pub query_history: Mutex<Vec<QueryHistory>>,
    pub current_process: Mutex<ProcessStatus>,
    pub app_config: Mutex<AppConfig>,
}

// Tauri命令：获取可用算法列表
#[command]
async fn get_algorithms() -> Result<Vec<String>, String> {
    Ok(vec!["FIFO".to_string(), "BALANCE_METHOD".to_string()])
}

// Tauri命令：运行审计分析
#[command]
async fn run_audit(config: AuditConfig, state: State<'_, AppState>) -> Result<AuditResult, String> {
    info!("Starting audit with algorithm: {}, input: {}", config.algorithm, config.input_file);
    
    // 步骤1: 初始化
    {
        let mut process_status = state.current_process.lock().await;
        *process_status = ProcessStatus {
            running: true,
            command: Some(format!("audit_{}", config.algorithm)),
            progress: Some(0.0),
            message: Some("初始化分析环境...".to_string()),
            output_log: Vec::new(),
        };
    }
    
    // 步骤2: 检查文件
    {
        let mut process_status = state.current_process.lock().await;
        process_status.progress = Some(10.0);
        process_status.message = Some("检查输入文件...".to_string());
    }
    
    // 获取Python可执行文件路径
    let python_exe = find_python_executable();
    
    // 构建Python脚本路径
    let project_root = get_project_root()?;
    let script_path = project_root.join("src").join("main_new.py");
    
    // 步骤3: 准备命令
    {
        let mut process_status = state.current_process.lock().await;
        process_status.progress = Some(20.0);
        process_status.message = Some("准备Python分析命令...".to_string());
    }
    
    let mut cmd = Command::new(&python_exe);
    cmd.current_dir(&project_root)
        .arg(script_path)
        .arg("--algorithm")
        .arg(&config.algorithm)
        .arg("--input")
        .arg(&config.input_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    if let Some(output) = &config.output_file {
        cmd.arg("--output").arg(output);
    }
    
    // 步骤4: 开始执行
    {
        let mut process_status = state.current_process.lock().await;
        process_status.progress = Some(30.0);
        process_status.message = Some("启动Python分析进程...".to_string());
    }
    
    let result = match cmd.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);
            
            let mut output_lines = Vec::new();
            let mut final_progress = 30.0;
            
            // 实时读取输出
            for line in reader.lines() {
                match line {
                    Ok(line_str) => {
                        output_lines.push(line_str.clone());
                        
                        // 解析进度信息
                        let progress = parse_progress_from_line(&line_str);
                        if progress > final_progress {
                            final_progress = progress;
                        }
                        
                        // 更新进程状态
                        {
                            let mut process_status = state.current_process.lock().await;
                            process_status.progress = Some(final_progress);
                            process_status.message = Some(extract_message_from_line(&line_str));
                            process_status.output_log.push(format!("[{}] {}", 
                                chrono::Utc::now().format("%H:%M:%S"), 
                                line_str
                            ));
                            
                            // 限制日志长度，避免内存占用过多
                            if process_status.output_log.len() > 1000 {
                                process_status.output_log.drain(0..100);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading line: {}", e);
                        break;
                    }
                }
            }
            
            // 等待进程结束
            let exit_status = child.wait().unwrap();
            let full_output = output_lines.join("\n");
            
            if exit_status.success() {
                info!("Audit completed successfully");
                
                // 步骤5: 解析输出文件
                {
                    let mut process_status = state.current_process.lock().await;
                    process_status.progress = Some(100.0);
                    process_status.message = Some("分析完成".to_string());
                }
                
                // 尝试解析输出文件列表
                let output_files = extract_output_files(&full_output);
                
                AuditResult {
                    success: true,
                    message: full_output,
                    data: None,
                    output_files,
                }
            } else {
                error!("Python process failed with exit code: {:?}", exit_status.code());
                AuditResult {
                    success: false,
                    message: format!("Python进程失败，退出代码: {:?}", exit_status.code()),
                    data: None,
                    output_files: vec![],
                }
            }
        }
        Err(e) => {
            error!("Failed to execute audit: {}", e);
            return Err(format!("执行失败: {}", e));
        }
    };
    
    // 重置进程状态
    {
        let mut process_status = state.current_process.lock().await;
        *process_status = ProcessStatus {
            running: false,
            command: None,
            progress: None,
            message: None,
            output_log: Vec::new(),
        };
    }
    
    Ok(result)
}

// Tauri命令：时点查询
#[command]
async fn time_point_query(query: TimePointQuery, state: State<'_, AppState>) -> Result<QueryResult, String> {
    info!("Time point query: file={}, row={}, algorithm={}", query.file_path, query.row_number, query.algorithm);
    
    let python_exe = find_python_executable();
    let project_root = get_project_root()?;
    let script_path = project_root.join("src").join("services").join("query_cli.py");
    
    let mut cmd = Command::new(&python_exe);
    cmd.current_dir(&project_root)
        .arg(script_path)
        .arg("--file")
        .arg(&query.file_path)
        .arg("--row")
        .arg(&query.row_number.to_string())
        .arg("--algorithm")
        .arg(&query.algorithm);
    
    let result = match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if output.status.success() {
                info!("Time point query completed successfully");
                QueryResult {
                    success: true,
                    data: parse_query_output(&stdout),
                    message: stdout.to_string(),
                }
            } else {
                warn!("Time point query failed: {}", stderr);
                QueryResult {
                    success: false,
                    data: None,
                    message: format!("查询失败: {}", stderr),
                }
            }
        }
        Err(e) => {
            error!("Failed to execute time point query: {}", e);
            return Err(format!("执行失败: {}", e));
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

// Tauri命令：检查Python环境
#[command]
async fn check_python_env() -> Result<serde_json::Value, String> {
    let python_exe = find_python_executable();
    let mut cmd = Command::new(&python_exe);
    cmd.arg("--version");
    
    match cmd.output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            let project_root = get_project_root().unwrap_or_else(|_| PathBuf::from("."));
            
            Ok(serde_json::json!({
                "python_available": output.status.success(),
                "python_version": version.trim(),
                "python_path": python_exe.to_string_lossy(),
                "project_root": project_root.to_string_lossy()
            }))
        }
        Err(e) => Err(format!("Failed to check Python environment: {}", e)),
    }
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
        if path.join("src").join("main_new.py").exists() {
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
    // 尝试从输出中提取JSON数据
    for line in output.lines() {
        if line.trim().starts_with('{') && line.trim().ends_with('}') {
            if let Ok(json) = serde_json::from_str(line.trim()) {
                return Some(json);
            }
        }
    }
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
        }),
        app_config: Mutex::new(create_default_config()),
    }
}

// 辅助函数：从输出行解析进度百分比
fn parse_progress_from_line(line: &str) -> f32 {
    // 1. 查找百分比模式，如 "进度: 45%" 或 "[45%]"
    if let Ok(re) = Regex::new(r"(\d+)%") {
        if let Some(captures) = re.captures(line) {
            if let Some(percent_str) = captures.get(1) {
                if let Ok(percent) = percent_str.as_str().parse::<f32>() {
                    return percent.min(100.0);
                }
            }
        }
    }
    
    // 2. 查找"处理进度: X/Y"模式（核心进度信息）
    if let Ok(re) = Regex::new(r"处理进度:\s*(\d+)/(\d+)") {
        if let Some(captures) = re.captures(line) {
            if let (Some(current_str), Some(total_str)) = (captures.get(1), captures.get(2)) {
                if let (Ok(current), Ok(total)) = (
                    current_str.as_str().parse::<f32>(), 
                    total_str.as_str().parse::<f32>()
                ) {
                    if total > 0.0 {
                        // 数据处理阶段占整个分析的60%-90%
                        let data_progress = (current / total) * 100.0;
                        return 60.0 + (data_progress * 0.3); // 60% + (进度 * 30%)
                    }
                }
            }
        }
    }
    
    // 3. 检查特定关键词并映射到进度阶段
    if line.contains("数据预处理") || line.contains("预处理财务数据") {
        return 40.0;
    } else if line.contains("流水完整性验证") || line.contains("完整性验证") {
        return 45.0;
    } else if line.contains("数据验证") {
        return 50.0;
    } else if line.contains("计算初始余额") {
        return 55.0;
    } else if line.contains("开始") && (line.contains("FIFO") || line.contains("差额计算")) {
        return 60.0; // 开始实际的追踪分析
    } else if line.contains("资金追踪完成") || line.contains("追踪完成") {
        return 90.0;
    } else if line.contains("保存结果") {
        return 95.0;
    } else if line.contains("生成投资产品交易记录") {
        return 97.0;
    } else if line.contains("流水数据处理完成") || line.contains("分析完成") {
        return 100.0;
    }
    
    0.0 // 默认返回0，表示没有进度更新
}

// 辅助函数：从输出行提取显示消息
fn extract_message_from_line(line: &str) -> String {
    // 移除时间戳和日志级别前缀
    let cleaned = if let Ok(re) = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d+ - \w+ - ") {
        re.replace(line, "").to_string()
    } else {
        line.to_string()
    };
    
    // 如果行太长，截断显示
    if cleaned.len() > 80 {
        format!("{}...", &cleaned[..77])
    } else {
        cleaned
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
            time_point_query,
            check_python_env,
            get_query_history,
            clear_query_history,
            delete_query_history_item,
            get_process_status,
            get_app_config,
            update_app_config,
            get_file_info,
            export_query_result,
            validate_file_path
        ])
        .setup(|_app| {
            info!("Application setup completed");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}