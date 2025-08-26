//! 投资产品业务服务
//! 
//! 处理投资产品的申购、赎回等业务逻辑，可被FIFO和差额计算法复用

use crate::errors::{AuditError, AuditResult};
use crate::data_models::{Config, investment::{InvestmentPool, InvestmentTransactionRecord, ProfitRecord, RedemptionResult}};
use rust_decimal::Decimal;
use chrono::NaiveDateTime;
use std::collections::HashMap;
use log::{info, warn};

/// 投资产品服务
/// 
/// 提供投资产品相关的所有业务逻辑，包括申购、赎回、统计等
#[derive(Debug, Clone)]
pub struct InvestmentService {
    /// 投资产品资金池
    pools: HashMap<String, InvestmentPool>,
    
    /// 交易记录
    transaction_records: Vec<InvestmentTransactionRecord>,
}

impl InvestmentService {
    /// 创建新的投资服务
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            transaction_records: Vec::new(),
        }
    }
    
    /// 处理投资产品申购
    /// 
    /// 更新投资产品资金池状态，记录申购信息
    pub fn process_purchase(
        &mut self,
        product_id: &str,
        amount: Decimal,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<()> {
        // 确保产品存在
        if !self.pools.contains_key(product_id) {
            self.create_pool(product_id);
        }
        
        let personal_amount = amount * personal_ratio;
        let company_amount = amount * company_ratio;
        
        // 处理盈利重置
        let needs_reset = {
            let pool = self.pools.get(product_id).unwrap();
            pool.total_amount < Decimal::ZERO
        };
        
        if needs_reset {
            let pool = self.pools.get_mut(product_id).unwrap();
            Self::handle_profit_reset(pool, transaction_date)?;
        }
        
        // 更新资金池
        {
            let pool = self.pools.get_mut(product_id).unwrap();
            Self::update_pool_on_purchase(pool, amount, personal_amount, company_amount, personal_ratio, company_ratio);
        }
        
        // 记录交易（复制必要数据避免借用冲突）
        let pool_data = {
            let pool = self.pools.get(product_id).unwrap();
            (pool.total_amount, pool.personal_amount, pool.company_amount, 
             pool.latest_personal_ratio, pool.latest_company_ratio,
             pool.total_purchase, pool.total_redemption)
        };
        
        self.record_purchase_transaction_with_data(
            product_id, pool_data, amount, personal_amount, company_amount, transaction_date
        );
        
        info!("处理投资申购: {} 金额: {:.2} 个人: {:.2} 公司: {:.2}", 
              product_id, amount, personal_amount, company_amount);
        
        Ok(())
    }
    
    /// 处理投资产品赎回
    /// 
    /// 按照投资占比计算赎回分配，区分本金和收益
    pub fn process_redemption(
        &mut self,
        product_id: &str,
        amount: Decimal,
        transaction_date: Option<NaiveDateTime>,
    ) -> AuditResult<RedemptionResult> {
        if !self.pools.contains_key(product_id) {
            return Err(AuditError::validation_error(
                format!("投资产品{}不存在申购记录", product_id)
            ));
        }
        
        // 检查占比有效性
        {
            let pool = self.pools.get(product_id).unwrap();
            if !pool.has_valid_ratios() {
                return Err(AuditError::validation_error(
                    format!("投资产品{}从未有过有效资金池，无法分配收益", product_id)
                ));
            }
        }
        
        // 计算赎回分配
        let result = {
            let pool = self.pools.get(product_id).unwrap();
            self.calculate_redemption_distribution(pool, amount, product_id)?
        };
        
        // 更新资金池状态
        {
            let pool = self.pools.get_mut(product_id).unwrap();
            Self::update_pool_on_redemption(pool, &result, amount);
        }
        
        // 记录交易（复制必要数据避免借用冲突）
        let pool_data = {
            let pool = self.pools.get(product_id).unwrap();
            (pool.total_amount, pool.personal_amount, pool.company_amount, 
             pool.latest_personal_ratio, pool.latest_company_ratio,
             pool.total_purchase, pool.total_redemption)
        };
        
        self.record_redemption_transaction_with_data(
            product_id, pool_data, &result, amount, transaction_date
        );
        
        info!("处理投资赎回: {} 金额: {:.2} 个人: {:.2} 公司: {:.2} 收益: {:.2}", 
              product_id, amount, result.personal_return, result.company_return, result.profit);
        
        Ok(result)
    }
    
    /// 获取投资产品池信息
    pub fn get_pool(&self, product_id: &str) -> Option<&InvestmentPool> {
        self.pools.get(product_id)
    }
    
    /// 获取所有投资产品
    pub fn get_all_pools(&self) -> &HashMap<String, InvestmentPool> {
        &self.pools
    }
    
    /// 获取交易记录
    pub fn get_transaction_records(&self) -> &[InvestmentTransactionRecord] {
        &self.transaction_records
    }
    
    /// 获取统计信息
    pub fn get_statistics(&self, config: &Config) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        
        let total_products = self.pools.len();
        let total_investment: Decimal = self.pools.values().map(|p| p.total_amount).sum();
        let total_purchase: Decimal = self.pools.values().map(|p| p.total_purchase).sum();
        let total_redemption: Decimal = self.pools.values().map(|p| p.total_redemption).sum();
        
        stats.insert("产品总数".to_string(), total_products.to_string());
        stats.insert("总投资金额".to_string(), config.format_number(total_investment).to_string());
        stats.insert("累计申购".to_string(), config.format_number(total_purchase).to_string());
        stats.insert("累计赎回".to_string(), config.format_number(total_redemption).to_string());
        stats.insert("净投资".to_string(), config.format_number(total_purchase - total_redemption).to_string());
        
        stats
    }
    
    /// 验证产品数据一致性
    pub fn validate_product_data(&self, product_id: &str, config: &Config) -> Vec<String> {
        if let Some(pool) = self.pools.get(product_id) {
            let mut errors = Vec::new();
            
            // 检查总金额计算
            let calculated_total = pool.personal_amount + pool.company_amount;
            if !config.is_balance_within_tolerance(calculated_total, pool.total_amount) {
                errors.push(format!(
                    "总金额计算错误：个人金额({:.2}) + 公司金额({:.2}) = {:.2} ≠ 总金额({:.2})",
                    pool.personal_amount, pool.company_amount, calculated_total, pool.total_amount
                ));
            }
            
            // 检查占比计算
            if pool.total_amount > Decimal::ZERO {
                let expected_personal_ratio = pool.personal_amount / pool.total_amount;
                let expected_company_ratio = pool.company_amount / pool.total_amount;
                
                if !config.is_balance_within_tolerance(expected_personal_ratio, pool.latest_personal_ratio) {
                    errors.push(format!(
                        "个人占比计算错误：期望 {:.4}，实际 {:.4}",
                        expected_personal_ratio, pool.latest_personal_ratio
                    ));
                }
                
                if !config.is_balance_within_tolerance(expected_company_ratio, pool.latest_company_ratio) {
                    errors.push(format!(
                        "公司占比计算错误：期望 {:.4}，实际 {:.4}",
                        expected_company_ratio, pool.latest_company_ratio
                    ));
                }
            }
            
            errors
        } else {
            vec![format!("投资产品{}不存在", product_id)]
        }
    }
    
    /// 清理无效产品
    pub fn cleanup_invalid_products(&mut self, config: &Config) -> usize {
        let mut to_remove = Vec::new();
        
        for (product_id, pool) in &self.pools {
            if pool.total_amount.abs() < config.numeric.minimum_amount &&
               pool.total_purchase == Decimal::ZERO &&
               pool.total_redemption == Decimal::ZERO {
                to_remove.push(product_id.clone());
            }
        }
        
        let cleanup_count = to_remove.len();
        for product_id in to_remove {
            self.pools.remove(&product_id);
            info!("清理无效投资产品: {}", product_id);
        }
        
        cleanup_count
    }
    
    // === 私有辅助方法 ===
    
    /// 创建新的投资产品池
    fn create_pool(&mut self, product_id: &str) {
        self.pools.insert(product_id.to_string(), InvestmentPool::new());
        info!("创建投资产品资金池: {}", product_id);
    }
    
    /// 处理盈利重置
    fn handle_profit_reset(
        pool: &mut InvestmentPool,
        transaction_date: Option<NaiveDateTime>
    ) -> AuditResult<()> {
        let realized_profit = pool.total_amount.abs();
        
        let profit_record = ProfitRecord {
            reset_time: transaction_date
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "未知时间".to_string()),
            profit_amount: realized_profit,
            description: format!("重置前实现盈利 ¥{:.2}", realized_profit),
        };
        
        pool.profit_history.push(profit_record);
        pool.total_realized_profit += realized_profit;
        
        warn!("投资产品资金池重置，实现盈利 {:.2}", realized_profit);
        Ok(())
    }
    
    /// 更新申购时的资金池状态
    fn update_pool_on_purchase(
        pool: &mut InvestmentPool,
        amount: Decimal,
        personal_amount: Decimal,
        company_amount: Decimal,
        personal_ratio: Decimal,
        company_ratio: Decimal,
    ) {
        if pool.total_amount < Decimal::ZERO {
            // 重置后的新申购
            pool.personal_amount = personal_amount;
            pool.company_amount = company_amount;
            pool.total_amount = amount;
            pool.latest_personal_ratio = personal_ratio;
            pool.latest_company_ratio = company_ratio;
        } else {
            // 正常累加
            pool.personal_amount += personal_amount;
            pool.company_amount += company_amount;
            pool.total_amount += amount;
            
            // 重新计算占比
            if pool.total_amount > Decimal::ZERO {
                pool.latest_personal_ratio = pool.personal_amount / pool.total_amount;
                pool.latest_company_ratio = pool.company_amount / pool.total_amount;
            }
        }
        
        pool.total_purchase += amount;
    }
    
    /// 计算赎回分配
    fn calculate_redemption_distribution(
        &self,
        pool: &InvestmentPool,
        amount: Decimal,
        product_id: &str,
    ) -> AuditResult<RedemptionResult> {
        let mut result = RedemptionResult::new(
            amount * pool.latest_personal_ratio,
            amount * pool.latest_company_ratio,
        );
        
        // 计算收益和本金归还
        if pool.total_amount > Decimal::ZERO {
            // 正常赎回逻辑
            let redemption_ratio = amount / pool.total_amount;
            let corresponding_cost = pool.total_amount * redemption_ratio;
            result.profit = amount - corresponding_cost;
            
            // 计算本金归还
            if amount <= pool.total_amount {
                result.personal_principal_returned = pool.personal_amount * redemption_ratio;
                result.company_principal_returned = pool.company_amount * redemption_ratio;
            } else {
                result.personal_principal_returned = pool.personal_amount;
                result.company_principal_returned = pool.company_amount;
            }
        } else {
            // 纯收益分配
            result.profit = amount;
        }
        
        // 收益分配
        if result.profit > Decimal::ZERO {
            result.personal_profit = result.profit * pool.latest_personal_ratio;
            result.company_profit = result.profit * pool.latest_company_ratio;
        }
        
        // 生成行为描述
        result.behavior_description = self.generate_redemption_description(
            product_id, &result, pool.latest_personal_ratio, pool.latest_company_ratio
        );
        
        Ok(result)
    }
    
    /// 更新赎回时的资金池状态
    fn update_pool_on_redemption(
        pool: &mut InvestmentPool,
        result: &RedemptionResult,
        amount: Decimal,
    ) {
        if pool.total_amount > Decimal::ZERO {
            let redemption_ratio = amount / pool.total_amount;
            pool.personal_amount -= pool.personal_amount * redemption_ratio;
            pool.company_amount -= pool.company_amount * redemption_ratio;
            pool.total_amount -= pool.total_amount * redemption_ratio;
        } else {
            pool.personal_amount -= result.personal_return;
            pool.company_amount -= result.company_return;
            pool.total_amount -= amount;
        }
        
        pool.total_redemption += amount;
    }
    
    /// 生成赎回行为描述
    fn generate_redemption_description(
        &self,
        product_id: &str,
        result: &RedemptionResult,
        personal_ratio: Decimal,
        company_ratio: Decimal,
    ) -> String {
        let prefix = product_id.split('-').next().unwrap_or("投资");
        
        if result.profit > Decimal::ZERO {
            if personal_ratio > Decimal::ZERO && company_ratio > Decimal::ZERO {
                format!("{}赎回-{}：个人{:.2}（本金{:.2}+收益{:.2}），公司{:.2}（本金{:.2}+收益{:.2}），总收益{:.2}",
                    prefix, product_id,
                    result.personal_return, result.personal_principal_returned, result.personal_profit,
                    result.company_return, result.company_principal_returned, result.company_profit,
                    result.profit)
            } else if personal_ratio > Decimal::ZERO {
                format!("{}赎回-{}：个人{:.2}（本金{:.2}+收益{:.2}），总收益{:.2}",
                    prefix, product_id, result.personal_return, result.personal_principal_returned,
                    result.personal_profit, result.profit)
            } else {
                format!("{}赎回-{}：公司{:.2}（本金{:.2}+收益{:.2}），总收益{:.2}",
                    prefix, product_id, result.company_return, result.company_principal_returned,
                    result.company_profit, result.profit)
            }
        } else if result.profit < Decimal::ZERO {
            format!("{}赎回-{}：个人{:.2}（本金{:.2}），公司{:.2}（本金{:.2}），亏损{:.2}",
                prefix, product_id, result.personal_return, result.personal_principal_returned,
                result.company_return, result.company_principal_returned, result.profit.abs())
        } else {
            format!("{}赎回-{}：个人{:.2}（本金{:.2}），公司{:.2}（本金{:.2}），无收益",
                prefix, product_id, result.personal_return, result.personal_principal_returned,
                result.company_return, result.company_principal_returned)
        }
    }
    
    /// 记录申购交易（使用池数据避免借用冲突）
    fn record_purchase_transaction_with_data(
        &mut self,
        product_id: &str,
        pool_data: (Decimal, Decimal, Decimal, Decimal, Decimal, Decimal, Decimal), // total_amount, personal_amount, company_amount, personal_ratio, company_ratio, total_purchase, total_redemption
        amount: Decimal,
        personal_amount: Decimal,
        company_amount: Decimal,
        transaction_date: Option<NaiveDateTime>,
    ) {
        let (total_amount, pool_personal_amount, pool_company_amount, latest_personal_ratio, latest_company_ratio, total_purchase, total_redemption) = pool_data;
        let personal_ratio = if amount > Decimal::ZERO { personal_amount / amount } else { Decimal::ZERO };
        let company_ratio = if amount > Decimal::ZERO { company_amount / amount } else { Decimal::ZERO };
        
        let record = InvestmentTransactionRecord::new(
            transaction_date
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "未知时间".to_string()),
            product_id.to_string(),
            amount,
            Decimal::ZERO,
            total_amount,
            pool_personal_amount,
            pool_company_amount,
            format!("个人{:.1}%，公司{:.1}%", personal_ratio * Decimal::from(100), company_ratio * Decimal::from(100)),
            format!("个人{:.1}%，公司{:.1}%", latest_personal_ratio * Decimal::from(100), latest_company_ratio * Decimal::from(100)),
            format!("入金（个人{:.0}，公司{:.0}）", personal_amount, company_amount),
            total_purchase,
            total_redemption,
        );
        
        self.transaction_records.push(record);
    }
    
    /// 记录赎回交易（使用池数据避免借用冲突）
    fn record_redemption_transaction_with_data(
        &mut self,
        product_id: &str,
        pool_data: (Decimal, Decimal, Decimal, Decimal, Decimal, Decimal, Decimal), // total_amount, personal_amount, company_amount, personal_ratio, company_ratio, total_purchase, total_redemption
        result: &RedemptionResult,
        amount: Decimal,
        transaction_date: Option<NaiveDateTime>,
    ) {
        let (total_amount, personal_amount, company_amount, latest_personal_ratio, latest_company_ratio, total_purchase, total_redemption) = pool_data;
        let record = InvestmentTransactionRecord::new(
            transaction_date
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "未知时间".to_string()),
            product_id.to_string(),
            Decimal::ZERO,
            amount,
            total_amount,
            personal_amount,
            company_amount,
            format!("个人{:.1}%，公司{:.1}%", latest_personal_ratio * Decimal::from(100), latest_company_ratio * Decimal::from(100)),
            format!("个人{:.1}%，公司{:.1}%", latest_personal_ratio * Decimal::from(100), latest_company_ratio * Decimal::from(100)),
            format!("出金（个人{:.0}，公司{:.0}，收益{:.0}）", 
                    result.personal_return, result.company_return, result.profit),
            total_purchase,
            total_redemption,
        );
        
        self.transaction_records.push(record);
    }
}

impl Default for InvestmentService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_investment_service_creation() {
        let service = InvestmentService::new();
        assert_eq!(service.pools.len(), 0);
        assert_eq!(service.transaction_records.len(), 0);
    }
    
    #[test]
    fn test_process_purchase() {
        let mut service = InvestmentService::new();
        
        let result = service.process_purchase(
            "理财-SL001",
            Decimal::from(100000),
            Decimal::from_f64_retain(0.6).unwrap(),
            Decimal::from_f64_retain(0.4).unwrap(),
            None,
        );
        
        assert!(result.is_ok());
        
        let pool = service.get_pool("理财-SL001").unwrap();
        assert_eq!(pool.total_amount, Decimal::from(100000));
        assert_eq!(pool.personal_amount, Decimal::from(60000));
        assert_eq!(pool.company_amount, Decimal::from(40000));
        assert_eq!(service.transaction_records.len(), 1);
    }
    
    #[test]
    fn test_process_redemption() {
        let mut service = InvestmentService::new();
        
        // 先申购
        service.process_purchase(
            "理财-SL001",
            Decimal::from(100000),
            Decimal::from_f64_retain(0.6).unwrap(),
            Decimal::from_f64_retain(0.4).unwrap(),
            None,
        ).unwrap();
        
        // 再赎回（有收益）
        let result = service.process_redemption(
            "理财-SL001",
            Decimal::from(120000),
            None,
        ).unwrap();
        
        assert_eq!(result.profit, Decimal::from(20000));
        assert_eq!(result.personal_return, Decimal::from(72000));
        assert_eq!(result.company_return, Decimal::from(48000));
        assert_eq!(result.personal_profit, Decimal::from(12000));
        assert_eq!(result.company_profit, Decimal::from(8000));
    }
}