//! FIFO资金追踪器实现
//!
//! 基于新共享架构实现，使用TrackerBase作为基础状态管理
//! 对应Python版本的FIFO资金追踪器完整功能

use super::shared::{
    TrackerBase, BehaviorAnalyzer, InvestmentPoolManager, FundFlowCommon, SummaryGenerator
};
use crate::data_models::{Config, AuditSummary, Transaction};
use crate::errors::{AuditError, AuditResult};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use std::collections::VecDeque;

/// FIFO资金追踪器
/// 
/// 基于共享架构实现先进先出的资金追踪算法
/// 对应Python版本的FIFO资金追踪器
#[derive(Debug, Clone)]
pub struct FifoTracker {
    /// 共享基础状态（13个状态变量）
    base: TrackerBase,
    /// 行为分析器（挪用垫付分析）
    behavior_analyzer: BehaviorAnalyzer,
    
    // === FIFO特有的数据结构 ===
    /// 资金流入队列 - 对应Python版本的deque结构
    /// 元素格式: (金额, 类型, 时间)
    fund_inflow_queue: VecDeque<FundEntry>,
}

/// 资金条目（FIFO队列中的元素）
/// 对应Python版本的(金额, 类型, 时间)元组结构
#[derive(Debug, Clone)]
struct FundEntry {
    /// 资金金额
    amount: Decimal,
    /// 资金类型（"个人" 或 "公司"）
    fund_type: String,
    /// 流入时间
    entry_time: Option<NaiveDateTime>,
}

impl FifoTracker {
    /// 创建新的FIFO追踪器
    /// 对应Python版本的__init__方法
    pub fn new(config: Config) -> Self {
        Self {
            base: TrackerBase::new(config),
            behavior_analyzer: BehaviorAnalyzer::new(),
            fund_inflow_queue: VecDeque::new(),
        }
    }
    
    /// 初始化余额
    /// 对应Python版本的初始化余额处理
    pub fn initialize_balance(&mut self, initial_balance: Decimal, balance_type: &str) -> AuditResult<()> {
        // 使用基础类初始化
        self.base.initialize_balance(initial_balance, balance_type)?;
        
        // 添加到FIFO队列
        self.fund_inflow_queue.push_back(FundEntry {
            amount: initial_balance,
            fund_type: balance_type.to_string(),
            entry_time: None,
        });
        
        Ok(())
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
        
        // 添加到FIFO队列（按实际分配金额）
        if personal_ratio > Decimal::ZERO {
            self.fund_inflow_queue.push_back(FundEntry {
                amount: amount * personal_ratio,
                fund_type: "个人".to_string(),
                entry_time: transaction_date,
            });
        }
        if company_ratio > Decimal::ZERO {
            self.fund_inflow_queue.push_back(FundEntry {
                amount: amount * company_ratio,
                fund_type: "公司".to_string(),
                entry_time: transaction_date,
            });
        }
        
        Ok((personal_ratio, company_ratio, behavior))
    }
    
    /// FIFO资金扣除函数
    /// 对应Python版本的FIFO队列扣除逻辑
    fn fifo_deduction(&mut self, amount: Decimal) -> (Decimal, Decimal) {
        let mut remaining_amount = amount;
        let mut personal_deducted = Decimal::ZERO;
        let mut company_deducted = Decimal::ZERO;
        
        // 从队列前端开始消费资金
        while remaining_amount > Decimal::ZERO && !self.fund_inflow_queue.is_empty() {
            if let Some(mut entry) = self.fund_inflow_queue.pop_front() {
                let used_amount = remaining_amount.min(entry.amount);
                
                if self.base.config.is_personal_fund(&entry.fund_type) {
                    personal_deducted += used_amount;
                } else if self.base.config.is_company_fund(&entry.fund_type) {
                    company_deducted += used_amount;
                }
                
                remaining_amount -= used_amount;
                
                // 如果条目还有剩余，放回队列前端
                if entry.amount > used_amount {
                    entry.amount -= used_amount;
                    self.fund_inflow_queue.push_front(entry);
                }
            } else {
                break;
            }
        }
        
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
        
        // 使用FIFO扣除函数
        let (personal_deduction, company_deduction) = self.fifo_deduction(amount);
        
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
        
        // 使用共同投资处理逻辑，传入FIFO扣除函数
        let result = FundFlowCommon::process_investment_purchase(
            &mut self.base,
            &self.behavior_analyzer,
            amount,
            fund_attribute,
            transaction_date,
            |base, amount| {
                // FIFO扣除逻辑的闭包版本
                let mut temp_tracker = FifoTracker {
                    base: base.clone(),
                    behavior_analyzer: BehaviorAnalyzer::new(),
                    fund_inflow_queue: self.fund_inflow_queue.clone(),
                };
                let (personal, company) = temp_tracker.fifo_deduction(amount);
                // 更新原始base状态
                base.personal_balance = temp_tracker.base.personal_balance;
                base.company_balance = temp_tracker.base.company_balance;
                base.update_total_balance();
                // 更新队列状态
                self.fund_inflow_queue = temp_tracker.fund_inflow_queue;
                (personal, company)
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
            Ok((personal_ratio, company_ratio, behavior)) => {
                // 赎回金额重新进入FIFO队列
                if personal_ratio > Decimal::ZERO {
                    self.fund_inflow_queue.push_back(FundEntry {
                        amount: amount * personal_ratio,
                        fund_type: "个人".to_string(),
                        entry_time: transaction_date,
                    });
                }
                if company_ratio > Decimal::ZERO {
                    self.fund_inflow_queue.push_back(FundEntry {
                        amount: amount * company_ratio,
                        fund_type: "公司".to_string(),
                        entry_time: transaction_date,
                    });
                }
                
                Ok((personal_ratio, company_ratio, behavior))
            }
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
        "FIFO"
    }
    
    /// 获取算法描述
    pub fn get_description(&self) -> &'static str {
        "先进先出算法 - 按资金流入时间顺序进行扣除"
    }
    
    /// 重置追踪器状态
    pub fn reset(&mut self) -> AuditResult<()> {
        self.base.reset();
        self.behavior_analyzer = BehaviorAnalyzer::new();
        self.fund_inflow_queue.clear();
        Ok(())
    }
    
    /// 生成详细的摘要文本
    pub fn generate_detailed_summary_text(&self) -> String {
        SummaryGenerator::generate_detailed_summary_text(&self.base, "FIFO")
    }
    
    /// 获取场外资金池记录管理器
    pub fn get_offsite_pool_records(&self) -> &crate::data_models::OffsitePoolRecordManager {
        &self.base.offsite_pool_records
    }
    
    /// 获取投资池数据（用于完整统计计算）
    pub fn get_investment_pools(&self) -> &std::collections::HashMap<String, crate::algorithms::shared::tracker_base::InvestmentPool> {
        &self.base.investment_pools
    }
    
    /// 获取base引用（用于访问场外资金池记录）
    pub fn get_base(&self) -> &TrackerBase {
        &self.base
    }
    
    /// 获取FIFO队列状态（用于调试）
    pub fn get_queue_info(&self) -> String {
        if self.fund_inflow_queue.is_empty() {
            "FIFO队列为空".to_string()
        } else {
            let mut info = Vec::new();
            info.push(format!("FIFO队列长度: {}", self.fund_inflow_queue.len()));
            for (i, entry) in self.fund_inflow_queue.iter().enumerate() {
                info.push(format!(
                    "  [{}] {}: ¥{:.2} ({})",
                    i,
                    entry.fund_type,
                    entry.amount,
                    entry.entry_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or("未知时间".to_string())
                ));
            }
            info.join("\n")
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fifo_tracker_creation() {
        let config = Config::new();
        let tracker = FifoTracker::new(config);
        
        assert_eq!(tracker.get_name(), "FIFO");
        assert!(!tracker.is_initialized());
        assert_eq!(tracker.personal_balance, Decimal::ZERO);
        assert_eq!(tracker.company_balance, Decimal::ZERO);
    }
    
    #[test]
    fn test_initialization() {
        let config = Config::new();
        let mut tracker = FifoTracker::new(config);
        
        let result = tracker.initialize_balance(Decimal::from(100000), "个人");
        assert!(result.is_ok());
        assert!(tracker.is_initialized());
        assert_eq!(tracker.personal_balance, Decimal::from(100000));
    }
    
    #[test]
    fn test_process_inflow() {
        let config = Config::new();
        let mut tracker = FifoTracker::new(config);
        
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
        assert_eq!(behavior, "个人收入");
        assert_eq!(tracker.personal_balance, Decimal::from(80000));
    }
    
    #[test]
    fn test_process_outflow() {
        let config = Config::new();
        let mut tracker = FifoTracker::new(config);
        
        // 初始化混合余额
        tracker.initialize_balance(Decimal::from(50000), "个人").unwrap();
        tracker.process_inflow(Decimal::from(30000), "公司应收", None).unwrap();
        
        let result = tracker.process_outflow(
            Decimal::from(20000),
            "个人应付",
            None,
        );
        
        assert!(result.is_ok());
        let (personal_ratio, company_ratio, behavior) = result.unwrap();
        // 应该优先使用个人资金（FIFO）
        assert!(personal_ratio > Decimal::ZERO);
        assert!(behavior.contains("个人支出") || behavior.contains("挪用"));
    }
}

/// FIFO资金追踪器的公开接口
/// 
/// 提供与其他组件集成的API
impl FifoTracker {
    /// 获取内部状态（供测试和调试使用）
    #[cfg(test)]
    pub fn get_internal_state(&self) -> (&TrackerBase, &BehaviorAnalyzer, &VecDeque<FundEntry>) {
        (&self.base, &self.behavior_analyzer, &self.fund_inflow_queue)
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