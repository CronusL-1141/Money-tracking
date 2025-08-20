//! # 处理结果数据结构
//! 
//! 定义算法处理结果的数据结构。

use serde::{Deserialize, Serialize};

/// 处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub success: bool,
    pub message: String,
    pub new_balance: f64,
    pub behavior_analysis: Option<BehaviorAnalysis>,
    pub fund_pools_affected: Vec<String>,
}

/// 行为分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorAnalysis {
    pub behavior_type: String,
    pub personal_ratio: f64,
    pub company_ratio: f64,
    pub misappropriation_amount: f64,
    pub advance_amount: f64,
}

/// 交易类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Income,
    Expense,
    Investment,
    Redemption,
    Transfer,
}

impl ProcessResult {
    pub fn success(message: String, new_balance: f64) -> Self {
        Self {
            success: true,
            message,
            new_balance,
            behavior_analysis: None,
            fund_pools_affected: Vec::new(),
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            new_balance: 0.0,
            behavior_analysis: None,
            fund_pools_affected: Vec::new(),
        }
    }
}
