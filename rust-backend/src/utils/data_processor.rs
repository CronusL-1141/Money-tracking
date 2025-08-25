//! 数据处理工具

use crate::errors::AuditResult;
use crate::models::Transaction;
use log::{info, debug};
use rust_decimal::Decimal;

/// 数据处理器
/// 
/// 提供各种数据处理和转换功能
#[derive(Debug)]
pub struct DataProcessor;

impl DataProcessor {
    /// 清理和预处理交易数据
    pub fn preprocess_transactions(transactions: &mut Vec<Transaction>) -> AuditResult<()> {
        info!("📊 开始数据预处理");
        
        let original_count = transactions.len();
        
        // 移除无效记录
        transactions.retain(|tx| {
            // 检查基本有效性
            !tx.fund_attribute.trim().is_empty() && 
            (tx.income_amount > Decimal::ZERO || tx.expense_amount > Decimal::ZERO)
        });
        
        let removed_count = original_count - transactions.len();
        if removed_count > 0 {
            info!("🔧 移除无效记录: {} 条", removed_count);
        }
        
        // 按交易日期排序
        transactions.sort_by(|a, b| a.transaction_date.cmp(&b.transaction_date));
        debug!("📅 交易记录已按日期排序");
        
        info!("✅ 数据预处理完成，有效记录: {} 条", transactions.len());
        Ok(())
    }
    
    /// 计算数据统计信息
    pub fn calculate_statistics(transactions: &[Transaction]) -> DataStatistics {
        let mut stats = DataStatistics::new();
        
        stats.total_records = transactions.len();
        
        for tx in transactions {
            if tx.is_income() {
                stats.income_records += 1;
                stats.total_income += tx.income_amount;
            }
            
            if tx.is_expense() {
                stats.expense_records += 1;
                stats.total_expense += tx.expense_amount;
            }
            
            if tx.balance > stats.max_balance {
                stats.max_balance = tx.balance;
            }
            
            if tx.balance < stats.min_balance {
                stats.min_balance = tx.balance;
            }
        }
        
        if let (Some(first), Some(last)) = (transactions.first(), transactions.last()) {
            stats.date_range_start = Some(first.transaction_date);
            stats.date_range_end = Some(last.transaction_date);
        }
        
        stats
    }
}

/// 数据统计信息
#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub total_records: usize,
    pub income_records: usize,
    pub expense_records: usize,
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub max_balance: Decimal,
    pub min_balance: Decimal,
    pub date_range_start: Option<chrono::NaiveDateTime>,
    pub date_range_end: Option<chrono::NaiveDateTime>,
}

impl DataStatistics {
    fn new() -> Self {
        Self {
            total_records: 0,
            income_records: 0,
            expense_records: 0,
            total_income: Decimal::ZERO,
            total_expense: Decimal::ZERO,
            max_balance: Decimal::MIN,
            min_balance: Decimal::MAX,
            date_range_start: None,
            date_range_end: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Datelike};
    
    #[test]
    fn test_preprocess_transactions() {
        let mut transactions = vec![
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                "143025".to_string(),
                Decimal::from(50000),
                Decimal::ZERO,
                Decimal::from(120000),
                "个人应收".to_string(),
            ),
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 10).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                "100000".to_string(),
                Decimal::ZERO,
                Decimal::from(30000),
                Decimal::from(70000),
                "公司应付".to_string(),
            ),
            // 无效记录
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 20).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                "120000".to_string(),
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::from(70000),
                "".to_string(),
            ),
        ];
        
        DataProcessor::preprocess_transactions(&mut transactions).unwrap();
        
        assert_eq!(transactions.len(), 2); // 移除了无效记录
        assert_eq!(transactions[0].transaction_date.day(), 10); // 按日期排序
        assert_eq!(transactions[1].transaction_date.day(), 15);
    }
}