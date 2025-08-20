//! # 服务模块
//! 
//! 提供高级业务服务，包括审计分析、时点查询、资金池查询等功能。
//! 
//! ## 服务架构
//! 
//! ```text
//! services/
//! ├── audit/           # 审计分析服务
//! ├── time_point/      # 时点查询服务
//! ├── fund_pool/       # 资金池查询服务
//! └── data_processing/ # 数据处理服务
//! ```
//! 
//! ## 主要服务
//! 
//! - [`AuditService`]: 核心审计分析服务，处理Excel文件并生成分析报告
//! - [`TimePointQueryService`]: 时点查询服务，查询特定行的系统状态
//! - [`FundPoolQueryService`]: 资金池查询服务，提供资金池详细信息
//! - [`DataProcessor`]: 数据处理服务，处理Excel读写和数据验证

pub mod audit;
pub mod time_point;
pub mod fund_pool;
pub mod data_processing;
pub mod realtime;

// 重新导出主要类型
pub use audit::{AuditService, AuditConfig, AuditResult};
pub use time_point::{TimePointQueryService, TimePointQuery, QueryResult};
pub use fund_pool::{FundPoolQueryService, FundPoolQueryResult};
pub use data_processing::{DataProcessor, ExcelProcessor};
pub use realtime::{RealtimeReporter, ProgressMessage, ProgressLevel};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// 最大处理行数
    pub max_rows: usize,
    
    /// 进度报告间隔
    pub progress_interval: usize,
    
    /// 是否启用缓存
    pub enable_cache: bool,
    
    /// 缓存过期时间（秒）
    pub cache_ttl: u64,
    
    /// 并发处理线程数
    pub worker_threads: usize,
    
    /// Excel处理配置
    pub excel_config: ExcelConfig,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_rows: 10_000_000, // 1000万行
            progress_interval: 1000,
            enable_cache: true,
            cache_ttl: 3600, // 1小时
            worker_threads: num_cpus::get(),
            excel_config: ExcelConfig::default(),
        }
    }
}

/// Excel处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelConfig {
    /// 是否自动检测编码
    pub auto_detect_encoding: bool,
    
    /// 默认工作表索引
    pub default_sheet_index: usize,
    
    /// 表头行数
    pub header_rows: usize,
    
    /// 最大读取行数
    pub max_read_rows: Option<usize>,
    
    /// 数据验证设置
    pub validation_config: ValidationConfig,
}

impl Default for ExcelConfig {
    fn default() -> Self {
        Self {
            auto_detect_encoding: true,
            default_sheet_index: 0,
            header_rows: 1,
            max_read_rows: None,
            validation_config: ValidationConfig::default(),
        }
    }
}

/// 数据验证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// 是否启用严格验证
    pub strict_validation: bool,
    
    /// 允许的空值比例（0.0-1.0）
    pub max_null_ratio: f32,
    
    /// 是否自动修复数据
    pub auto_fix: bool,
    
    /// 必需的列名
    pub required_columns: Vec<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_validation: false,
            max_null_ratio: 0.1, // 允许10%空值
            auto_fix: true,
            required_columns: vec![
                "交易日期".to_string(),
                "交易时间".to_string(),
                "交易收入金额".to_string(),
                "交易支出金额".to_string(),
                "余额".to_string(),
                "资金属性".to_string(),
            ],
        }
    }
}

/// 服务响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse<T> {
    /// 是否成功
    pub success: bool,
    
    /// 返回数据
    pub data: Option<T>,
    
    /// 错误消息
    pub error: Option<String>,
    
    /// 响应时间戳
    pub timestamp: DateTime<Utc>,
    
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    
    /// 额外元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl<T> ServiceResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            processing_time_ms: 0,
            metadata: HashMap::new(),
        }
    }

    /// 创建错误响应
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
            processing_time_ms: 0,
            metadata: HashMap::new(),
        }
    }

    /// 设置处理时间
    pub fn with_processing_time(mut self, time_ms: u64) -> Self {
        self.processing_time_ms = time_ms;
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 分页查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    /// 页码（从1开始）
    pub page: usize,
    
    /// 每页大小
    pub page_size: usize,
    
    /// 排序字段
    pub sort_by: Option<String>,
    
    /// 是否降序
    pub descending: bool,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 100,
            sort_by: None,
            descending: false,
        }
    }
}

impl PaginationParams {
    /// 计算跳过的记录数
    pub fn offset(&self) -> usize {
        (self.page.saturating_sub(1)) * self.page_size
    }

    /// 验证分页参数
    pub fn validate(&self) -> Result<(), String> {
        if self.page == 0 {
            return Err("页码必须大于0".to_string());
        }
        
        if self.page_size == 0 || self.page_size > 10000 {
            return Err("每页大小必须在1-10000之间".to_string());
        }
        
        Ok(())
    }
}

/// 分页查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    /// 数据项
    pub items: Vec<T>,
    
    /// 总数
    pub total: usize,
    
    /// 当前页
    pub page: usize,
    
    /// 每页大小
    pub page_size: usize,
    
    /// 总页数
    pub total_pages: usize,
    
    /// 是否有下一页
    pub has_next: bool,
    
    /// 是否有上一页
    pub has_previous: bool,
}

impl<T> PaginatedResult<T> {
    /// 创建分页结果
    pub fn new(items: Vec<T>, total: usize, params: &PaginationParams) -> Self {
        let total_pages = (total + params.page_size - 1) / params.page_size;
        
        Self {
            items,
            total,
            page: params.page,
            page_size: params.page_size,
            total_pages,
            has_next: params.page < total_pages,
            has_previous: params.page > 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert!(config.max_rows > 0);
        assert!(config.progress_interval > 0);
        assert!(config.worker_threads > 0);
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams {
            page: 2,
            page_size: 50,
            sort_by: None,
            descending: false,
        };
        
        assert_eq!(params.offset(), 50);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_paginated_result() {
        let items = vec![1, 2, 3, 4, 5];
        let params = PaginationParams {
            page: 1,
            page_size: 5,
            sort_by: None,
            descending: false,
        };
        
        let result = PaginatedResult::new(items, 20, &params);
        assert_eq!(result.total_pages, 4);
        assert!(result.has_next);
        assert!(!result.has_previous);
    }

    #[test]
    fn test_service_response() {
        let response = ServiceResponse::success("test data".to_string())
            .with_processing_time(100)
            .with_metadata("key".to_string(), serde_json::Value::String("value".to_string()));
        
        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.processing_time_ms, 100);
        assert!(response.metadata.contains_key("key"));
    }
}
