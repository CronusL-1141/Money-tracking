//! 错误处理模块
//! 
//! 定义系统中所有可能出现的错误类型，提供统一的错误处理机制。

use thiserror::Error;

/// 审计系统的主要错误类型
#[derive(Error, Debug)]
pub enum AuditError {
    /// Excel文件处理错误
    #[error("Excel文件处理错误: {0}")]
    ExcelError(String),
    
    /// 数据验证错误
    #[error("数据验证失败: {0}")]
    ValidationError(String),
    
    /// 算法执行错误
    #[error("算法执行错误: {0}")]
    AlgorithmError(String),
    
    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    /// IO操作错误
    #[error("IO操作失败: {0}")]
    IoError(#[from] std::io::Error),
    
    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// 时间解析错误
    #[error("时间解析错误: {0}")]
    TimeParseError(String),
    
    /// 数值计算错误
    #[error("数值计算错误: {0}")]
    CalculationError(String),
    
    /// 资金池操作错误
    #[error("资金池操作错误: {0}")]
    FundPoolError(String),
    
    /// 追踪器初始化错误
    #[error("追踪器初始化错误: {0}")]
    TrackerInitError(String),
    
    /// 未支持的操作
    #[error("不支持的操作: {0}")]
    UnsupportedOperation(String),
    
    /// 内部系统错误
    #[error("内部系统错误: {0}")]
    InternalError(String),
}

/// 审计结果类型别名
pub type AuditResult<T> = Result<T, AuditError>;

impl AuditError {
    /// 创建Excel错误
    pub fn excel_error<S: Into<String>>(msg: S) -> Self {
        Self::ExcelError(msg.into())
    }
    
    /// 创建验证错误
    pub fn validation_error<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }
    
    /// 创建算法错误
    pub fn algorithm_error<S: Into<String>>(msg: S) -> Self {
        Self::AlgorithmError(msg.into())
    }
    
    /// 创建配置错误
    pub fn config_error<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(msg.into())
    }
    
    /// 创建时间解析错误
    pub fn time_parse_error<S: Into<String>>(msg: S) -> Self {
        Self::TimeParseError(msg.into())
    }
    
    /// 创建计算错误
    pub fn calculation_error<S: Into<String>>(msg: S) -> Self {
        Self::CalculationError(msg.into())
    }
    
    /// 创建资金池错误
    pub fn fund_pool_error<S: Into<String>>(msg: S) -> Self {
        Self::FundPoolError(msg.into())
    }
    
    /// 创建追踪器初始化错误
    pub fn tracker_init_error<S: Into<String>>(msg: S) -> Self {
        Self::TrackerInitError(msg.into())
    }
    
    /// 创建不支持的操作错误
    pub fn unsupported_operation<S: Into<String>>(msg: S) -> Self {
        Self::UnsupportedOperation(msg.into())
    }
    
    /// 创建内部错误
    pub fn internal_error<S: Into<String>>(msg: S) -> Self {
        Self::InternalError(msg.into())
    }
}