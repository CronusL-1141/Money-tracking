//! 分析器接口定义

use crate::errors::AuditResult;
use crate::models::Transaction;

/// 行为分析器接口
pub trait BehaviorAnalyzer: Send + Sync {
    /// 分析交易行为性质
    /// 
    /// # Arguments
    /// * `transaction` - 交易记录
    /// * `personal_ratio` - 个人资金占比
    /// * `company_ratio` - 公司资金占比
    /// 
    /// # Returns
    /// * 行为性质描述
    fn analyze_behavior(
        &self,
        transaction: &Transaction,
        personal_ratio: f64,
        company_ratio: f64,
    ) -> AuditResult<String>;
    
    /// 分析挪用行为
    fn analyze_misappropriation(
        &self,
        transaction: &Transaction,
        current_balance_info: &BalanceInfo,
    ) -> AuditResult<MisappropriationAnalysis>;
    
    /// 分析垫付行为
    fn analyze_advance_payment(
        &self,
        transaction: &Transaction,
        current_balance_info: &BalanceInfo,
    ) -> AuditResult<AdvancePaymentAnalysis>;
}

/// 流量分析器接口
pub trait FlowAnalyzer: Send + Sync {
    /// 分析资金流向
    fn analyze_flow_direction(&self, transaction: &Transaction) -> AuditResult<FlowDirection>;
    
    /// 分析流量完整性
    fn validate_flow_integrity(&self, transactions: &[Transaction]) -> AuditResult<IntegrityReport>;
    
    /// 计算资金占比
    fn calculate_fund_ratio(
        &self,
        amount: rust_decimal::Decimal,
        fund_attribute: &str,
        current_balance: rust_decimal::Decimal,
    ) -> AuditResult<(rust_decimal::Decimal, rust_decimal::Decimal)>;
}

/// 余额信息
#[derive(Debug, Clone)]
pub struct BalanceInfo {
    pub personal_balance: rust_decimal::Decimal,
    pub company_balance: rust_decimal::Decimal,
    pub total_balance: rust_decimal::Decimal,
}

/// 挪用分析结果
#[derive(Debug, Clone)]
pub struct MisappropriationAnalysis {
    pub is_misappropriation: bool,
    pub amount: rust_decimal::Decimal,
    pub description: String,
}

/// 垫付分析结果
#[derive(Debug, Clone)]
pub struct AdvancePaymentAnalysis {
    pub is_advance_payment: bool,
    pub amount: rust_decimal::Decimal,
    pub description: String,
}

/// 资金流向
#[derive(Debug, Clone, PartialEq)]
pub enum FlowDirection {
    PersonalToCompany,
    CompanyToPersonal,
    Investment,
    InvestmentReturn,
    Normal,
}

/// 完整性报告
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub is_valid: bool,
    pub total_records: usize,
    pub inconsistencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_balance_info_creation() {
        let balance_info = BalanceInfo {
            personal_balance: rust_decimal::Decimal::from(50000),
            company_balance: rust_decimal::Decimal::from(30000),
            total_balance: rust_decimal::Decimal::from(80000),
        };
        
        assert_eq!(balance_info.personal_balance, rust_decimal::Decimal::from(50000));
        assert_eq!(balance_info.company_balance, rust_decimal::Decimal::from(30000));
        assert_eq!(balance_info.total_balance, rust_decimal::Decimal::from(80000));
    }
    
    #[test]
    fn test_flow_direction_enum() {
        let direction = FlowDirection::PersonalToCompany;
        assert_eq!(direction, FlowDirection::PersonalToCompany);
        assert_ne!(direction, FlowDirection::CompanyToPersonal);
    }
}