//! 交易记录数据模型

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 交易记录
/// 
/// 表示单个交易的完整信息，包括原始数据和系统计算字段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// 交易日期
    pub transaction_date: NaiveDateTime,
    
    /// 交易时间（原始格式，如 "143025" 或 "14:30:25"）
    pub transaction_time: String,
    
    /// 交易收入金额（资金流入）
    pub income_amount: Decimal,
    
    /// 交易支出金额（资金流出）
    pub expense_amount: Decimal,
    
    /// 交易后余额
    pub balance: Decimal,
    
    /// 资金属性（如：个人应收、公司应付、理财-产品代码等）
    pub fund_attribute: String,
    
    // === 系统计算字段 ===
    
    /// 个人资金占比（0-1之间）
    pub personal_ratio: Option<Decimal>,
    
    /// 公司资金占比（0-1之间）
    pub company_ratio: Option<Decimal>,
    
    /// 行为性质（挪用、垫付、正常、投资等）
    pub behavior_nature: Option<String>,
    
    /// 截至当前交易的累计挪用金额
    pub cumulative_misappropriation: Option<Decimal>,
    
    /// 截至当前交易的累计垫付金额
    pub cumulative_advance: Option<Decimal>,
    
    /// 截至当前交易累计已归还公司本金
    pub cumulative_company_principal_returned: Option<Decimal>,
    
    /// 截至当前交易累计已归还个人本金
    pub cumulative_personal_principal_returned: Option<Decimal>,
    
    /// 截至当前交易总计个人应分配利润
    pub cumulative_personal_profit: Option<Decimal>,
    
    /// 截至当前交易总计公司应分配利润
    pub cumulative_company_profit: Option<Decimal>,
    
    /// 资金缺口
    pub funding_gap: Option<Decimal>,
    
    /// 当前个人余额
    pub personal_balance: Option<Decimal>,
    
    /// 当前公司余额
    pub company_balance: Option<Decimal>,
}

impl Transaction {
    /// 创建新的交易记录
    pub fn new(
        transaction_date: NaiveDateTime,
        transaction_time: String,
        income_amount: Decimal,
        expense_amount: Decimal,
        balance: Decimal,
        fund_attribute: String,
    ) -> Self {
        Self {
            transaction_date,
            transaction_time,
            income_amount,
            expense_amount,
            balance,
            fund_attribute,
            personal_ratio: None,
            company_ratio: None,
            behavior_nature: None,
            cumulative_misappropriation: None,
            cumulative_advance: None,
            cumulative_company_principal_returned: None,
            cumulative_personal_principal_returned: None,
            cumulative_personal_profit: None,
            cumulative_company_profit: None,
            funding_gap: None,
            personal_balance: None,
            company_balance: None,
        }
    }
    
    /// 获取交易净金额（收入-支出）
    pub fn net_amount(&self) -> Decimal {
        self.income_amount - self.expense_amount
    }
    
    /// 判断是否为收入交易
    pub fn is_income(&self) -> bool {
        self.income_amount > Decimal::ZERO
    }
    
    /// 判断是否为支出交易
    pub fn is_expense(&self) -> bool {
        self.expense_amount > Decimal::ZERO
    }
    
    /// 获取交易金额的绝对值
    pub fn abs_amount(&self) -> Decimal {
        if self.is_income() {
            self.income_amount
        } else {
            self.expense_amount
        }
    }
    
    /// 设置计算字段
    pub fn set_calculated_fields(
        &mut self,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        behavior_nature: String,
        cumulative_misappropriation: Decimal,
        cumulative_advance: Decimal,
        cumulative_company_principal_returned: Decimal,
        cumulative_personal_principal_returned: Decimal,
        cumulative_personal_profit: Decimal,
        cumulative_company_profit: Decimal,
        funding_gap: Decimal,
        personal_balance: Decimal,
        company_balance: Decimal,
    ) {
        self.personal_ratio = Some(personal_ratio);
        self.company_ratio = Some(company_ratio);
        self.behavior_nature = Some(behavior_nature);
        self.cumulative_misappropriation = Some(cumulative_misappropriation);
        self.cumulative_advance = Some(cumulative_advance);
        self.cumulative_company_principal_returned = Some(cumulative_company_principal_returned);
        self.cumulative_personal_principal_returned = Some(cumulative_personal_principal_returned);
        self.cumulative_personal_profit = Some(cumulative_personal_profit);
        self.cumulative_company_profit = Some(cumulative_company_profit);
        self.funding_gap = Some(funding_gap);
        self.personal_balance = Some(personal_balance);
        self.company_balance = Some(company_balance);
    }
    
    /// 格式化时间字段用于显示
    pub fn formatted_time(&self) -> String {
        // 如果是6位数字格式（如143025），转换为时分秒格式
        if self.transaction_time.len() == 6 && self.transaction_time.chars().all(|c| c.is_ascii_digit()) {
            let hours = &self.transaction_time[0..2];
            let minutes = &self.transaction_time[2..4];
            let seconds = &self.transaction_time[4..6];
            format!("{}:{}:{}", hours, minutes, seconds)
        } else {
            self.transaction_time.clone()
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} | 收入:{} 支出:{} 余额:{} | {}",
            self.transaction_date.format("%Y-%m-%d"),
            self.formatted_time(),
            self.income_amount,
            self.expense_amount,
            self.balance,
            self.fund_attribute
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_transaction_creation() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        
        let tx = Transaction::new(
            date,
            "143025".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(120000),
            "个人应收".to_string(),
        );
        
        assert_eq!(tx.income_amount, Decimal::from(50000));
        assert_eq!(tx.expense_amount, Decimal::ZERO);
        assert_eq!(tx.balance, Decimal::from(120000));
        assert_eq!(tx.fund_attribute, "个人应收");
        assert!(tx.is_income());
        assert!(!tx.is_expense());
        assert_eq!(tx.net_amount(), Decimal::from(50000));
        assert_eq!(tx.abs_amount(), Decimal::from(50000));
    }
    
    #[test]
    fn test_formatted_time() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        
        let tx = Transaction::new(
            date,
            "143025".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(120000),
            "个人应收".to_string(),
        );
        
        assert_eq!(tx.formatted_time(), "14:30:25");
        
        let tx2 = Transaction::new(
            date,
            "14:30:25".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(120000),
            "个人应收".to_string(),
        );
        
        assert_eq!(tx2.formatted_time(), "14:30:25");
    }
}