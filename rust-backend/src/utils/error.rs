//! # 错误处理模块
//! 
//! 定义系统统一的错误类型和处理机制。

use thiserror::Error;

/// 审计系统错误类型
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Excel处理错误: {message}")]
    Excel { message: String },

    #[error("数据验证错误: {message}")]
    Validation { message: String },

    #[error("算法执行错误: {message}")]
    Algorithm { message: String },

    #[error("配置错误: {message}")]
    Config { message: String },

    #[error("业务逻辑错误: {message}")]
    Business { message: String },

    #[error("系统错误: {message}")]
    System { message: String },
}

/// 统一结果类型
pub type Result<T> = std::result::Result<T, AuditError>;

impl AuditError {
    /// 创建Excel错误
    pub fn excel(message: impl Into<String>) -> Self {
        Self::Excel {
            message: message.into(),
        }
    }

    /// 创建验证错误
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// 创建算法错误
    pub fn algorithm(message: impl Into<String>) -> Self {
        Self::Algorithm {
            message: message.into(),
        }
    }

    /// 创建配置错误
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 创建业务错误
    pub fn business(message: impl Into<String>) -> Self {
        Self::Business {
            message: message.into(),
        }
    }

    /// 创建系统错误
    pub fn system(message: impl Into<String>) -> Self {
        Self::System {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AuditError::excel("测试错误");
        assert!(error.to_string().contains("Excel处理错误"));

        let error = AuditError::validation("数据无效");
        assert!(error.to_string().contains("数据验证错误"));
    }
}
