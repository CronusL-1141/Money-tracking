//! # 日志模块
//! 
//! 提供统一的日志记录功能。

use tracing::{info, warn, error, debug, trace};

/// 日志器
pub struct Logger;

impl Logger {
    /// 创建新的日志器
    pub fn new() -> Self {
        Self
    }

    /// 初始化日志系统
    pub fn init(&self) -> crate::utils::Result<()> {
        tracing_subscriber::fmt::init();
        Ok(())
    }

    /// 记录信息
    pub fn info(&self, message: &str) {
        info!("{}", message);
    }

    /// 记录警告
    pub fn warn(&self, message: &str) {
        warn!("{}", message);
    }

    /// 记录错误
    pub fn error(&self, message: &str) {
        error!("{}", message);
    }

    /// 记录调试信息
    pub fn debug(&self, message: &str) {
        debug!("{}", message);
    }

    /// 记录追踪信息
    pub fn trace(&self, message: &str) {
        trace!("{}", message);
    }
}
