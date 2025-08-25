//! 数据预处理器
//! 
//! 对应Python版本DataProcessor中的数据预处理功能，专门负责将原始Excel数据转换为业务数据

use crate::errors::{AuditError, AuditResult};
use crate::models::{Transaction, Config};
use crate::utils::{ExcelReader, TimeProcessor, RawTransactionRow};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use log::{info, warn, debug};

/// 数据预处理器
#[derive(Debug)]
pub struct DataPreprocessor {
    config: Config,
}

impl DataPreprocessor {
    /// 创建新的数据预处理器
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 预处理财务数据
    /// 对应Python版本DataProcessor.预处理财务数据方法
    pub fn preprocess_financial_data(&self, raw_rows: Vec<RawTransactionRow>) -> AuditResult<Vec<Transaction>> {
        info!("🔄 开始数据预处理，原始行数: {}", raw_rows.len());
        
        let mut transactions = Vec::with_capacity(raw_rows.len());
        let mut processed_count = 0;
        let mut error_count = 0;
        
        // 逐行处理原始数据
        for raw_row in raw_rows {
            match self.process_raw_row(raw_row) {
                Ok(transaction) => {
                    transactions.push(transaction);
                    processed_count += 1;
                }
                Err(e) => {
                    warn!("处理第{}行失败: {}", raw_row.row_number, e);
                    error_count += 1;
                    // 继续处理其他行
                }
            }
            
            // 定期报告进度
            if processed_count % 1000 == 0 && processed_count > 0 {
                info!("⏳ 预处理进度: {}/{} ({:.1}%)", 
                    processed_count, 
                    processed_count + error_count,
                    processed_count as f64 / (processed_count + error_count) as f64 * 100.0
                );
            }
        }
        
        info!("📊 预处理统计: 成功处理{}行，失败{}行", processed_count, error_count);
        
        // 按时间排序（对应Python版本的排序逻辑）
        self.sort_transactions_by_time(&mut transactions)?;
        
        // 初始化结果列
        self.initialize_result_columns(&mut transactions);
        
        info!("✅ 数据预处理完成，最终交易记录数: {}", transactions.len());
        Ok(transactions)
    }
    
    /// 处理单个原始行
    fn process_raw_row(&self, raw_row: RawTransactionRow) -> AuditResult<Transaction> {
        // 解析交易日期
        let transaction_date = TimeProcessor::parse_excel_date(&raw_row.date_cell)
            .map_err(|e| AuditError::validation_error(
                format!("第{}行日期解析失败: {}", raw_row.row_number, e)
            ))?;
        
        // 解析交易时间
        let transaction_time = TimeProcessor::parse_transaction_time(&raw_row.time_cell);
        
        // 创建完整时间戳
        let complete_timestamp = TimeProcessor::create_complete_timestamp(transaction_date, &transaction_time);
        
        // 验证时间戳合理性
        if !TimeProcessor::validate_timestamp(&complete_timestamp) {
            warn!("第{}行时间戳不合理: {}", raw_row.row_number, complete_timestamp);
        }
        
        // 解析金额字段
        let income_amount = ExcelReader::parse_decimal(&raw_row.income_cell)
            .unwrap_or_else(|e| {
                warn!("第{}行收入金额解析失败: {}, 使用0", raw_row.row_number, e);
                Decimal::ZERO
            });
        
        let expense_amount = ExcelReader::parse_decimal(&raw_row.expense_cell)
            .unwrap_or_else(|e| {
                warn!("第{}行支出金额解析失败: {}, 使用0", raw_row.row_number, e);
                Decimal::ZERO
            });
        
        let balance = ExcelReader::parse_decimal(&raw_row.balance_cell)
            .map_err(|e| AuditError::validation_error(
                format!("第{}行余额解析失败: {}", raw_row.row_number, e)
            ))?;
        
        // 解析资金属性
        let fund_attribute = ExcelReader::parse_string(&raw_row.fund_attribute_cell);
        
        if fund_attribute.trim().is_empty() {
            return Err(AuditError::validation_error(
                format!("第{}行资金属性不能为空", raw_row.row_number)
            ));
        }
        
        // 创建交易记录
        let mut transaction = Transaction::new(
            complete_timestamp,
            transaction_time,
            income_amount,
            expense_amount,
            balance,
            fund_attribute,
        );
        
        // 记录原始行号
        transaction.original_row_number = Some(raw_row.row_number);
        
        Ok(transaction)
    }
    
    /// 按时间排序交易记录
    /// 对应Python版本的排序逻辑，保持相同时间交易的原始顺序
    fn sort_transactions_by_time(&self, transactions: &mut [Transaction]) -> AuditResult<()> {
        info!("🔄 按时间排序交易记录");
        
        // 使用稳定排序，保持相同时间戳的原始顺序
        transactions.sort_by(|a, b| {
            let time_cmp = a.transaction_date.cmp(&b.transaction_date);
            if time_cmp == std::cmp::Ordering::Equal {
                // 如果时间相同，按原始行号排序
                a.original_row_number.cmp(&b.original_row_number)
            } else {
                time_cmp
            }
        });
        
        info!("✅ 时间排序完成");
        Ok(())
    }
    
    /// 初始化结果列
    /// 对应Python版本DataProcessor._initialize_result_columns方法
    fn initialize_result_columns(&self, transactions: &mut [Transaction]) {
        info!("🔄 初始化结果列");
        
        for transaction in transactions.iter_mut() {
            // 初始化所有结果字段为默认值
            transaction.personal_ratio = Some(Decimal::ZERO);
            transaction.company_ratio = Some(Decimal::ZERO);
            transaction.behavior_nature = Some(String::new());
            transaction.cumulative_misappropriation = Some(Decimal::ZERO);
            transaction.cumulative_advance = Some(Decimal::ZERO);
            transaction.cumulative_company_principal_returned = Some(Decimal::ZERO);
            transaction.cumulative_personal_principal_returned = Some(Decimal::ZERO);
            transaction.cumulative_illegal_gains = Some(Decimal::ZERO);
            transaction.total_personal_profit = Some(Decimal::ZERO);
            transaction.total_company_profit = Some(Decimal::ZERO);
            transaction.personal_balance = Some(Decimal::ZERO);
            transaction.company_balance = Some(Decimal::ZERO);
            transaction.total_balance = Some(Decimal::ZERO);
            transaction.funding_gap = Some(Decimal::ZERO);
        }
        
        info!("✅ 结果列初始化完成");
    }
    
    /// 计算初始余额
    /// 对应Python版本DataProcessor.计算初始余额方法
    pub fn calculate_initial_balance(&self, transactions: &[Transaction], silent: bool) -> Decimal {
        if transactions.is_empty() {
            return Decimal::ZERO;
        }
        
        let first_tx = &transactions[0];
        let first_balance = first_tx.balance;
        
        // 计算第一笔交易金额
        let first_transaction_amount = if first_tx.income_amount > Decimal::ZERO {
            first_tx.income_amount
        } else if first_tx.expense_amount > Decimal::ZERO {
            -first_tx.expense_amount
        } else {
            Decimal::ZERO
        };
        
        // 计算交易前余额（初始余额）
        let initial_balance = first_balance - first_transaction_amount;
        
        // 非静默模式下才输出日志
        if initial_balance > Decimal::ZERO && !silent {
            info!("💰 计算得出初始余额: {:.2}（第一笔余额{:.2} - 第一笔交易{:.2}）", 
                initial_balance, first_balance, first_transaction_amount);
            info!("📋 将初始余额作为公司应收在FIFO账本簿中初始化");
        }
        
        initial_balance
    }
    
    /// 生成数据摘要
    /// 对应Python版本DataProcessor.生成数据摘要方法
    pub fn generate_data_summary(&self, transactions: &[Transaction]) -> DataSummary {
        let mut summary = DataSummary::new();
        
        summary.total_rows = transactions.len();
        
        if transactions.is_empty() {
            return summary;
        }
        
        // 时间范围统计
        let start_time = transactions.first().unwrap().transaction_date;
        let end_time = transactions.last().unwrap().transaction_date;
        summary.time_range = Some(TimeRange {
            start_time,
            end_time,
            total_days: (end_time - start_time).num_days(),
        });
        
        // 金额统计
        let mut total_income = Decimal::ZERO;
        let mut total_expense = Decimal::ZERO;
        let mut max_income = Decimal::ZERO;
        let mut max_expense = Decimal::ZERO;
        
        for tx in transactions {
            if tx.income_amount > Decimal::ZERO {
                total_income += tx.income_amount;
                if tx.income_amount > max_income {
                    max_income = tx.income_amount;
                }
            }
            
            if tx.expense_amount > Decimal::ZERO {
                total_expense += tx.expense_amount;
                if tx.expense_amount > max_expense {
                    max_expense = tx.expense_amount;
                }
            }
        }
        
        summary.amount_stats = Some(AmountStats {
            total_income,
            total_expense,
            net_inflow: total_income - total_expense,
            max_income,
            max_expense,
        });
        
        summary
    }
    
    /// 显示时间戳示例
    /// 对应Python版本的_show_timestamp_examples方法
    pub fn show_timestamp_examples(&self, transactions: &[Transaction], count: usize) {
        if transactions.is_empty() {
            warn!("没有交易记录可显示");
            return;
        }
        
        info!("📅 完整时间戳示例:");
        let display_count = std::cmp::min(count, transactions.len());
        
        for (i, tx) in transactions.iter().take(display_count).enumerate() {
            info!("  {}. {} (原始时间: {})", 
                i + 1, 
                tx.transaction_date.format("%Y-%m-%d %H:%M:%S"),
                tx.transaction_time
            );
        }
    }
}

/// 数据摘要
#[derive(Debug, Clone)]
pub struct DataSummary {
    pub total_rows: usize,
    pub time_range: Option<TimeRange>,
    pub amount_stats: Option<AmountStats>,
}

impl DataSummary {
    fn new() -> Self {
        Self {
            total_rows: 0,
            time_range: None,
            amount_stats: None,
        }
    }
}

/// 时间范围信息
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub total_days: i64,
}

/// 金额统计信息
#[derive(Debug, Clone)]
pub struct AmountStats {
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub net_inflow: Decimal,
    pub max_income: Decimal,
    pub max_expense: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_calculate_initial_balance() {
        let config = Config::new();
        let preprocessor = DataPreprocessor::new(config);
        
        let transactions = vec![
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(14, 30, 25).unwrap(),
                "143025".to_string(),
                Decimal::from(50000),
                Decimal::ZERO,
                Decimal::from(150000),
                "个人应收".to_string(),
            )
        ];
        
        // 初始余额 = 150000 - 50000 = 100000
        let initial_balance = preprocessor.calculate_initial_balance(&transactions, true);
        assert_eq!(initial_balance, Decimal::from(100000));
    }
    
    #[test]
    fn test_generate_data_summary() {
        let config = Config::new();
        let preprocessor = DataPreprocessor::new(config);
        
        let transactions = vec![
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(14, 30, 25).unwrap(),
                "143025".to_string(),
                Decimal::from(50000),
                Decimal::ZERO,
                Decimal::from(150000),
                "个人应收".to_string(),
            ),
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 16).unwrap().and_hms_opt(10, 15, 30).unwrap(),
                "101530".to_string(),
                Decimal::ZERO,
                Decimal::from(30000),
                Decimal::from(120000),
                "个人支出".to_string(),
            )
        ];
        
        let summary = preprocessor.generate_data_summary(&transactions);
        
        assert_eq!(summary.total_rows, 2);
        assert!(summary.time_range.is_some());
        assert!(summary.amount_stats.is_some());
        
        if let Some(amount_stats) = summary.amount_stats {
            assert_eq!(amount_stats.total_income, Decimal::from(50000));
            assert_eq!(amount_stats.total_expense, Decimal::from(30000));
            assert_eq!(amount_stats.net_inflow, Decimal::from(20000));
        }
    }
}