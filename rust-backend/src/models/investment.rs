//! 投资产品相关数据模型
//! 
//! 定义投资产品资金池、交易记录等纯数据结构，不包含业务逻辑

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

/// 投资产品资金池
/// 
/// 存储单个投资产品的资金状态信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvestmentPool {
    /// 个人金额
    pub personal_amount: Decimal,
    
    /// 公司金额
    pub company_amount: Decimal,
    
    /// 总金额
    pub total_amount: Decimal,
    
    /// 累计申购
    pub total_purchase: Decimal,
    
    /// 累计赎回
    pub total_redemption: Decimal,
    
    /// 最新个人占比
    pub latest_personal_ratio: Decimal,
    
    /// 最新公司占比
    pub latest_company_ratio: Decimal,
    
    /// 历史盈利记录
    pub profit_history: Vec<ProfitRecord>,
    
    /// 累计已实现盈利
    pub total_realized_profit: Decimal,
}

impl InvestmentPool {
    /// 创建新的投资产品资金池
    pub fn new() -> Self {
        Self {
            personal_amount: Decimal::ZERO,
            company_amount: Decimal::ZERO,
            total_amount: Decimal::ZERO,
            total_purchase: Decimal::ZERO,
            total_redemption: Decimal::ZERO,
            latest_personal_ratio: Decimal::ZERO,
            latest_company_ratio: Decimal::ZERO,
            profit_history: Vec::new(),
            total_realized_profit: Decimal::ZERO,
        }
    }
    
    /// 检查是否有有效的占比记录
    pub fn has_valid_ratios(&self) -> bool {
        self.latest_personal_ratio > Decimal::ZERO || self.latest_company_ratio > Decimal::ZERO
    }
}

impl Default for InvestmentPool {
    fn default() -> Self {
        Self::new()
    }
}

/// 盈利记录
/// 
/// 记录资金池重置时的盈利信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfitRecord {
    /// 重置时间
    pub reset_time: String,
    
    /// 盈利金额
    pub profit_amount: Decimal,
    
    /// 描述信息
    pub description: String,
}

/// 投资产品交易记录
/// 
/// 记录投资产品的单笔申购或赎回交易
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvestmentTransactionRecord {
    /// 交易时间
    pub transaction_time: String,
    
    /// 资金池名称（产品编号）
    pub pool_name: String,
    
    /// 入金金额（申购）
    pub inflow: Decimal,
    
    /// 出金金额（赎回）
    pub outflow: Decimal,
    
    /// 当前总余额
    pub total_balance: Decimal,
    
    /// 个人余额
    pub personal_balance: Decimal,
    
    /// 公司余额
    pub company_balance: Decimal,
    
    /// 单笔资金占比
    pub single_fund_ratio: String,
    
    /// 总资金占比
    pub total_fund_ratio: String,
    
    /// 行为性质
    pub behavior_nature: String,
    
    /// 累计申购
    pub cumulative_purchase: Decimal,
    
    /// 累计赎回
    pub cumulative_redemption: Decimal,
}

impl InvestmentTransactionRecord {
    /// 创建新的交易记录
    pub fn new(
        transaction_time: String,
        pool_name: String,
        inflow: Decimal,
        outflow: Decimal,
        total_balance: Decimal,
        personal_balance: Decimal,
        company_balance: Decimal,
        single_fund_ratio: String,
        total_fund_ratio: String,
        behavior_nature: String,
        cumulative_purchase: Decimal,
        cumulative_redemption: Decimal,
    ) -> Self {
        Self {
            transaction_time,
            pool_name,
            inflow,
            outflow,
            total_balance,
            personal_balance,
            company_balance,
            single_fund_ratio,
            total_fund_ratio,
            behavior_nature,
            cumulative_purchase,
            cumulative_redemption,
        }
    }
    
    /// 获取净现金流
    pub fn net_cash_flow(&self) -> Decimal {
        self.inflow - self.outflow
    }
    
    /// 判断是否为申购操作
    pub fn is_purchase(&self) -> bool {
        self.inflow > Decimal::ZERO && self.outflow == Decimal::ZERO
    }
    
    /// 判断是否为赎回操作
    pub fn is_redemption(&self) -> bool {
        self.outflow > Decimal::ZERO && self.inflow == Decimal::ZERO
    }
    
    /// 计算盈亏情况
    pub fn calculate_profit_loss(&self) -> Decimal {
        self.cumulative_redemption - self.cumulative_purchase
    }
}

/// 赎回处理结果
/// 
/// 包含赎回操作的所有计算结果
#[derive(Debug, Clone, PartialEq)]
pub struct RedemptionResult {
    /// 个人返还金额
    pub personal_return: Decimal,
    
    /// 公司返还金额
    pub company_return: Decimal,
    
    /// 收益金额
    pub profit: Decimal,
    
    /// 归还的个人本金
    pub personal_principal_returned: Decimal,
    
    /// 归还的公司本金
    pub company_principal_returned: Decimal,
    
    /// 个人收益
    pub personal_profit: Decimal,
    
    /// 公司收益
    pub company_profit: Decimal,
    
    /// 行为性质描述
    pub behavior_description: String,
}

impl RedemptionResult {
    /// 创建新的赎回结果
    pub fn new(
        personal_return: Decimal,
        company_return: Decimal,
    ) -> Self {
        Self {
            personal_return,
            company_return,
            profit: Decimal::ZERO,
            personal_principal_returned: Decimal::ZERO,
            company_principal_returned: Decimal::ZERO,
            personal_profit: Decimal::ZERO,
            company_profit: Decimal::ZERO,
            behavior_description: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_investment_pool_creation() {
        let pool = InvestmentPool::new();
        assert_eq!(pool.total_amount, Decimal::ZERO);
        assert!(!pool.has_valid_ratios());
    }
    
    #[test]
    fn test_investment_transaction_record() {
        let record = InvestmentTransactionRecord::new(
            "2023-01-15 14:30:25".to_string(),
            "理财-SL001".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(50000),
            Decimal::from(30000),
            Decimal::from(20000),
            "个人:60%，公司:40%".to_string(),
            "个人:60%，公司:40%".to_string(),
            "申购".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
        );
        
        assert!(record.is_purchase());
        assert!(!record.is_redemption());
        assert_eq!(record.net_cash_flow(), Decimal::from(50000));
        assert_eq!(record.calculate_profit_loss(), Decimal::from(-50000));
    }
    
    #[test]
    fn test_redemption_result() {
        let result = RedemptionResult::new(
            Decimal::from(60000),
            Decimal::from(40000)
        );
        
        assert_eq!(result.personal_return, Decimal::from(60000));
        assert_eq!(result.company_return, Decimal::from(40000));
        assert_eq!(result.profit, Decimal::ZERO);
    }
}