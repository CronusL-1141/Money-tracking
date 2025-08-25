//! 审计摘要数据模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 审计分析摘要
/// 
/// 包含审计分析的所有关键指标和统计信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditSummary {
    /// 当前个人余额
    #[serde(rename = "个人余额")]
    pub personal_balance: Decimal,
    
    /// 当前公司余额
    #[serde(rename = "公司余额")]
    pub company_balance: Decimal,
    
    /// 累计挪用金额
    #[serde(rename = "累计挪用金额")]
    pub total_misappropriation: Decimal,
    
    /// 累计垫付金额
    #[serde(rename = "累计垫付金额")]
    pub total_advance_payment: Decimal,
    
    /// 累计由资金池回归公司余额本金
    #[serde(rename = "累计由资金池回归公司余额本金")]
    pub total_company_principal_returned: Decimal,
    
    /// 累计由资金池回归个人余额本金
    #[serde(rename = "累计由资金池回归个人余额本金")]
    pub total_personal_principal_returned: Decimal,
    
    /// 总计个人应分配利润
    #[serde(rename = "总计个人应分配利润")]
    pub total_personal_profit: Decimal,
    
    /// 总计公司应分配利润
    #[serde(rename = "总计公司应分配利润")]
    pub total_company_profit: Decimal,
    
    /// 资金缺口
    #[serde(rename = "资金缺口")]
    pub funding_gap: Decimal,
    
    /// 投资产品数量
    #[serde(rename = "投资产品数量")]
    pub investment_product_count: u32,
    
    /// 总余额
    #[serde(rename = "总余额")]
    pub total_balance: Decimal,
}

impl AuditSummary {
    /// 创建新的审计摘要
    pub fn new() -> Self {
        Self {
            personal_balance: Decimal::ZERO,
            company_balance: Decimal::ZERO,
            total_misappropriation: Decimal::ZERO,
            total_advance_payment: Decimal::ZERO,
            total_company_principal_returned: Decimal::ZERO,
            total_personal_principal_returned: Decimal::ZERO,
            total_personal_profit: Decimal::ZERO,
            total_company_profit: Decimal::ZERO,
            funding_gap: Decimal::ZERO,
            investment_product_count: 0,
            total_balance: Decimal::ZERO,
        }
    }
    
    /// 计算资金缺口
    /// 
    /// 资金缺口 = 挪用金额 - 归还本金 - 垫付金额
    pub fn calculate_funding_gap(&self) -> Decimal {
        self.total_misappropriation 
            - self.total_company_principal_returned
            - self.total_advance_payment
    }
    
    /// 计算总余额
    pub fn calculate_total_balance(&self) -> Decimal {
        self.personal_balance + self.company_balance
    }
    
    /// 计算总利润
    pub fn calculate_total_profit(&self) -> Decimal {
        self.total_personal_profit + self.total_company_profit
    }
    
    /// 计算总归还本金
    pub fn calculate_total_principal_returned(&self) -> Decimal {
        self.total_company_principal_returned + self.total_personal_principal_returned
    }
    
    /// 更新计算字段
    pub fn update_calculated_fields(&mut self) {
        self.funding_gap = self.calculate_funding_gap();
        self.total_balance = self.calculate_total_balance();
    }
    
    /// 检查数据一致性
    pub fn validate(&self) -> Result<(), String> {
        // 检查余额是否为负（可能的异常情况）
        if self.personal_balance < Decimal::ZERO {
            return Err(format!("个人余额为负: {}", self.personal_balance));
        }
        
        if self.company_balance < Decimal::ZERO {
            return Err(format!("公司余额为负: {}", self.company_balance));
        }
        
        // 检查总余额是否一致
        let calculated_total = self.calculate_total_balance();
        if (self.total_balance - calculated_total).abs() > Decimal::from_f64_retain(0.01).unwrap() {
            return Err(format!(
                "总余额不一致: 记录={}, 计算={}",
                self.total_balance, calculated_total
            ));
        }
        
        // 检查资金缺口是否一致
        let calculated_gap = self.calculate_funding_gap();
        if (self.funding_gap - calculated_gap).abs() > Decimal::from_f64_retain(0.01).unwrap() {
            return Err(format!(
                "资金缺口不一致: 记录={}, 计算={}",
                self.funding_gap, calculated_gap
            ));
        }
        
        Ok(())
    }
    
    /// 获取摘要统计信息
    pub fn get_statistics(&self) -> SummaryStatistics {
        SummaryStatistics {
            total_transactions_processed: 0, // 需要从外部传入
            has_funding_gap: self.funding_gap != Decimal::ZERO,
            net_profit_loss: self.calculate_total_profit(),
            personal_balance_ratio: if self.total_balance != Decimal::ZERO {
                self.personal_balance / self.total_balance
            } else {
                Decimal::ZERO
            },
            company_balance_ratio: if self.total_balance != Decimal::ZERO {
                self.company_balance / self.total_balance
            } else {
                Decimal::ZERO
            },
        }
    }
}

impl Default for AuditSummary {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AuditSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== 审计分析摘要 ===")?;
        writeln!(f, "个人余额: {:>15}", self.personal_balance)?;
        writeln!(f, "公司余额: {:>15}", self.company_balance)?;
        writeln!(f, "总余额:   {:>15}", self.total_balance)?;
        writeln!(f, "累计挪用: {:>15}", self.total_misappropriation)?;
        writeln!(f, "累计垫付: {:>15}", self.total_advance_payment)?;
        writeln!(f, "个人利润: {:>15}", self.total_personal_profit)?;
        writeln!(f, "公司利润: {:>15}", self.total_company_profit)?;
        writeln!(f, "资金缺口: {:>15}", self.funding_gap)?;
        writeln!(f, "投资产品数量: {}", self.investment_product_count)?;
        write!(f, "==================")
    }
}

/// 摘要统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStatistics {
    /// 处理的交易总数
    pub total_transactions_processed: usize,
    
    /// 是否存在资金缺口
    pub has_funding_gap: bool,
    
    /// 净盈亏
    pub net_profit_loss: Decimal,
    
    /// 个人余额占比
    pub personal_balance_ratio: Decimal,
    
    /// 公司余额占比
    pub company_balance_ratio: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_summary_creation() {
        let summary = AuditSummary::new();
        
        assert_eq!(summary.personal_balance, Decimal::ZERO);
        assert_eq!(summary.company_balance, Decimal::ZERO);
        assert_eq!(summary.total_balance, Decimal::ZERO);
        assert_eq!(summary.funding_gap, Decimal::ZERO);
        assert_eq!(summary.investment_product_count, 0);
    }
    
    #[test]
    fn test_funding_gap_calculation() {
        let mut summary = AuditSummary::new();
        summary.total_misappropriation = Decimal::from(100000);
        summary.total_company_principal_returned = Decimal::from(30000);
        summary.total_advance_payment = Decimal::from(20000);
        
        let gap = summary.calculate_funding_gap();
        assert_eq!(gap, Decimal::from(50000)); // 100000 - 30000 - 20000
    }
    
    #[test]
    fn test_total_balance_calculation() {
        let mut summary = AuditSummary::new();
        summary.personal_balance = Decimal::from(75000);
        summary.company_balance = Decimal::from(45000);
        
        let total = summary.calculate_total_balance();
        assert_eq!(total, Decimal::from(120000));
    }
    
    #[test]
    fn test_validation() {
        let mut summary = AuditSummary::new();
        summary.personal_balance = Decimal::from(75000);
        summary.company_balance = Decimal::from(45000);
        summary.total_balance = Decimal::from(120000);
        summary.total_misappropriation = Decimal::from(50000);
        summary.total_advance_payment = Decimal::from(10000);
        summary.total_company_principal_returned = Decimal::from(20000);
        summary.funding_gap = Decimal::from(20000); // 50000 - 20000 - 10000
        
        assert!(summary.validate().is_ok());
        
        // 测试不一致的情况
        summary.total_balance = Decimal::from(100000); // 错误的总余额
        assert!(summary.validate().is_err());
    }
}