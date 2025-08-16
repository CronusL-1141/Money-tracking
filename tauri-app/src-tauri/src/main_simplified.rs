// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::path::PathBuf;
use tauri::command;
use serde::{Deserialize, Serialize};

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

// Tauri命令：获取可用算法列表
#[command]
async fn get_algorithms() -> Result<Vec<String>, String> {
    Ok(vec!["FIFO".to_string(), "BALANCE_METHOD".to_string()])
}

// Tauri命令：运行审计分析
#[command]
async fn run_audit(config: AuditConfig) -> Result<AuditResult, String> {
    // 获取Python可执行文件路径
    let python_exe = find_python_executable();
    
    // 构建Python脚本路径
    let project_root = get_project_root()?;
    let script_path = project_root.join("src").join("main_new.py");
    
    let mut cmd = Command::new(&python_exe);
    cmd.current_dir(&project_root)
        .arg(script_path)
        .arg("--algorithm")
        .arg(&config.algorithm)
        .arg("--input")
        .arg(&config.input_file);
    
    if let Some(output) = &config.output_file {
        cmd.arg("--output").arg(output);
    }
    
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if output.status.success() {
                Ok(AuditResult {
                    success: true,
                    message: stdout.to_string(),
                    data: None,
                    output_files: vec![], 
                })
            } else {
                Ok(AuditResult {
                    success: false,
                    message: format!("Error: {}", stderr),
                    data: None,
                    output_files: vec![],
                })
            }
        }
        Err(e) => Err(format!("Failed to execute: {}", e)),
    }
}

// Tauri命令：时点查询
#[command]
async fn time_point_query(query: TimePointQuery) -> Result<QueryResult, String> {
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
    
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if output.status.success() {
                Ok(QueryResult {
                    success: true,
                    data: None,
                    message: stdout.to_string(),
                })
            } else {
                Ok(QueryResult {
                    success: false,
                    data: None,
                    message: format!("Query failed: {}", stderr),
                })
            }
        }
        Err(e) => Err(format!("Failed to execute: {}", e)),
    }
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

// Tauri命令：选择文件
#[command]
async fn select_file(
    title: String,
    filters: Vec<(String, Vec<String>)>,
) -> Result<Option<String>, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;
    
    let mut dialog = FileDialogBuilder::new().set_title(&title);
    
    for (name, extensions) in filters {
        dialog = dialog.add_filter(&name, &extensions);
    }
    
    match dialog.pick_file() {
        Some(path) => Ok(Some(path.to_string_lossy().to_string())),
        None => Ok(None),
    }
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

fn main() {
    // 初始化日志
    env_logger::init();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_algorithms,
            run_audit,
            time_point_query,
            check_python_env,
            select_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}