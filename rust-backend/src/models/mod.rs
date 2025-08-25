//! 数据模型模块
//! 
//! 定义系统中所有核心数据结构，包括交易记录、审计摘要、资金池记录等。

pub mod transaction;
pub mod audit_summary;
pub mod fund_pool;
pub mod config;
pub mod investment;

// 重新导出主要类型
pub use transaction::*;
pub use audit_summary::*;
pub use fund_pool::*;
pub use config::*;
pub use investment::*;