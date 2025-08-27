//! FIFO资金追踪审计系统 Rust后端
//! 
//! 这是一个高性能的资金追踪和审计分析系统，支持FIFO和差额计算法两种算法。
//! 
//! ## 主要功能
//! 
//! - 🏦 **双算法支持**: FIFO先进先出算法和差额计算法
//! - 📊 **Excel处理**: 高效读写Excel文件
//! - 🔍 **数据验证**: 完整性和准确性验证
//! - 📈 **实时分析**: 流式数据处理和进度反馈
//! - 🚀 **高性能**: 编译优化的Rust实现
//! 
//! ## 架构设计
//! 
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │    Services     │────│    Algorithms   │────│     Models      │
//! │    服务层       │    │     算法层      │    │    数据模型     │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!           │                        │                        │
//!           │                        │                        │
//!           ▼                        ▼                        ▼
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │     Utils       │    │   Interfaces    │    │     Errors      │
//! │    工具层       │    │    接口层       │    │    错误处理     │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```

// #![deny(missing_docs)]  // 暂时禁用，待完成文档后再启用
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod algorithms;
pub mod services;
pub mod data_models;
pub mod utils;
pub mod errors;

// 重新导出核心类型
pub use algorithms::*;
pub use services::*;
pub use data_models::*;
pub use errors::*;
pub use utils::*;

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 库描述
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");