//! 接口定义模块
//! 
//! 定义系统中各种接口和trait，确保组件间的松耦合。

pub mod tracker;
pub mod analyzer;

// 重新导出主要接口
pub use tracker::*;
pub use analyzer::*;