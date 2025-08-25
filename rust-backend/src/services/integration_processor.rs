//! 集成处理器
//! 
//! 提供与Tauri前端的集成接口

use crate::services::AuditService;
use crate::models::AuditSummary;
use crate::errors::{AuditError, AuditResult};
use serde::{Deserialize, Serialize};
use log::{info, error};

/// 集成处理器
/// 
/// 提供与前端框架集成的高级接口
#[derive(Debug)]
pub struct IntegrationProcessor {
    audit_service: AuditService,
}

impl IntegrationProcessor {
    /// 创建新的集成处理器
    pub fn new() -> Self {
        Self {
            audit_service: AuditService::new(),
        }
    }
    
    /// 执行审计分析（前端接口）
    pub async fn run_audit_analysis(&mut self, request: AuditAnalysisRequest) -> AuditResult<AuditAnalysisResponse> {
        info!("收到审计分析请求: {:?}", request);
        
        let start_time = std::time::Instant::now();
        
        let summary = self.audit_service.analyze_financial_data(
            &request.algorithm,
            &request.input_file,
            request.output_file.as_ref(),
        ).await?;
        
        let processing_time = start_time.elapsed().as_secs_f64();
        
        Ok(AuditAnalysisResponse {
            success: true,
            summary,
            output_files: vec![format!("{}_资金追踪结果.xlsx", request.algorithm)],
            processing_time: Some(processing_time),
            message: Some("分析完成".to_string()),
        })
    }
    
    /// 时点查询（前端接口）
    pub async fn query_time_point(&mut self, request: TimePointQueryRequest) -> AuditResult<TimePointQueryResponse> {
        info!("收到时点查询请求: {:?}", request);
        
        let summary = self.audit_service.query_time_point(
            &request.file_path,
            request.row_number as usize,
            &request.algorithm,
        ).await?;
        
        Ok(TimePointQueryResponse {
            success: true,
            summary: Some(summary),
            message: Some("查询完成".to_string()),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        })
    }
    
    /// 获取算法列表（前端接口）
    pub fn get_algorithms(&self) -> Vec<AlgorithmInfo> {
        let info = self.audit_service.get_algorithms_info();
        
        info.into_iter().map(|(name, description)| {
            AlgorithmInfo {
                name: name.to_string(),
                description: description.to_string(),
            }
        }).collect()
    }
    
    /// 验证文件路径
    pub fn validate_file_path(&self, file_path: &str) -> bool {
        std::path::Path::new(file_path).exists()
    }
}

impl Default for IntegrationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// 审计分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAnalysisRequest {
    /// 算法类型
    pub algorithm: String,
    
    /// 输入文件路径
    pub input_file: String,
    
    /// 输出文件路径（可选）
    pub output_file: Option<String>,
}

/// 审计分析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAnalysisResponse {
    /// 是否成功
    pub success: bool,
    
    /// 审计摘要
    pub summary: AuditSummary,
    
    /// 输出文件列表
    pub output_files: Vec<String>,
    
    /// 处理时间（秒）
    pub processing_time: Option<f64>,
    
    /// 消息
    pub message: Option<String>,
}

/// 时点查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePointQueryRequest {
    /// 文件路径
    pub file_path: String,
    
    /// 行号
    pub row_number: u32,
    
    /// 算法类型
    pub algorithm: String,
}

/// 时点查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePointQueryResponse {
    /// 是否成功
    pub success: bool,
    
    /// 审计摘要
    pub summary: Option<AuditSummary>,
    
    /// 消息
    pub message: Option<String>,
    
    /// 时间戳
    pub timestamp: Option<String>,
}

/// 算法信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmInfo {
    /// 算法名称
    pub name: String,
    
    /// 算法描述
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integration_processor_creation() {
        let processor = IntegrationProcessor::new();
        let algorithms = processor.get_algorithms();
        
        assert!(!algorithms.is_empty());
        assert!(algorithms.iter().any(|a| a.name == "FIFO"));
        assert!(algorithms.iter().any(|a| a.name == "BALANCE_METHOD"));
    }
    
    #[test]
    fn test_algorithm_info_serialization() {
        let info = AlgorithmInfo {
            name: "FIFO".to_string(),
            description: "先进先出算法".to_string(),
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: AlgorithmInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.description, deserialized.description);
    }
}