//! 服务层模块
//! 
//! 提供高层业务逻辑和服务接口

pub mod audit_service;
pub mod integration_processor;
pub mod investment_service;

// 重新导出主要服务
pub use audit_service::*;
pub use integration_processor::*;
pub use investment_service::*;