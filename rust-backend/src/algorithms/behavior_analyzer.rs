//! 行为分析器实现

use crate::interfaces::{BehaviorAnalyzer, BalanceInfo, MisappropriationAnalysis, AdvancePaymentAnalysis};
use crate::errors::AuditResult;
use crate::models::Transaction;
use rust_decimal::Decimal;

/// 默认行为分析器
#[derive(Debug, Clone)]
pub struct DefaultBehaviorAnalyzer;

impl DefaultBehaviorAnalyzer {
    /// 创建新的行为分析器
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultBehaviorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorAnalyzer for DefaultBehaviorAnalyzer {
    fn analyze_behavior(
        &self,
        _transaction: &Transaction,
        personal_ratio: f64,
        company_ratio: f64,
    ) -> AuditResult<String> {
        let behavior = match (personal_ratio > 0.0, company_ratio > 0.0) {
            (true, true) => "混合资金",
            (true, false) => "个人资金",
            (false, true) => "公司资金",
            (false, false) => "无资金流动",
        };
        
        Ok(behavior.to_string())
    }
    
    fn analyze_misappropriation(
        &self,
        transaction: &Transaction,
        balance_info: &BalanceInfo,
    ) -> AuditResult<MisappropriationAnalysis> {
        // 简化实现：基本的挪用检测逻辑
        let is_misappropriation = transaction.fund_attribute.contains("个人") && 
                                 transaction.is_expense() &&
                                 balance_info.company_balance < transaction.expense_amount;
        
        let amount = if is_misappropriation {
            transaction.expense_amount - balance_info.company_balance.max(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };
        
        Ok(MisappropriationAnalysis {
            is_misappropriation,
            amount,
            description: if is_misappropriation {
                "检测到可能的挪用行为".to_string()
            } else {
                "正常交易".to_string()
            },
        })
    }
    
    fn analyze_advance_payment(
        &self,
        transaction: &Transaction,
        balance_info: &BalanceInfo,
    ) -> AuditResult<AdvancePaymentAnalysis> {
        // 简化实现：基本的垫付检测逻辑
        let is_advance_payment = transaction.fund_attribute.contains("公司") && 
                                transaction.is_expense() &&
                                balance_info.personal_balance < transaction.expense_amount;
        
        let amount = if is_advance_payment {
            transaction.expense_amount - balance_info.personal_balance.max(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };
        
        Ok(AdvancePaymentAnalysis {
            is_advance_payment,
            amount,
            description: if is_advance_payment {
                "检测到可能的垫付行为".to_string()
            } else {
                "正常交易".to_string()
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_behavior_analysis() {
        let analyzer = DefaultBehaviorAnalyzer::new();
        
        // 测试个人资金
        let result = analyzer.analyze_behavior(&create_test_transaction(), 1.0, 0.0).unwrap();
        assert_eq!(result, "个人资金");
        
        // 测试公司资金
        let result = analyzer.analyze_behavior(&create_test_transaction(), 0.0, 1.0).unwrap();
        assert_eq!(result, "公司资金");
        
        // 测试混合资金
        let result = analyzer.analyze_behavior(&create_test_transaction(), 0.6, 0.4).unwrap();
        assert_eq!(result, "混合资金");
    }
    
    fn create_test_transaction() -> Transaction {
        use crate::models::Transaction;
        
        Transaction::new(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            "143025".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(120000),
            "个人应收".to_string(),
        )
    }
}