//! 追踪器共享基础类
//! 
//! 包含FIFO和差额计算法共同的13个状态变量和基础功能
//! 对应Python版本的FIFO资金追踪器核心状态管理

use crate::data_models::{Config, AuditSummary, OffsitePoolRecordManager};
use crate::errors::{AuditResult, AuditError};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 追踪器共享基础类
/// 
/// 包含两种算法共同的状态变量和基础功能
#[derive(Debug, Clone)]
pub struct TrackerBase {
    /// 配置对象
    pub config: Config,
    /// 是否已初始化
    pub initialized: bool,
    
    // === 基础余额状态 ===
    /// 个人资金池余额
    pub personal_balance: Decimal,
    /// 公司资金池余额  
    pub company_balance: Decimal,
    
    // === 9个累计统计状态变量 ===
    /// 累计挪用金额（个人使用公司资金，包括投资挪用）
    pub total_misappropriation: Decimal,
    /// 累计垫付金额（公司使用个人资金）
    pub total_advance_payment: Decimal,
    /// 累计由资金池回归公司余额本金（通过投资产品赎回归还的公司本金）
    pub total_company_principal_returned: Decimal,
    /// 累计由资金池回归个人余额本金（通过投资产品赎回归还的个人本金）
    pub total_personal_principal_returned: Decimal,
    /// 累计非法所得（投资收益中的非法部分）
    pub total_illegal_gain: Decimal,
    /// 总计个人应分配利润
    pub total_personal_profit: Decimal,
    /// 总计公司应分配利润  
    pub total_company_profit: Decimal,
    /// 投资产品数量统计
    pub investment_product_count: u32,
    /// 总余额（个人余额 + 公司余额）
    pub total_balance: Decimal,
    
    // === 投资产品资金池管理 ===
    /// 投资产品资金池字典 - 对应Python的复杂10字段结构
    pub investment_pools: HashMap<String, InvestmentPool>,
    /// 场外资金池记录管理器 - 对应Python的场外资金池记录
    pub offsite_pool_records: OffsitePoolRecordManager,
    
    // === 行为分析器增量管理 ===
    /// 上次行为分析器挪用金额（用于增量计算）
    pub last_analyzer_misappropriation: Decimal,
    /// 上次行为分析器垫付金额（用于增量计算）
    pub last_analyzer_advance_payment: Decimal,
}

/// 投资产品资金池
/// 对应Python版本的10字段复杂数据结构
#[derive(Debug, Clone)]
pub struct InvestmentPool {
    /// 个人投入资金
    pub personal_amount: Decimal,
    /// 公司投入资金
    pub company_amount: Decimal,
    /// 当前总金额（可为负数=盈利）
    pub total_amount: Decimal,
    /// 累计申购总额
    pub cumulative_purchase: Decimal,
    /// 累计赎回总额
    pub cumulative_redemption: Decimal,
    /// 最新个人占比（动态计算）
    pub latest_personal_ratio: Decimal,
    /// 最新公司占比（动态计算）
    pub latest_company_ratio: Decimal,
    /// 锁定的个人占比（资金池变负数时锁定）
    pub locked_personal_ratio: Option<Decimal>,
    /// 锁定的公司占比（资金池变负数时锁定）
    pub locked_company_ratio: Option<Decimal>,
    /// 历史盈利记录（每次重置的盈利记录）
    pub historical_profit_records: Vec<ProfitRecord>,
    /// 累计已实现盈利（所有重置盈利的累计）
    pub cumulative_realized_profit: Decimal,
}

/// 盈利记录
#[derive(Debug, Clone)]
pub struct ProfitRecord {
    /// 重置时间
    pub reset_time: String,
    /// 盈利金额
    pub profit_amount: Decimal,
    /// 描述信息
    pub description: String,
}

impl TrackerBase {
    /// 创建新的追踪器基础
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
            total_illegal_gain: Decimal::ZERO,
            total_personal_profit: Decimal::ZERO,
            total_company_profit: Decimal::ZERO,
            investment_product_count: 0,
            total_balance: Decimal::ZERO,
            investment_pools: HashMap::new(),
            offsite_pool_records: OffsitePoolRecordManager::new(),
            last_analyzer_misappropriation: Decimal::ZERO,
            last_analyzer_advance_payment: Decimal::ZERO,
        }
    }
    
    /// 初始化余额
    pub fn initialize_balance(&mut self, initial_balance: Decimal, balance_type: &str) -> AuditResult<()> {
        if self.config.is_personal_fund(balance_type) {
            self.personal_balance = initial_balance;
        } else if self.config.is_company_fund(balance_type) {
            self.company_balance = initial_balance;
        } else {
            return Err(AuditError::validation_error(
                format!("未知的余额类型: {}", balance_type)
            ));
        }
        
        self.update_total_balance();
        self.initialized = true;
        Ok(())
    }
    
    /// 更新总余额
    pub fn update_total_balance(&mut self) {
        self.total_balance = self.personal_balance + self.company_balance;
    }
    
    /// 格式化数值（避免科学计数法）
    pub fn format_decimal(&self, value: Decimal) -> Decimal {
        // 处理极小值，避免科学计数法显示
        if value.abs() < Decimal::new(1, 10) { // 小于0.0000000001
            Decimal::ZERO
        } else {
            value.round_dp(2) // 保留2位小数
        }
    }
    
    /// 计算资金缺口
    pub fn calculate_funding_gap(&self) -> Decimal {
        self.total_misappropriation 
            - self.total_company_principal_returned
            - self.total_advance_payment
    }
    
    /// 获取当前余额比例
    pub fn get_current_ratios(&self) -> (Decimal, Decimal) {
        if self.total_balance > Decimal::ZERO {
            (
                self.personal_balance / self.total_balance,
                self.company_balance / self.total_balance,
            )
        } else {
            (Decimal::ZERO, Decimal::ZERO)
        }
    }
    
    /// 生成审计摘要
    pub fn get_audit_summary(&self) -> AuditSummary {
        AuditSummary {
            personal_balance: self.format_decimal(self.personal_balance),
            company_balance: self.format_decimal(self.company_balance),
            total_misappropriation: self.format_decimal(self.total_misappropriation),
            total_advance_payment: self.format_decimal(self.total_advance_payment),
            total_company_principal_returned: self.format_decimal(self.total_company_principal_returned),
            total_personal_principal_returned: self.format_decimal(self.total_personal_principal_returned),
            total_personal_profit: self.format_decimal(self.total_personal_profit),
            total_company_profit: self.format_decimal(self.total_company_profit),
            funding_gap: self.format_decimal(self.calculate_funding_gap()),
            investment_product_count: self.investment_product_count,
            total_balance: self.format_decimal(self.total_balance),
        }
    }
    
    /// 处理行为分析器增量累计
    /// 对应Python版本的增量管理机制
    pub fn process_analyzer_incremental(&mut self, analyzer_misappropriation: Decimal, analyzer_advance_payment: Decimal) {
        let misappropriation_increment = analyzer_misappropriation - self.last_analyzer_misappropriation;
        let advance_payment_increment = analyzer_advance_payment - self.last_analyzer_advance_payment;
        
        self.total_misappropriation += misappropriation_increment;
        self.total_advance_payment += advance_payment_increment;
        
        // 更新记录的累计值
        self.last_analyzer_misappropriation = analyzer_misappropriation;
        self.last_analyzer_advance_payment = analyzer_advance_payment;
        
        // 格式化数值
        self.total_misappropriation = self.format_decimal(self.total_misappropriation);
        self.total_advance_payment = self.format_decimal(self.total_advance_payment);
    }
    
    /// 重置追踪器状态
    pub fn reset(&mut self) {
        *self = Self::new(self.config.clone());
    }
    
    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for InvestmentPool {
    fn default() -> Self {
        Self {
            personal_amount: Decimal::ZERO,
            company_amount: Decimal::ZERO,
            total_amount: Decimal::ZERO,
            cumulative_purchase: Decimal::ZERO,
            cumulative_redemption: Decimal::ZERO,
            latest_personal_ratio: Decimal::ZERO,
            latest_company_ratio: Decimal::ZERO,
            locked_personal_ratio: None,
            locked_company_ratio: None,
            historical_profit_records: Vec::new(),
            cumulative_realized_profit: Decimal::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tracker_base_creation() {
        let config = Config::new();
        let base = TrackerBase::new(config);
        
        assert!(!base.is_initialized());
        assert_eq!(base.personal_balance, Decimal::ZERO);
        assert_eq!(base.company_balance, Decimal::ZERO);
        assert_eq!(base.total_misappropriation, Decimal::ZERO);
    }
    
    #[test]
    fn test_initialize_balance() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);
        
        let result = base.initialize_balance(Decimal::from(100000), "个人");
        assert!(result.is_ok());
        assert!(base.is_initialized());
        assert_eq!(base.personal_balance, Decimal::from(100000));
        assert_eq!(base.total_balance, Decimal::from(100000));
    }
    
    #[test]
    fn test_format_decimal() {
        let config = Config::new();
        let base = TrackerBase::new(config);
        
        // 测试极小值处理
        let small_value = Decimal::new(1, 11); // 0.00000000001
        assert_eq!(base.format_decimal(small_value), Decimal::ZERO);
        
        // 测试正常值
        let normal_value = Decimal::new(123456, 2); // 1234.56
        assert_eq!(base.format_decimal(normal_value), Decimal::new(123456, 2));
    }
    
    #[test]
    fn test_funding_gap_calculation() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);
        
        base.total_misappropriation = Decimal::from(10000);
        base.total_company_principal_returned = Decimal::from(3000);
        base.total_advance_payment = Decimal::from(2000);
        
        assert_eq!(base.calculate_funding_gap(), Decimal::from(5000));
    }
    
    #[test]
    fn test_current_ratios() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);
        
        base.personal_balance = Decimal::from(6000);
        base.company_balance = Decimal::from(4000);
        base.update_total_balance();
        
        let (personal_ratio, company_ratio) = base.get_current_ratios();
        assert_eq!(personal_ratio, Decimal::new(6, 1)); // 0.6
        assert_eq!(company_ratio, Decimal::new(4, 1)); // 0.4
    }
}