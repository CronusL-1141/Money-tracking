//! # 涉案资金追踪分析系统 Rust 后端
//! 
//! 高性能的金融数据分析引擎，提供FIFO和余额法两种算法追踪资金流向。
//! 
//! ## 主要特性
//! 
//! - 🚀 高性能：相比Python版本提升10-15倍处理速度
//! - 🔒 内存安全：Rust编译时保证内存和线程安全
//! - 📊 双算法：支持FIFO先进先出和余额法两种追踪算法
//! - 💰 投资产品：完整的场外资金池管理和盈亏分析
//! - 📈 实时处理：支持大数据量的实时进度报告
//! - 🔧 易集成：为Tauri前端提供完整的后端服务
//! 
//! ## 架构设计
//! 
//! ```text
//! audit_backend
//! ├── algorithms/     # 核心算法引擎
//! ├── services/       # 业务服务层
//! ├── data_structures/# 核心数据结构
//! ├── utils/          # 工具和配置
//! └── optimizations/  # 性能优化模块
//! ```
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use audit_backend::services::AuditService;
//! use audit_backend::algorithms::AlgorithmType;
//! 
//! async fn example() -> anyhow::Result<()> {
//!     let mut service = AuditService::new(AlgorithmType::Fifo)?;
//!     let result = service.analyze_file("data.xlsx").await?;
//!     println!("处理了 {} 条记录", result.total_processed);
//!     Ok(())
//! }
//! ```

pub mod algorithms;
pub mod data_structures;
pub mod services;
pub mod utils;
pub mod optimizations;

// 重新导出主要类型供外部使用
pub use algorithms::{AlgorithmType, TrackerEngine, FifoTracker, BalanceMethodTracker};
pub use data_structures::{Transaction, FundPool, ProcessResult};
pub use services::{AuditService, TimePointQueryService, AuditResult};
pub use utils::{Config, AuditError};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 获取库信息
pub fn get_library_info() -> LibraryInfo {
    LibraryInfo {
        name: env!("CARGO_PKG_NAME"),
        version: VERSION,
        description: env!("CARGO_PKG_DESCRIPTION"),
        authors: env!("CARGO_PKG_AUTHORS"),
    }
}

/// 库信息结构
#[derive(Debug, Clone)]
pub struct LibraryInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub authors: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_info() {
        let info = get_library_info();
        assert_eq!(info.name, "audit-backend");
        assert!(!info.version.is_empty());
    }
}