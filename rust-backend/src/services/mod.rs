//! 服务层模块
//! 
//! 提供高层业务逻辑和服务接口
//! 基于成功测试代码的简化实现模式

pub mod audit_service;

// 重新导出主要服务
pub use audit_service::*;