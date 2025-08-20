//! # 交易记录数据结构
//! 
//! 定义核心的交易记录数据结构。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// 交易ID
    pub id: String,
    
    /// 交易日期
    pub date: NaiveDateTime,
    
    /// 交易收入金额
    pub amount_in: Option<f64>,
    
    /// 交易支出金额
    pub amount_out: Option<f64>,
    
    /// 账户余额
    pub balance: f64,
    
    /// 资金属性
    pub fund_attribute: String,
    
    /// 备注信息
    pub note: Option<String>,
}

impl Transaction {
    /// 创建新的交易记录
    pub fn new(
        id: String,
        date: NaiveDateTime,
        amount_in: Option<f64>,
        amount_out: Option<f64>,
        balance: f64,
        fund_attribute: String,
    ) -> Self {
        Self {
            id,
            date,
            amount_in,
            amount_out,
            balance,
            fund_attribute,
            note: None,
        }
    }

    /// 获取净流动金额
    pub fn net_amount(&self) -> f64 {
        let inflow = self.amount_in.unwrap_or(0.0);
        let outflow = self.amount_out.unwrap_or(0.0);
        inflow - outflow
    }

    /// 是否为资金流入
    pub fn is_inflow(&self) -> bool {
        self.amount_in.is_some() && self.amount_in.unwrap() > 0.0
    }

    /// 是否为资金流出
    pub fn is_outflow(&self) -> bool {
        self.amount_out.is_some() && self.amount_out.unwrap() > 0.0
    }
}
