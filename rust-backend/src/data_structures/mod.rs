//! # 数据结构模块
//! 
//! 定义核心数据结构，包括交易记录、资金池、处理结果等。
//! 
//! ## 主要数据结构
//! 
//! - [`Transaction`]: 交易记录，包含完整的交易信息
//! - [`FundPool`]: 资金池，管理投资产品相关信息
//! - [`ProcessResult`]: 处理结果，包含算法执行后的状态
//! - [`StateSummary`]: 状态摘要，提供当前系统状态的概览

pub mod transaction;
pub mod fund_pool;
pub mod fifo_queue;
pub mod statistics;
pub mod process_result;

// 重新导出主要类型
pub use transaction::Transaction;
pub use fund_pool::{FundPool, FundPoolDetails, FundPoolRecord};
pub use fifo_queue::{FifoQueue, FifoEntry, OutflowResult};
pub use statistics::{TrackerStatistics, ProcessingStats, StateSummary};
pub use process_result::{ProcessResult, BehaviorAnalysis, TransactionType};

use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

/// 通用的金额类型，支持序列化
pub type Amount = f64;

/// 通用的比例类型（0.0 到 1.0）
pub type Ratio = f64;

/// 资金属性枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FundAttribute {
    /// 个人资金
    Personal,
    /// 公司资金
    Company,
    /// 投资产品
    Investment(String),
    /// 其他
    Other(String),
}

impl FundAttribute {
    /// 从字符串解析资金属性
    pub fn from_str(s: &str) -> Self {
        let s = s.trim();
        
        if crate::utils::is_personal_fund(s) {
            Self::Personal
        } else if crate::utils::is_company_fund(s) {
            Self::Company
        } else if crate::utils::is_investment_product(s) {
            Self::Investment(s.to_string())
        } else {
            Self::Other(s.to_string())
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            Self::Personal => "个人".to_string(),
            Self::Company => "公司".to_string(),
            Self::Investment(name) => name.clone(),
            Self::Other(name) => name.clone(),
        }
    }

    /// 判断是否为个人资金
    pub fn is_personal(&self) -> bool {
        matches!(self, Self::Personal)
    }

    /// 判断是否为公司资金
    pub fn is_company(&self) -> bool {
        matches!(self, Self::Company)
    }

    /// 判断是否为投资产品
    pub fn is_investment(&self) -> bool {
        matches!(self, Self::Investment(_))
    }
}

/// 行为性质枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorType {
    /// 正常交易
    Normal,
    /// 挪用
    Misappropriation,
    /// 垫付
    AdvancePayment,
    /// 归还
    Repayment,
    /// 投资
    Investment,
    /// 赎回
    Redemption,
    /// 其他
    Other(String),
}

impl BehaviorType {
    /// 转换为中文字符串
    pub fn to_chinese(&self) -> &str {
        match self {
            Self::Normal => "正常",
            Self::Misappropriation => "挪用",
            Self::AdvancePayment => "垫付",
            Self::Repayment => "归还",
            Self::Investment => "投资",
            Self::Redemption => "赎回",
            Self::Other(s) => s,
        }
    }

    /// 从字符串解析行为性质
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "正常" => Self::Normal,
            "挪用" => Self::Misappropriation,
            "垫付" => Self::AdvancePayment,
            "归还" => Self::Repayment,
            "投资" => Self::Investment,
            "赎回" => Self::Redemption,
            other => Self::Other(other.to_string()),
        }
    }
}

/// 通用时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

impl TimeRange {
    /// 创建新的时间范围
    pub fn new(start: NaiveDateTime, end: NaiveDateTime) -> Self {
        Self { start, end }
    }

    /// 检查给定时间是否在范围内
    pub fn contains(&self, time: &NaiveDateTime) -> bool {
        *time >= self.start && *time <= self.end
    }

    /// 获取时间范围的持续时间（秒）
    pub fn duration_seconds(&self) -> i64 {
        (self.end - self.start).num_seconds()
    }
}

/// 数据验证错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    #[error("无效的金额: {amount}")]
    InvalidAmount { amount: f64 },

    #[error("无效的日期: {date}")]
    InvalidDate { date: String },

    #[error("无效的资金属性: {attribute}")]
    InvalidFundAttribute { attribute: String },

    #[error("数据缺失: {field}")]
    MissingField { field: String },

    #[error("数据不一致: {message}")]
    InconsistentData { message: String },
}

/// 数据验证特征
pub trait Validate {
    /// 验证数据有效性
    fn validate(&self) -> Result<(), ValidationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fund_attribute_parsing() {
        assert_eq!(FundAttribute::from_str("个人"), FundAttribute::Personal);
        assert_eq!(FundAttribute::from_str("公司"), FundAttribute::Company);
        
        if let FundAttribute::Investment(name) = FundAttribute::from_str("理财-产品A") {
            assert!(name.contains("理财"));
        } else {
            panic!("Expected Investment variant");
        }
    }

    #[test]
    fn test_behavior_type_conversion() {
        let behavior = BehaviorType::Misappropriation;
        assert_eq!(behavior.to_chinese(), "挪用");
        
        let parsed = BehaviorType::from_str("挪用");
        assert_eq!(parsed, BehaviorType::Misappropriation);
    }

    #[test]
    fn test_time_range() {
        use chrono::NaiveDate;
        
        let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let end = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap().and_hms_opt(23, 59, 59).unwrap();
        let range = TimeRange::new(start, end);
        
        let test_time = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap().and_hms_opt(12, 0, 0).unwrap();
        assert!(range.contains(&test_time));
        
        let outside_time = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        assert!(!range.contains(&outside_time));
    }
}
