//! 流量分析器实现

use crate::interfaces::{FlowAnalyzer, FlowDirection, IntegrityReport};
use crate::errors::AuditResult;
use crate::models::Transaction;
use rust_decimal::Decimal;

/// 默认流量分析器
#[derive(Debug, Clone)]
pub struct DefaultFlowAnalyzer;

impl DefaultFlowAnalyzer {
    /// 创建新的流量分析器
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultFlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowAnalyzer for DefaultFlowAnalyzer {
    fn analyze_flow_direction(&self, transaction: &Transaction) -> AuditResult<FlowDirection> {
        let direction = if transaction.fund_attribute.starts_with("理财-") ||
                          transaction.fund_attribute.starts_with("投资-") ||
                          transaction.fund_attribute.starts_with("保险-") {
            if transaction.is_income() {
                FlowDirection::InvestmentReturn
            } else {
                FlowDirection::Investment
            }
        } else if transaction.fund_attribute.contains("个人") && transaction.fund_attribute.contains("公司") {
            if transaction.is_income() {
                FlowDirection::CompanyToPersonal
            } else {
                FlowDirection::PersonalToCompany
            }
        } else {
            FlowDirection::Normal
        };
        
        Ok(direction)
    }
    
    fn validate_flow_integrity(&self, transactions: &[Transaction]) -> AuditResult<IntegrityReport> {
        let mut inconsistencies = Vec::new();
        let total_records = transactions.len();
        
        // 检查基本完整性
        for (i, tx) in transactions.iter().enumerate() {
            if tx.fund_attribute.is_empty() {
                inconsistencies.push(format!("第{}行：资金属性为空", i + 1));
            }
            
            if tx.income_amount == Decimal::ZERO && tx.expense_amount == Decimal::ZERO {
                inconsistencies.push(format!("第{}行：收入和支出均为零", i + 1));
            }
            
            if tx.income_amount > Decimal::ZERO && tx.expense_amount > Decimal::ZERO {
                inconsistencies.push(format!("第{}行：收入和支出同时存在", i + 1));
            }
        }
        
        Ok(IntegrityReport {
            is_valid: inconsistencies.is_empty(),
            total_records,
            inconsistencies,
        })
    }
    
    fn calculate_fund_ratio(
        &self,
        amount: Decimal,
        fund_attribute: &str,
        current_balance: Decimal,
    ) -> AuditResult<(Decimal, Decimal)> {
        // 简化实现：基于资金属性判断比例
        let (personal_ratio, company_ratio) = if fund_attribute.contains("个人") {
            (Decimal::ONE, Decimal::ZERO)
        } else if fund_attribute.contains("公司") {
            (Decimal::ZERO, Decimal::ONE)
        } else {
            // 未知类型，按当前余额比例分配
            if current_balance > Decimal::ZERO {
                (Decimal::from_f64_retain(0.5).unwrap(), Decimal::from_f64_retain(0.5).unwrap())
            } else {
                (Decimal::ZERO, Decimal::ZERO)
            }
        };
        
        Ok((personal_ratio, company_ratio))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Transaction;
    use chrono::NaiveDate;
    
    #[test]
    fn test_flow_direction_analysis() {
        let analyzer = DefaultFlowAnalyzer::new();
        
        // 投资交易
        let investment_tx = Transaction::new(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            "143025".to_string(),
            Decimal::ZERO,
            Decimal::from(50000),
            Decimal::from(120000),
            "理财-SL100613100620".to_string(),
        );
        
        let direction = analyzer.analyze_flow_direction(&investment_tx).unwrap();
        assert_eq!(direction, FlowDirection::Investment);
        
        // 正常交易
        let normal_tx = Transaction::new(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            "143025".to_string(),
            Decimal::from(30000),
            Decimal::ZERO,
            Decimal::from(150000),
            "个人应收".to_string(),
        );
        
        let direction = analyzer.analyze_flow_direction(&normal_tx).unwrap();
        assert_eq!(direction, FlowDirection::Normal);
    }
    
    #[test]
    fn test_flow_integrity_validation() {
        let analyzer = DefaultFlowAnalyzer::new();
        
        let transactions = vec![
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                "143025".to_string(),
                Decimal::from(50000),
                Decimal::ZERO,
                Decimal::from(120000),
                "个人应收".to_string(),
            ),
            // 无效交易：资金属性为空
            Transaction::new(
                NaiveDate::from_ymd_opt(2023, 1, 16).unwrap().and_hms_opt(0, 0, 0).unwrap(),
                "143025".to_string(),
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::from(120000),
                "".to_string(),
            ),
        ];
        
        let report = analyzer.validate_flow_integrity(&transactions).unwrap();
        assert!(!report.is_valid);
        assert_eq!(report.total_records, 2);
        assert!(!report.inconsistencies.is_empty());
    }
    
    #[test]
    fn test_fund_ratio_calculation() {
        let analyzer = DefaultFlowAnalyzer::new();
        
        // 个人资金
        let (personal_ratio, company_ratio) = analyzer
            .calculate_fund_ratio(
                Decimal::from(50000),
                "个人应收",
                Decimal::from(100000)
            )
            .unwrap();
        
        assert_eq!(personal_ratio, Decimal::ONE);
        assert_eq!(company_ratio, Decimal::ZERO);
        
        // 公司资金
        let (personal_ratio, company_ratio) = analyzer
            .calculate_fund_ratio(
                Decimal::from(30000),
                "公司应付",
                Decimal::from(100000)
            )
            .unwrap();
        
        assert_eq!(personal_ratio, Decimal::ZERO);
        assert_eq!(company_ratio, Decimal::ONE);
    }
}