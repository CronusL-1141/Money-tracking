//! 差额计算法追踪器实现

use crate::interfaces::Tracker;
use crate::models::{Config, AuditSummary};
use crate::errors::{AuditError, AuditResult};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

/// 差额计算法追踪器
/// 
/// 实现差额计算法的资金追踪算法
/// 特点：个人余额优先扣除，简化计算逻辑
#[derive(Debug, Clone)]
pub struct BalanceMethodTracker {
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
}

impl BalanceMethodTracker {
    /// 创建新的差额计算法追踪器
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
        }
    }
    
    /// 计算资金缺口
    fn calculate_funding_gap(&self) -> Decimal {
        self.total_misappropriation 
            - self.total_company_principal_returned
            - self.total_advance_payment
    }
    
    /// 处理差额计算法的资金流出
    /// 
    /// 算法逻辑：
    /// 1. 如果是个人支出，优先从个人余额扣除
    /// 2. 个人余额不足时，从公司余额扣除（构成挪用）
    /// 3. 如果是公司支出，优先从公司余额扣除
    /// 4. 公司余额不足时，从个人余额扣除（构成垫付）
    fn process_balance_method_outflow(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
    ) -> (Decimal, Decimal, String) {
        let mut personal_used = Decimal::ZERO;
        let mut company_used = Decimal::ZERO;
        let mut behavior = String::new();
        
        if self.config.is_personal_fund(fund_attribute) {
            // 个人支出
            let personal_available = self.personal_balance.min(amount);
            personal_used = personal_available;
            self.personal_balance -= personal_available;
            
            let remaining = amount - personal_available;
            if remaining > Decimal::ZERO {
                // 个人余额不足，从公司余额扣除（挪用）
                let company_available = self.company_balance.min(remaining);
                company_used = company_available;
                self.company_balance -= company_available;
                
                // 记录挪用
                self.total_misappropriation += company_used;
                
                behavior = if personal_used > Decimal::ZERO {
                    "个人支出+挪用".to_string()
                } else {
                    "挪用".to_string()
                };
            } else {
                behavior = "个人支出".to_string();
            }
        } else if self.config.is_company_fund(fund_attribute) {
            // 公司支出
            let company_available = self.company_balance.min(amount);
            company_used = company_available;
            self.company_balance -= company_available;
            
            let remaining = amount - company_available;
            if remaining > Decimal::ZERO {
                // 公司余额不足，从个人余额扣除（垫付）
                let personal_available = self.personal_balance.min(remaining);
                personal_used = personal_available;
                self.personal_balance -= personal_available;
                
                // 记录垫付
                self.total_advance_payment += personal_used;
                
                behavior = if company_used > Decimal::ZERO {
                    "公司支出+垫付".to_string()
                } else {
                    "垫付".to_string()
                };
            } else {
                behavior = "公司支出".to_string();
            }
        } else {
            // 未知资金属性，按总余额比例分配
            let total_balance = self.personal_balance + self.company_balance;
            if total_balance > Decimal::ZERO {
                let personal_ratio = self.personal_balance / total_balance;
                personal_used = (amount * personal_ratio).min(self.personal_balance);
                company_used = amount - personal_used;
                company_used = company_used.min(self.company_balance);
                
                self.personal_balance -= personal_used;
                self.company_balance -= company_used;
                
                behavior = "比例分配支出".to_string();
            } else {
                behavior = "余额不足".to_string();
            }
        }
        
        // 计算使用比例
        let total_used = personal_used + company_used;
        let (personal_ratio, company_ratio) = if total_used > Decimal::ZERO {
            (personal_used / total_used, company_used / total_used)
        } else {
            (Decimal::ZERO, Decimal::ZERO)
        };
        
        (personal_ratio, company_ratio, behavior)
    }
    
    /// 处理投资产品赎回（差额计算法）
    fn process_investment_redemption_balance_method(
        &mut self,
        amount: Decimal,
        fund_attribute: &str,
    ) -> (Decimal, Decimal, String) {
        // 简化处理：按当前余额比例分配赎回收益
        let total_balance = self.personal_balance + self.company_balance;
        
        if total_balance > Decimal::ZERO {
            let personal_ratio = self.personal_balance / total_balance;
            let company_ratio = self.company_balance / total_balance;
            
            let personal_share = amount * personal_ratio;
            let company_share = amount * company_ratio;
            
            self.personal_balance += personal_share;
            self.company_balance += company_share;
            
            // 记录投资收益
            self.total_personal_profit += personal_share;
            self.total_company_profit += company_share;
            
            self.investment_product_count += 1;
            
            (personal_ratio, company_ratio, format!("投资赎回 - {}", fund_attribute))
        } else {
            // 余额为零时，全部分配给个人（或根据具体业务规则）
            self.personal_balance += amount;
            self.total_personal_profit += amount;
            self.investment_product_count += 1;
            
            (Decimal::ONE, Decimal::ZERO, format!("投资赎回 - {}", fund_attribute))
        }
    }
}

impl Tracker for BalanceMethodTracker {
    fn initialize_balance(&mut self, initial_balance: Decimal, balance_type: &str) -> AuditResult<()> {
        if self.config.is_personal_fund(balance_type) {
            self.personal_balance = initial_balance;
        } else if self.config.is_company_fund(balance_type) {
            self.company_balance = initial_balance;
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
        _transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.initialized {
            return Err(AuditError::tracker_init_error("追踪器未初始化"));
        }
        
        if self.config.is_personal_fund(fund_attribute) {
            self.personal_balance += amount;
            Ok((Decimal::ONE, Decimal::ZERO, "个人收入".to_string()))
        } else if self.config.is_company_fund(fund_attribute) {
            self.company_balance += amount;
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
        _transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<(Decimal, Decimal, String)> {
        if !self.initialized {
            return Err(AuditError::tracker_init_error("追踪器未初始化"));
        }
        
        let result = self.process_balance_method_outflow(amount, fund_attribute);
        Ok(result)
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
        
        let result = self.process_investment_redemption_balance_method(amount, fund_attribute);
        Ok(result)
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
        "BALANCE_METHOD"
    }
    
    fn get_description(&self) -> &'static str {
        "差额计算法 - 个人优先扣除的简化算法"
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
    fn test_balance_method_tracker_creation() {
        let config = Config::new();
        let tracker = BalanceMethodTracker::new(config);
        
        assert_eq!(tracker.get_name(), "BALANCE_METHOD");
        assert!(!tracker.is_initialized());
        assert_eq!(tracker.personal_balance, Decimal::ZERO);
        assert_eq!(tracker.company_balance, Decimal::ZERO);
    }
    
    #[test]
    fn test_initialization() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        let result = tracker.initialize_balance(Decimal::from(100000), "个人");
        assert!(result.is_ok());
        assert!(tracker.is_initialized());
        assert_eq!(tracker.personal_balance, Decimal::from(100000));
    }
    
    #[test]
    fn test_personal_outflow_with_sufficient_balance() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        tracker.initialize_balance(Decimal::from(100000), "个人").unwrap();
        
        let result = tracker.process_outflow(
            Decimal::from(30000),
            "个人应付",
            None,
        );
        
        assert!(result.is_ok());
        let (personal_ratio, company_ratio, behavior) = result.unwrap();
        assert_eq!(personal_ratio, Decimal::ONE);
        assert_eq!(company_ratio, Decimal::ZERO);
        assert_eq!(behavior, "个人支出");
        assert_eq!(tracker.personal_balance, Decimal::from(70000));
    }
    
    #[test]
    fn test_personal_outflow_with_insufficient_balance() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        // 设置初始余额：个人50000，公司30000
        tracker.initialize_balance(Decimal::from(50000), "个人").unwrap();
        tracker.process_inflow(Decimal::from(30000), "公司应收", None).unwrap();
        
        // 个人支出80000（超过个人余额）
        let result = tracker.process_outflow(
            Decimal::from(80000),
            "个人应付",
            None,
        );
        
        assert!(result.is_ok());
        let (personal_ratio, company_ratio, behavior) = result.unwrap();
        
        // 应该是个人50000 + 公司30000 = 总计80000
        assert!(personal_ratio > Decimal::ZERO);
        assert!(company_ratio > Decimal::ZERO);
        assert!(behavior.contains("挪用"));
        assert!(tracker.total_misappropriation > Decimal::ZERO);
    }
    
    #[test]
    fn test_company_outflow_with_advance_payment() {
        let config = Config::new();
        let mut tracker = BalanceMethodTracker::new(config);
        
        // 设置初始余额：个人80000，公司20000
        tracker.initialize_balance(Decimal::from(80000), "个人").unwrap();
        tracker.process_inflow(Decimal::from(20000), "公司应收", None).unwrap();
        
        // 公司支出50000（超过公司余额）
        let result = tracker.process_outflow(
            Decimal::from(50000),
            "公司应付",
            None,
        );
        
        assert!(result.is_ok());
        let (personal_ratio, company_ratio, behavior) = result.unwrap();
        
        // 应该是公司20000 + 个人30000 = 总计50000
        assert!(personal_ratio > Decimal::ZERO);
        assert!(company_ratio > Decimal::ZERO);
        assert!(behavior.contains("垫付"));
        assert!(tracker.total_advance_payment > Decimal::ZERO);
    }
}