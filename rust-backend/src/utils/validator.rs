//! 数据验证工具

use crate::errors::{AuditError, AuditResult};
use crate::models::{Transaction, Config};
use rust_decimal::Decimal;
use log::{info, warn, error};

/// 数据验证器
#[derive(Debug)]
pub struct Validator {
    config: Config,
}

impl Validator {
    /// 创建新的验证器
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 验证交易数据完整性
    pub fn validate_transactions(&self, transactions: &[Transaction]) -> AuditResult<ValidationResult> {
        info!("🔍 开始数据验证");
        
        let mut result = ValidationResult::new();
        result.total_records = transactions.len();
        
        // 验证每条交易记录
        for (index, tx) in transactions.iter().enumerate() {
            if let Err(e) = self.validate_single_transaction(tx) {
                result.errors.push(ValidationError {
                    row_index: index + 1,
                    error_type: "交易记录验证".to_string(),
                    message: e.to_string(),
                });
            }
        }
        
        // 验证流水完整性
        if let Err(e) = self.validate_balance_continuity(transactions) {
            result.errors.push(ValidationError {
                row_index: 0,
                error_type: "余额连续性验证".to_string(),
                message: e.to_string(),
            });
        }
        
        result.is_valid = result.errors.is_empty();
        
        if result.is_valid {
            info!("✅ 数据验证通过");
        } else {
            warn!("⚠️ 发现 {} 个验证错误", result.errors.len());
            for error in &result.errors {
                error!("第{}行: {} - {}", error.row_index, error.error_type, error.message);
            }
        }
        
        Ok(result)
    }
    
    /// 验证单条交易记录
    fn validate_single_transaction(&self, tx: &Transaction) -> AuditResult<()> {
        // 验证资金属性不为空
        if tx.fund_attribute.trim().is_empty() {
            return Err(AuditError::validation_error("资金属性不能为空"));
        }
        
        // 验证收入和支出不能同时为正
        if tx.income_amount > Decimal::ZERO && tx.expense_amount > Decimal::ZERO {
            return Err(AuditError::validation_error("收入和支出不能同时存在"));
        }
        
        // 验证收入和支出不能同时为零
        if tx.income_amount == Decimal::ZERO && tx.expense_amount == Decimal::ZERO {
            return Err(AuditError::validation_error("收入和支出不能同时为零"));
        }
        
        // 验证余额不能为负（某些特殊情况除外）
        if tx.balance < Decimal::ZERO && !self.is_negative_balance_allowed(&tx.fund_attribute) {
            return Err(AuditError::validation_error("余额不能为负"));
        }
        
        Ok(())
    }
    
    /// 验证余额连续性
    fn validate_balance_continuity(&self, transactions: &[Transaction]) -> AuditResult<()> {
        if transactions.len() < 2 {
            return Ok(());
        }
        
        for i in 1..transactions.len() {
            let prev_tx = &transactions[i - 1];
            let curr_tx = &transactions[i];
            
            // 计算预期余额
            let expected_balance = prev_tx.balance + curr_tx.income_amount - curr_tx.expense_amount;
            
            // 检查余额是否匹配（考虑容差）
            if !self.config.is_balance_within_tolerance(expected_balance, curr_tx.balance) {
                return Err(AuditError::validation_error(
                    format!("第{}行余额不连续: 预期={}, 实际={}", 
                        i + 1, expected_balance, curr_tx.balance)
                ));
            }
        }
        
        Ok(())
    }
    
    /// 判断是否允许负余额
    fn is_negative_balance_allowed(&self, fund_attribute: &str) -> bool {
        // 某些特殊账户类型可能允许负余额
        fund_attribute.contains("应付") || fund_attribute.contains("借款")
    }
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub total_records: usize,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            is_valid: true,
            total_records: 0,
            errors: Vec::new(),
        }
    }
}

/// 验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub row_index: usize,
    pub error_type: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_single_transaction_validation() {
        let config = Config::new();
        let validator = Validator::new(config);
        
        // 有效交易
        let valid_tx = Transaction::new(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            "143025".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(120000),
            "个人应收".to_string(),
        );
        
        assert!(validator.validate_single_transaction(&valid_tx).is_ok());
        
        // 无效交易 - 资金属性为空
        let invalid_tx = Transaction::new(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            "143025".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(120000),
            "".to_string(),
        );
        
        assert!(validator.validate_single_transaction(&invalid_tx).is_err());
    }
    
    #[test]
    fn test_balance_continuity() {
        let config = Config::new();
        let validator = Validator::new(config);
        
        let transactions = vec![
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 10).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                "100000".to_string(),
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::from(100000),
                "初始余额".to_string(),
            ),
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                "143025".to_string(),
                Decimal::from(50000),
                Decimal::ZERO,
                Decimal::from(150000), // 100000 + 50000
                "个人应收".to_string(),
            ),
        ];
        
        // 修改第一个交易，让它有合理的收入或支出
        let mut modified_transactions = transactions;
        modified_transactions[0].income_amount = Decimal::from(100000);
        modified_transactions[0].expense_amount = Decimal::ZERO;
        
        assert!(validator.validate_balance_continuity(&modified_transactions).is_ok());
    }
}