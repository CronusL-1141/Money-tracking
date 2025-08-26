//! 算法模块
//! 
//! 实现各种资金追踪算法，包括FIFO和差额计算法。
//! 使用共享架构避免代码重复

pub mod shared;
pub mod fifo_tracker;
pub mod balance_method_tracker;

// 重新导出主要类型
pub use fifo_tracker::*;
pub use balance_method_tracker::*;

// 重新导出共享组件
pub use shared::*;