//! 算法模块
//! 
//! 实现各种资金追踪算法，包括FIFO和差额计算法。

pub mod fifo_tracker;
pub mod balance_method_tracker;
pub mod behavior_analyzer;
pub mod flow_analyzer;
pub mod tracker_factory;

// 重新导出主要类型
pub use fifo_tracker::*;
pub use balance_method_tracker::*;
pub use behavior_analyzer::*;
pub use flow_analyzer::*;
pub use tracker_factory::*;