//! 差额计算法追踪器实现
//!
//! 基于新共享架构实现，使用TrackerBase作为基础状态管理
//! 对应Python版本的差额计算法资金追踪器完整功能

use super::shared::{
    TrackerBase, BehaviorAnalyzer, InvestmentPoolManager, FundFlowCommon, SummaryGenerator
};
use crate::data_models::{Config, AuditSummary, Transaction};
use crate::errors::{AuditError, AuditResult};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

/// 差额计算法追踪器
/// 
/// 基于共享架构实现差额计算法的资金追踪算法
/// 特点：余额优先扣除策略，简化计算逻辑
/// 对应Python版本的差额计算法资金追踪器
#[derive(Debug, Clone)]
pub struct BalanceMethodTracker {
    /// 共享基础状态（13个状态变量）
    base: TrackerBase,
    /// 行为分析器（挪用垫付分析）
    behavior_analyzer: BehaviorAnalyzer,
}

impl BalanceMethodTracker {
    /// 创建新的差额计算法追踪器
    /// 对应Python版本的__init__方法
    pub fn new(config: Config) -> Self {
        Self {
            base: TrackerBase::new(config),
            behavior_analyzer: BehaviorAnalyzer::new(),
        }
    }
    
    /// 初始化余额
    /// 对应Python版本的初始化余额处理
    pub fn initialize_balance(&mut self, initial_balance: Decimal, balance_type: &str) -> AuditResult<()> {
        // 使用基础类初始化
        self.base.initialize_balance(initial_balance, balance_type)
    }
    
    /// 处理资金流入
    /// 对应Python版本的处理资金流入方法
    pub fn process_inflow(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.base.is_initialized() {
            return Err(AuditError::validation_error("追踪器未初始化"));
        }
        
        // 使用共同资金流处理逻辑
        let (personal_ratio, company_ratio, behavior) = FundFlowCommon::process_fund_inflow(
            &mut self.base,
            amount,
            fund_attribute,
            transaction_date,
        );
        
        Ok((personal_ratio, company_ratio, behavior))
    }
    
    /// 差额计算法资金扣除函数
    /// 对应Python版本的余额优先扣除逻辑 - 根据资金属性优先扣除对应账户
    fn balance_method_deduction_by_attribute(&mut self, amount: Decimal, fund_attribute: &str) -> (Decimal, Decimal) {
        let personal_balance = self.base.personal_balance;
        let company_balance = self.base.company_balance;
        
        // 根据资金属性确定优先扣除的账户
        let (personal_deducted, company_deducted) = if self.base.config.is_personal_fund(fund_attribute) {
            // 个人相关支出，优先扣除个人余额
            let personal_used = amount.min(personal_balance);
            let remaining = amount - personal_used;
            let company_used = remaining.min(company_balance);
            (personal_used, company_used)
        } else if self.base.config.is_company_fund(fund_attribute) {
            // 公司相关支出，优先扣除公司余额
            let company_used = amount.min(company_balance);
            let remaining = amount - company_used;
            let personal_used = remaining.min(personal_balance);
            (personal_used, company_used)
        } else {
            // 其他支出，优先使用余额较多的一方
            if personal_balance >= company_balance {
                let personal_used = amount.min(personal_balance);
                let remaining = amount - personal_used;
                let company_used = remaining.min(company_balance);
                (personal_used, company_used)
            } else {
                let company_used = amount.min(company_balance);
                let remaining = amount - company_used;
                let personal_used = remaining.min(personal_balance);
                (personal_used, company_used)
            }
        };
        
        // 更新基础余额
        FundFlowCommon::update_balances_with_deduction(
            &mut self.base,
            personal_deducted,
            company_deducted,
        );
        
        (personal_deducted, company_deducted)
    }
    
    /// 处理普通资金流出
    /// 对应Python版本的普通支出处理逻辑
    pub fn process_outflow(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.base.is_initialized() {
            return Err(AuditError::validation_error("追踪器未初始化"));
        }
        
        // 使用差额计算法扣除函数 - 根据资金属性优先扣除对应账户
        let (personal_deduction, company_deduction) = self.balance_method_deduction_by_attribute(amount, fund_attribute);
        
        // 计算占比（基于原始金额）
        let (personal_ratio, company_ratio) = FundFlowCommon::calculate_ratios(
            personal_deduction,
            company_deduction,
            amount,
        );
        
        // 分析行为性质
        let behavior = FundFlowCommon::analyze_common_outflow_behavior(
            &mut self.base,
            &mut self.behavior_analyzer,
            fund_attribute,
            personal_deduction,
            company_deduction,
            amount,
        );
        
        Ok((personal_ratio, company_ratio, behavior))
    }
    
    /// 处理投资产品申购
    /// 对应Python版本的投资产品申购逻辑
    pub fn process_investment_purchase(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.base.is_initialized() {
            return Err(AuditError::validation_error("追踪器未初始化"));
        }
        
        // 使用共同投资处理逻辑，传入差额计算法扣除函数
        let result = FundFlowCommon::process_investment_purchase(
            &mut self.base,
            &self.behavior_analyzer,
            amount,
            fund_attribute,
            transaction_date,
            |base, amount| {
                // 差额计算法扣除逻辑的闭包版本 - 应该使用正常的差额计算法逻辑
                let personal_balance = base.personal_balance;
                let company_balance = base.company_balance;
                
                // 投资申购是个人行为（带"-"分隔），优先扣除个人余额，不足部分算挪用公司资金
                let personal_used = amount.min(personal_balance);
                let remaining = amount - personal_used;
                let company_used = remaining.min(company_balance);
                let (personal_deducted, company_deducted) = (personal_used, company_used);
                
                // 更新余额
                base.personal_balance = (base.personal_balance - personal_deducted).max(Decimal::ZERO);
                base.company_balance = (base.company_balance - company_deducted).max(Decimal::ZERO);
                base.update_total_balance();
                
                (personal_deducted, company_deducted)
            },
        );
        
        result.map_err(|e| AuditError::validation_error(e))
    }
    
    /// 处理投资产品赎回
    /// 对应Python版本的投资产品赎回逻辑
    pub fn process_investment_redemption(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.base.is_initialized() {
            return Err(AuditError::validation_error("追踪器未初始化"));
        }
        
        // 使用投资产品管理器处理赎回
        let result = InvestmentPoolManager::process_investment_redemption(
            &mut self.base,
            fund_attribute,
            amount,
            transaction_date,
        );
        
        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(AuditError::validation_error(e)),
        }
    }
    
    /// 获取审计摘要
    /// 对应Python版本的获取状态摘要方法
    pub fn get_summary(&self) -> AuditResult<AuditSummary> {
        Ok(SummaryGenerator::generate_audit_summary(&self.base))
    }
    
    /// 获取当前余额占比
    pub fn get_current_ratios(&self) -> AuditResult<(Decimal, Decimal)> {
        Ok(self.base.get_current_ratios())
    }
    
    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.base.is_initialized()
    }
    
    /// 获取算法名称
    pub fn get_name(&self) -> &'static str {
        "BALANCE_METHOD"
    }
    
    /// 获取算法描述
    pub fn get_description(&self) -> &'static str {
        "差额计算法 - 基于当前余额优先策略进行资金扣除"
    }
    
    /// 重置追踪器状态
    pub fn reset(&mut self) -> AuditResult<()> {
        self.base.reset();
        self.behavior_analyzer = BehaviorAnalyzer::new();
        Ok(())
    }
    
    /// 生成详细的摘要文本
    pub fn generate_detailed_summary_text(&self) -> String {
        SummaryGenerator::generate_detailed_summary_text(&self.base, "差额计算法")
    }
    
    /// 获取场外资金池记录管理器
    pub fn get_offsite_pool_records(&self) -> &crate::data_models::OffsitePoolRecordManager {
        &self.base.offsite_pool_records
    }
    
    /// 获取投资池数据（用于完整统计计算）
    pub fn get_investment_pools(&self) -> &std::collections::HashMap<String, crate::algorithms::shared::tracker_base::InvestmentPool> {
        &self.base.investment_pools
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_balance_method_tracker_creation() {
        let config = Config::new();
        let tracker = BalanceMethodTracker::new(config);
        
        assert_eq!(tracker.get_name(), "BALANCE_METHOD");
        assert!(!tracker.is_initialized());
        assert_eq!(tracker.base.personal_balance, Decimal::ZERO);
        assert_eq!(tracker.base.company_balance, Decimal::ZERO);
    }
    
    #[test]
    fn test_initialization() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        let result = tracker.initialize_balance(Decimal::from(100000), "个人");
        assert!(result.is_ok());
        assert!(tracker.is_initialized());
        assert_eq!(tracker.base.personal_balance, Decimal::from(100000));
    }
    
    #[test]
    fn test_process_inflow() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        tracker.initialize_balance(Decimal::from(50000), "个人").unwrap();
        
        let result = tracker.process_inflow(
            Decimal::from(30000),
            "个人应收",
            None,
        );
        
        assert!(result.is_ok());
        let (personal_ratio, company_ratio, behavior) = result.unwrap();
        assert_eq!(personal_ratio, Decimal::ONE);
        assert_eq!(company_ratio, Decimal::ZERO);
        assert!(behavior.contains("个人资金流入"));
        assert_eq!(tracker.base.personal_balance, Decimal::from(80000));
    }
    
    #[test]
    fn test_balance_method_priority() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        // 设置初始余额：个人60000，公司40000（个人余额更多）
        tracker.initialize_balance(Decimal::from(60000), "个人").unwrap();
        tracker.process_inflow(Decimal::from(40000), "公司应收", None).unwrap();
        
        // 流出50000，应该优先扣除个人资金
        let result = tracker.process_outflow(
            Decimal::from(50000),
            "其他支出",
            None,
        );
        
        assert!(result.is_ok());
        let (personal_ratio, company_ratio, _behavior) = result.unwrap();
        
        // 个人余额更多，应该优先扣除个人资金
        // 50000全部来自个人资金：50000个人 + 0公司
        assert_eq!(personal_ratio, Decimal::ONE); // 50000/50000
        assert_eq!(company_ratio, Decimal::ZERO); // 0/50000
        assert_eq!(tracker.base.personal_balance, Decimal::from(10000)); // 60000-50000
        assert_eq!(tracker.base.company_balance, Decimal::from(40000)); // 未动
    }
    
    #[test]
    fn test_balance_method_mixed_deduction() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        // 设置初始余额：个人30000，公司70000（公司余额更多）
        tracker.initialize_balance(Decimal::from(30000), "个人").unwrap();
        tracker.process_inflow(Decimal::from(70000), "公司应收", None).unwrap();
        
        // 流出80000，应该优先扣除公司资金，不足部分扣除个人资金
        let result = tracker.process_outflow(
            Decimal::from(80000),
            "个人应付", // 个人支出使用公司资金构成挪用
            None,
        );
        
        assert!(result.is_ok());
        let (personal_ratio, company_ratio, behavior) = result.unwrap();
        
        // 公司余额更多，优先扣除：70000公司 + 10000个人
        assert_eq!(personal_ratio, Decimal::new(125, 3)); // 10000/80000 = 0.125
        assert_eq!(company_ratio, Decimal::new(875, 3)); // 70000/80000 = 0.875
        assert!(behavior.contains("挪用")); // 个人支出使用公司资金构成挪用
        assert_eq!(tracker.base.personal_balance, Decimal::from(20000)); // 30000-10000
        assert_eq!(tracker.base.company_balance, Decimal::ZERO); // 70000-70000
    }
    
    #[test]
    fn test_get_current_ratios() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        tracker.initialize_balance(Decimal::from(60000), "个人").unwrap();
        tracker.process_inflow(Decimal::from(40000), "公司应收", None).unwrap();
        
        let result = tracker.get_current_ratios().unwrap();
        let (personal_ratio, company_ratio) = result;
        
        assert_eq!(personal_ratio, Decimal::new(6, 1)); // 60000/100000 = 0.6
        assert_eq!(company_ratio, Decimal::new(4, 1)); // 40000/100000 = 0.4
    }
}

/// 差额计算法追踪器的公开接口
/// 
/// 提供与其他组件集成的API
impl BalanceMethodTracker {
    /// 获取内部状态（供测试和调试使用）
    #[cfg(test)]
    pub fn get_internal_state(&self) -> (&TrackerBase, &BehaviorAnalyzer) {
        (&self.base, &self.behavior_analyzer)
    }
    
    /// 更新交易记录的所有计算字段
    /// 
    /// 这个方法将当前追踪器的状态同步到Transaction结构中
    pub fn update_transaction_fields(
        &self,
        transaction: &mut Transaction,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        behavior: &str,
    ) -> AuditResult<()> {
        // 获取当前摘要状态
        let summary = self.get_summary()?;
        
        // 更新算法计算字段
        transaction.personal_ratio = Some(personal_ratio);
        transaction.company_ratio = Some(company_ratio);
        transaction.behavior_nature = Some(behavior.to_string());
        
        // 更新累计字段
        transaction.cumulative_misappropriation = Some(summary.total_misappropriation);
        transaction.cumulative_advance = Some(summary.total_advance_payment);
        transaction.cumulative_company_principal_returned = Some(summary.total_company_principal_returned);
        transaction.cumulative_personal_principal_returned = Some(summary.total_personal_principal_returned);
        transaction.cumulative_personal_profit = Some(summary.total_personal_profit);
        transaction.cumulative_company_profit = Some(summary.total_company_profit);
        
        // 更新余额字段
        transaction.personal_balance = Some(summary.personal_balance);
        transaction.company_balance = Some(summary.company_balance);
        transaction.funding_gap = Some(summary.funding_gap);
        
        // 修复时间戳格式问题：确保完整的日期时间格式
        if !transaction.transaction_time.contains('/') && !transaction.transaction_time.contains('-') {
            // 如果transaction_time只是时间部分，合并日期和时间
            transaction.transaction_time = transaction.transaction_date.format("%Y/%m/%d %H:%M:%S").to_string();
        }
        
        Ok(())
    }
}