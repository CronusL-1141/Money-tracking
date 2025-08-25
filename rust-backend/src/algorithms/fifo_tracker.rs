//! FIFO资金追踪器实现

use crate::interfaces::Tracker;
use crate::models::{Config, AuditSummary};
use crate::errors::{AuditError, AuditResult};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use std::collections::VecDeque;

/// FIFO资金追踪器
/// 
/// 实现先进先出的资金追踪算法
#[derive(Debug, Clone)]
pub struct FifoTracker {
    config: Config,
    initialized: bool,
    
    // 核心状态
    personal_balance: Decimal,
    company_balance: Decimal,
    
    // 统计信息
    total_misappropriation: Decimal,
    total_advance_payment: Decimal,
    total_company_principal_returned: Decimal,
    total_personal_principal_returned: Decimal,
    total_personal_profit: Decimal,
    total_company_profit: Decimal,
    investment_product_count: u32,
    
    // FIFO队列（简化实现）
    fund_queue: VecDeque<FundEntry>,
}

/// 资金条目（FIFO队列中的元素）
#[derive(Debug, Clone)]
struct FundEntry {
    amount: Decimal,
    fund_type: FundType,
    entry_time: Option<NaiveDateTime>,
}

/// 资金类型
#[derive(Debug, Clone, PartialEq)]
enum FundType {
    Personal,
    Company,
}

impl FifoTracker {
    /// 创建新的FIFO追踪器
    pub fn new(config: Config) -> Self {
        Self {
            config,
            initialized: false,
            personal_balance: Decimal::ZERO,
            company_balance: Decimal::ZERO,
            total_misappropriation: Decimal::ZERO,
            total_advance_payment: Decimal::ZERO,
            total_company_principal_returned: Decimal::ZERO,
            total_personal_principal_returned: Decimal::ZERO,
            total_personal_profit: Decimal::ZERO,
            total_company_profit: Decimal::ZERO,
            investment_product_count: 0,
            fund_queue: VecDeque::new(),
        }
    }
    
    /// 计算资金缺口
    fn calculate_funding_gap(&self) -> Decimal {
        self.total_misappropriation 
            - self.total_company_principal_returned
            - self.total_advance_payment
    }
    
    /// 处理个人资金流入
    fn process_personal_inflow(&mut self, amount: Decimal, date: Option<NaiveDateTime>) {
        self.personal_balance += amount;
        
        // 添加到FIFO队列
        self.fund_queue.push_back(FundEntry {
            amount,
            fund_type: FundType::Personal,
            entry_time: date,
        });
    }
    
    /// 处理公司资金流入
    fn process_company_inflow(&mut self, amount: Decimal, date: Option<NaiveDateTime>) {
        self.company_balance += amount;
        
        // 添加到FIFO队列
        self.fund_queue.push_back(FundEntry {
            amount,
            fund_type: FundType::Company,
            entry_time: date,
        });
    }
    
    /// 处理资金流出（FIFO方式）
    fn process_fifo_outflow(&mut self, amount: Decimal) -> (Decimal, Decimal, String) {
        let mut remaining_amount = amount;
        let mut personal_used = Decimal::ZERO;
        let mut company_used = Decimal::ZERO;
        
        // 从队列前端开始消费资金
        while remaining_amount > Decimal::ZERO && !self.fund_queue.is_empty() {
            if let Some(mut entry) = self.fund_queue.pop_front() {
                let used_amount = remaining_amount.min(entry.amount);
                
                match entry.fund_type {
                    FundType::Personal => {
                        personal_used += used_amount;
                        self.personal_balance -= used_amount;
                    }
                    FundType::Company => {
                        company_used += used_amount;
                        self.company_balance -= used_amount;
                    }
                }
                
                remaining_amount -= used_amount;
                
                // 如果条目还有剩余，放回队列前端
                if entry.amount > used_amount {
                    entry.amount -= used_amount;
                    self.fund_queue.push_front(entry);
                }
            } else {
                break;
            }
        }
        
        // 计算占比
        let total_used = personal_used + company_used;
        let (personal_ratio, company_ratio) = if total_used > Decimal::ZERO {
            (personal_used / total_used, company_used / total_used)
        } else {
            (Decimal::ZERO, Decimal::ZERO)
        };
        
        // 确定行为性质
        let behavior = if personal_used > Decimal::ZERO && company_used > Decimal::ZERO {
            "混合支出".to_string()
        } else if personal_used > Decimal::ZERO {
            "个人支出".to_string()
        } else if company_used > Decimal::ZERO {
            "公司支出".to_string()
        } else {
            "无效支出".to_string()
        };
        
        (personal_ratio, company_ratio, behavior)
    }
}

impl Tracker for FifoTracker {
    fn initialize_balance(&mut self, initial_balance: Decimal, balance_type: &str) -> AuditResult<()> {
        if self.config.is_personal_fund(balance_type) {
            self.personal_balance = initial_balance;
            self.fund_queue.push_back(FundEntry {
                amount: initial_balance,
                fund_type: FundType::Personal,
                entry_time: None,
            });
        } else if self.config.is_company_fund(balance_type) {
            self.company_balance = initial_balance;
            self.fund_queue.push_back(FundEntry {
                amount: initial_balance,
                fund_type: FundType::Company,
                entry_time: None,
            });
        } else {
            return Err(AuditError::tracker_init_error(
                format!("未知的余额类型: {}", balance_type)
            ));
        }
        
        self.initialized = true;
        Ok(())
    }
    
    fn process_inflow(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.initialized {
            return Err(AuditError::tracker_init_error("追踪器未初始化"));
        }
        
        if self.config.is_personal_fund(fund_attribute) {
            self.process_personal_inflow(amount, transaction_date);
            Ok((Decimal::ONE, Decimal::ZERO, "个人收入".to_string()))
        } else if self.config.is_company_fund(fund_attribute) {
            self.process_company_inflow(amount, transaction_date);
            Ok((Decimal::ZERO, Decimal::ONE, "公司收入".to_string()))
        } else {
            Err(AuditError::validation_error(
                format!("未知的资金属性: {}", fund_attribute)
            ))
        }
    }
    
    fn process_outflow(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.initialized {
            return Err(AuditError::tracker_init_error("追踪器未初始化"));
        }
        
        let (personal_ratio, company_ratio, behavior) = self.process_fifo_outflow(amount);
        
        // 根据流出目标判断是否为挪用或垫付
        let behavior_description = if self.config.is_personal_fund(fund_attribute) && company_ratio > Decimal::ZERO {
            self.total_misappropriation += amount * company_ratio;
            format!("挪用 - {}", behavior)
        } else if self.config.is_company_fund(fund_attribute) && personal_ratio > Decimal::ZERO {
            self.total_advance_payment += amount * personal_ratio;
            format!("垫付 - {}", behavior)
        } else {
            behavior
        };
        
        Ok((personal_ratio, company_ratio, behavior_description))
    }
    
    fn process_investment_redemption(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
        _transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.initialized {
            return Err(AuditError::tracker_init_error("追踪器未初始化"));
        }
        
        // 投资赎回的处理逻辑
        let (personal_ratio, company_ratio, _) = self.process_fifo_outflow(amount);
        
        // 简化处理：假设赎回金额按原投资比例分配
        if personal_ratio > Decimal::ZERO {
            self.personal_balance += amount * personal_ratio;
        }
        if company_ratio > Decimal::ZERO {
            self.company_balance += amount * company_ratio;
        }
        
        self.investment_product_count += 1;
        
        Ok((personal_ratio, company_ratio, format!("投资赎回 - {}", fund_attribute)))
    }
    
    fn get_summary(&self) -> AuditResult<AuditSummary> {
        Ok(AuditSummary {
            personal_balance: self.personal_balance,
            company_balance: self.company_balance,
            total_misappropriation: self.total_misappropriation,
            total_advance_payment: self.total_advance_payment,
            total_company_principal_returned: self.total_company_principal_returned,
            total_personal_principal_returned: self.total_personal_principal_returned,
            total_personal_profit: self.total_personal_profit,
            total_company_profit: self.total_company_profit,
            funding_gap: self.calculate_funding_gap(),
            investment_product_count: self.investment_product_count,
            total_balance: self.personal_balance + self.company_balance,
        })
    }
    
    fn get_current_ratios(&self) -> AuditResult<(Decimal, Decimal)> {
        let total_balance = self.personal_balance + self.company_balance;
        if total_balance > Decimal::ZERO {
            Ok((
                self.personal_balance / total_balance,
                self.company_balance / total_balance,
            ))
        } else {
            Ok((Decimal::ZERO, Decimal::ZERO))
        }
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    fn get_name(&self) -> &'static str {
        "FIFO"
    }
    
    fn get_description(&self) -> &'static str {
        "先进先出算法 - 按时间顺序处理资金流动"
    }
    
    fn reset(&mut self) -> AuditResult<()> {
        *self = Self::new(self.config.clone());
        Ok(())
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