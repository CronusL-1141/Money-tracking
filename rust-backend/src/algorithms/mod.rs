//! # 算法引擎模块
//! 
//! 提供核心的资金追踪算法实现，包括FIFO和余额法两种算法。
//! 
//! ## 算法对比
//! 
//! | 特性 | FIFO算法 | 余额法 |
//! |------|----------|--------|
//! | **逻辑** | 先进先出队列 | 余额优先扣除 |
//! | **个人支出** | 按队列顺序 | 个人余额优先 |
//! | **公司支出** | 按队列顺序 | 公司余额优先 |
//! | **挪用计算** | 队列追溯 | 直接计算 |
//! | **性能** | 复杂 O(n) | 简单 O(1) |
//! | **精确度** | 高 | 中 |

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod fifo;
pub mod balance_method;
pub mod behavior_analyzer;
pub mod factory;

// 重新导出主要类型
pub use fifo::FifoTracker;
pub use balance_method::BalanceMethodTracker;
pub use behavior_analyzer::BehaviorAnalyzer;
pub use factory::TrackerFactory;

/// 算法类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgorithmType {
    /// FIFO先进先出算法
    #[serde(rename = "FIFO")]
    Fifo,
    /// 余额法算法
    #[serde(rename = "BALANCE_METHOD")]
    BalanceMethod,
}

impl fmt::Display for AlgorithmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fifo => write!(f, "FIFO"),
            Self::BalanceMethod => write!(f, "BALANCE_METHOD"),
        }
    }
}

impl AlgorithmType {
    /// 获取算法的中文名称
    pub fn chinese_name(&self) -> &'static str {
        match self {
            Self::Fifo => "FIFO先进先出法",
            Self::BalanceMethod => "余额优先扣除法",
        }
    }

    /// 获取算法描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Fifo => "按照资金流入的先后顺序进行追踪，精确反映资金流动轨迹",
            Self::BalanceMethod => "优先使用个人余额，简化计算逻辑，提高处理效率",
        }
    }

    /// 从字符串解析算法类型
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "FIFO" => Ok(Self::Fifo),
            "BALANCE_METHOD" => Ok(Self::BalanceMethod),
            _ => Err(format!("未知的算法类型: {}", s)),
        }
    }

    /// 获取所有可用算法
    pub fn all() -> Vec<Self> {
        vec![Self::Fifo, Self::BalanceMethod]
    }
}

/// 追踪器引擎抽象接口
#[async_trait]
pub trait TrackerEngine: Send + Sync {
    /// 初始化追踪器
    async fn initialize(&mut self, initial_balance: f64, balance_type: &str) -> crate::utils::Result<()>;

    /// 处理单笔交易
    async fn process_transaction(
        &mut self, 
        transaction: &crate::data_structures::Transaction
    ) -> crate::utils::Result<crate::data_structures::ProcessResult>;

    /// 获取当前状态摘要
    async fn get_state_summary(&self) -> crate::utils::Result<crate::data_structures::StateSummary>;

    /// 获取可用的资金池列表
    async fn get_available_fund_pools(&self) -> crate::utils::Result<Vec<crate::data_structures::FundPool>>;

    /// 查询特定资金池的详细信息
    async fn query_fund_pool(
        &self, 
        pool_name: &str
    ) -> crate::utils::Result<crate::data_structures::FundPoolDetails>;

    /// 导出处理结果到Excel
    async fn export_results(&self, output_path: &str) -> crate::utils::Result<()>;

    /// 获取算法类型
    fn algorithm_type(&self) -> AlgorithmType;

    /// 获取处理统计信息
    fn get_statistics(&self) -> crate::data_structures::TrackerStatistics;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_type_display() {
        assert_eq!(AlgorithmType::Fifo.to_string(), "FIFO");
        assert_eq!(AlgorithmType::BalanceMethod.to_string(), "BALANCE_METHOD");
    }

    #[test]
    fn test_algorithm_type_from_str() {
        assert_eq!(AlgorithmType::from_str("FIFO").unwrap(), AlgorithmType::Fifo);
        assert_eq!(AlgorithmType::from_str("fifo").unwrap(), AlgorithmType::Fifo);
        assert_eq!(AlgorithmType::from_str("BALANCE_METHOD").unwrap(), AlgorithmType::BalanceMethod);
        assert!(AlgorithmType::from_str("unknown").is_err());
    }

    #[test]
    fn test_algorithm_descriptions() {
        for algo in AlgorithmType::all() {
            assert!(!algo.chinese_name().is_empty());
            assert!(!algo.description().is_empty());
        }
    }
}
