//! Tauri GUI兼容的数据类型定义
//! 
//! 对应前端TypeScript中定义的接口类型，确保序列化兼容性

use serde::{Deserialize, Serialize};
use crate::data_models::AuditSummary;

/// 审计配置（与前端AuditConfig对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriAuditConfig {
    pub algorithm: String,
    pub input_file: String,
    pub output_file: Option<String>,
}

/// 审计结果（与前端AuditResult对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriAuditResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<TauriResultData>,
    pub output_files: Vec<String>,
}

/// 审计结果数据部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriResultData {
    pub summary: AuditSummary,
    pub transaction_count: usize,
    pub processing_time: f64,
    pub algorithm: String,
}

/// 进程状态（与前端ProcessStatus对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriProcessStatus {
    pub running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub output_log: Vec<String>,
}

/// 时点查询配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriTimePointQuery {
    pub file_path: String,
    pub row_number: usize,
    pub algorithm: String,
}

/// 时点查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriQueryResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_row: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_rows: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_time: Option<String>,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriFileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: String,
    pub exists: bool,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriAppConfig {
    pub default_algorithm: String,
    pub auto_export: bool,
    pub max_history: usize,
    pub language: String,
    pub theme: String,
}

impl TauriAuditResult {
    /// 创建成功的审计结果
    pub fn success(summary: AuditSummary, transaction_count: usize, processing_time: f64, algorithm: String, output_files: Vec<String>) -> Self {
        Self {
            success: true,
            message: format!("{}算法分析完成，处理 {} 条交易记录", algorithm, transaction_count),
            data: Some(TauriResultData {
                summary,
                transaction_count,
                processing_time,
                algorithm,
            }),
            output_files,
        }
    }
    
    /// 创建失败的审计结果
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            output_files: vec![],
        }
    }
}

impl TauriProcessStatus {
    /// 创建运行中状态
    pub fn running(progress: f64, message: String) -> Self {
        Self {
            running: true,
            command: Some("audit_analysis".to_string()),
            progress: Some(progress),
            message: Some(message),
            output_log: vec![],
        }
    }
    
    /// 创建空闲状态
    pub fn idle() -> Self {
        Self {
            running: false,
            command: None,
            progress: None,
            message: None,
            output_log: vec![],
        }
    }
    
    /// 添加日志消息
    pub fn with_log(mut self, log_message: String) -> Self {
        self.output_log.push(log_message);
        self
    }
}

impl Default for TauriAppConfig {
    fn default() -> Self {
        Self {
            default_algorithm: "FIFO".to_string(),
            auto_export: true,
            max_history: 100,
            language: "zh-CN".to_string(),
            theme: "light".to_string(),
        }
    }
}