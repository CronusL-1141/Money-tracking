//! # 资金池数据结构
//! 
//! 定义资金池相关的数据结构。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 资金池信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundPool {
    /// 资金池名称
    pub name: String,
    
    /// 创建时间
    pub created_at: NaiveDateTime,
    
    /// 总余额
    pub total_balance: f64,
    
    /// 是否活跃
    pub active: bool,
    
    /// 资金池类型
    pub pool_type: String,
}

/// 资金池详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundPoolDetails {
    /// 基础信息
    pub info: FundPool,
    
    /// 交易记录
    pub records: Vec<FundPoolRecord>,
    
    /// 统计信息
    pub statistics: FundPoolStatistics,
}

/// 资金池交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundPoolRecord {
    /// 交易时间
    pub transaction_time: NaiveDateTime,
    
    /// 入金金额
    pub amount_in: f64,
    
    /// 出金金额
    pub amount_out: f64,
    
    /// 总余额
    pub balance: f64,
    
    /// 单笔资金占比
    pub single_ratio: f64,
    
    /// 总资金占比
    pub total_ratio: f64,
}

/// 资金池统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundPoolStatistics {
    /// 总交易笔数
    pub total_transactions: usize,
    
    /// 总入金
    pub total_inflow: f64,
    
    /// 总出金
    pub total_outflow: f64,
    
    /// 当前余额
    pub current_balance: f64,
    
    /// 最高余额
    pub peak_balance: f64,
    
    /// 最低余额
    pub lowest_balance: f64,
}
