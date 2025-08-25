//! 追踪器接口定义

use crate::errors::AuditResult;
use crate::models::{Transaction, AuditSummary};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

/// 资金追踪器接口
/// 
/// 定义所有追踪器必须实现的核心方法
pub trait Tracker: Send + Sync {
    /// 初始化余额
    /// 
    /// # Arguments
    /// * `initial_balance` - 初始余额
    /// * `balance_type` - 余额类型描述
    fn initialize_balance(&mut self, initial_balance: Decimal, balance_type: &str) -> AuditResult<()>;
    
    /// 处理资金流入
    /// 
    /// # Arguments
    /// * `amount` - 流入金额
    /// * `fund_attribute` - 资金属性
    /// * `transaction_date` - 交易日期（可选）
    /// 
    /// # Returns
    /// * `(personal_ratio, company_ratio, behavior_nature)` - 个人占比、公司占比、行为性质
    fn process_inflow(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)>;
    
    /// 处理资金流出
    /// 
    /// # Arguments
    /// * `amount` - 流出金额
    /// * `fund_attribute` - 资金属性
    /// * `transaction_date` - 交易日期（可选）
    /// 
    /// # Returns
    /// * `(personal_ratio, company_ratio, behavior_nature)` - 个人占比、公司占比、行为性质
    fn process_outflow(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)>;
    
    /// 处理投资产品赎回
    /// 
    /// # Arguments
    /// * `amount` - 赎回金额
    /// * `fund_attribute` - 资金属性（产品名称）
    /// * `transaction_date` - 交易日期（可选）
    /// 
    /// # Returns
    /// * `(personal_ratio, company_ratio, behavior_nature)` - 个人占比、公司占比、行为性质
    fn process_investment_redemption(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)>;
    
    /// 获取当前审计摘要
    fn get_summary(&self) -> AuditResult<AuditSummary>;
    
    /// 获取当前资金占比
    /// 
    /// # Returns
    /// * `(personal_ratio, company_ratio)` - 个人占比、公司占比
    fn get_current_ratios(&self) -> AuditResult<(Decimal, Decimal)>;
    
    /// 检查是否已初始化
    fn is_initialized(&self) -> bool;
    
    /// 获取追踪器名称
    fn get_name(&self) -> &'static str;
    
    /// 获取追踪器描述
    fn get_description(&self) -> &'static str;
    
    /// 重置追踪器状态
    fn reset(&mut self) -> AuditResult<()>;
    
    /// 处理完整的交易记录
    /// 
    /// 这是一个便利方法，会根据交易类型调用相应的处理方法
    fn process_transaction(&mut self, transaction: &mut Transaction) -> AuditResult<()> {
        let (personal_ratio, company_ratio, behavior_nature) = if transaction.is_income() {
            self.process_inflow(
                transaction.income_amount,
                &transaction.fund_attribute,
                Some(transaction.transaction_date),
            )?
        } else if transaction.is_expense() {
            if self.is_investment_product(&transaction.fund_attribute) {
                self.process_investment_redemption(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )?
            } else {
                self.process_outflow(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )?
            }
        } else {
            return Err(crate::errors::AuditError::validation_error(
                "交易记录既不是收入也不是支出"
            ));
        };
        
        // 获取当前摘要信息
        let summary = self.get_summary()?;
        
        // 设置交易的计算字段
        transaction.set_calculated_fields(
            personal_ratio,
            company_ratio,
            behavior_nature,
            summary.total_misappropriation,
            summary.total_advance_payment,
            summary.total_company_principal_returned,
            summary.total_personal_principal_returned,
            summary.total_personal_profit,
            summary.total_company_profit,
            summary.funding_gap,
            summary.personal_balance,
            summary.company_balance,
        );
        
        Ok(())
    }
    
    /// 判断是否为投资产品（默认实现）
    fn is_investment_product(&self, fund_attribute: &str) -> bool {
        fund_attribute.starts_with("理财-") ||
        fund_attribute.starts_with("投资-") ||
        fund_attribute.starts_with("保险-") ||
        fund_attribute.starts_with("资金池-")
    }
}