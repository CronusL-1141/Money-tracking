//! 算法共享组件模块
//! 
//! 包含FIFO和差额计算法算法的共享组件
//! 实现90%的共同功能，避免代码重复

pub mod tracker_base;
pub mod behavior_analyzer;
pub mod investment_pool;
pub mod fund_flow_common;
pub mod summary;

// 重新导出主要类型
pub use tracker_base::{TrackerBase, InvestmentPool, ProfitRecord, OffSiteRecord};
pub use behavior_analyzer::BehaviorAnalyzer;
pub use investment_pool::InvestmentPoolManager;
pub use fund_flow_common::FundFlowCommon;
pub use summary::SummaryGenerator;