//! 场外资金池记录数据模型
//! 
//! 用于跟踪和记录投资产品的资金流入流出情况

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 场外资金池记录
/// 记录每笔投资产品的申购/赎回交易详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsitePoolRecord {
    /// 交易时间
    pub transaction_time: String,
    
    /// 资金池名称（投资产品编号）
    pub pool_name: String,
    
    /// 入金金额（申购）
    pub inflow: Decimal,
    
    /// 出金金额（赎回）
    pub outflow: Decimal,
    
    /// 资金池总余额
    pub total_balance: Decimal,
    
    /// 个人资金余额
    pub personal_balance: Decimal,
    
    /// 公司资金余额
    pub company_balance: Decimal,
    
    /// 资金占比描述文字
    pub fund_ratio: String,
    
    /// 行为性质描述
    pub behavior_nature: String,
    
    /// 累计申购总额
    pub cumulative_purchase: Decimal,
    
    /// 累计赎回总额
    pub cumulative_redemption: Decimal,
    
    /// 净盈亏（仅赎回时有意义）
    pub net_profit_loss: Decimal,
}

impl OffsitePoolRecord {
    /// 创建申购记录
    pub fn new_purchase(
        transaction_time: Option<DateTime<Local>>,
        pool_name: String,
        amount: Decimal,
        total_balance: Decimal,
        personal_balance: Decimal,
        company_balance: Decimal,
        personal_amount: Decimal,
        company_amount: Decimal,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        cumulative_purchase: Decimal,
        cumulative_redemption: Decimal,
    ) -> Self {
        let time_str = match transaction_time {
            Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => "未知时间".to_string(),
        };
        
        // 计算资金占比描述
        let fund_ratio = if total_balance > Decimal::ZERO {
            format!(
                "个人{:.1}%，公司{:.1}%", 
                personal_ratio * Decimal::from(100),
                company_ratio * Decimal::from(100)
            )
        } else {
            "无资金".to_string()
        };
        
        // 行为性质描述
        let behavior_nature = format!(
            "入金（个人{:.0}，公司{:.0}）",
            personal_amount, company_amount
        );
        
        Self {
            transaction_time: time_str,
            pool_name,
            inflow: amount,
            outflow: Decimal::ZERO,
            total_balance,
            personal_balance,
            company_balance,
            fund_ratio,
            behavior_nature,
            cumulative_purchase,
            cumulative_redemption,
            net_profit_loss: cumulative_redemption - cumulative_purchase, // 累计净盈亏
        }
    }
    
    /// 创建赎回记录
    pub fn new_redemption(
        transaction_time: Option<DateTime<Local>>,
        pool_name: String,
        amount: Decimal,
        total_balance: Decimal,
        personal_balance: Decimal,
        company_balance: Decimal,
        personal_return: Decimal,
        company_return: Decimal,
        profit: Decimal,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        cumulative_purchase: Decimal,
        cumulative_redemption: Decimal,
    ) -> Self {
        let time_str = match transaction_time {
            Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => "未知时间".to_string(),
        };
        
        // 计算资金占比描述
        let fund_ratio = if total_balance.abs() > Decimal::new(1, 2) { // > 0.01
            format!(
                "个人{:.1}%，公司{:.1}%", 
                personal_ratio * Decimal::from(100),
                company_ratio * Decimal::from(100)
            )
        } else {
            "资金池清空".to_string()
        };
        
        // 行为性质描述
        let behavior_nature = format!(
            "出金（个人{:.0}，公司{:.0}，收益{:.0}）",
            personal_return, company_return, profit
        );
        
        Self {
            transaction_time: time_str,
            pool_name,
            inflow: Decimal::ZERO,
            outflow: amount,
            total_balance,
            personal_balance,
            company_balance,
            fund_ratio,
            behavior_nature,
            cumulative_purchase,
            cumulative_redemption,
            net_profit_loss: cumulative_redemption - cumulative_purchase, // 累计净盈亏
        }
    }
}

/// 场外资金池记录管理器
/// 负责收集、组织和导出场外资金池记录
#[derive(Debug, Clone, Default)]
pub struct OffsitePoolRecordManager {
    /// 记录列表
    pub records: Vec<OffsitePoolRecord>,
}

impl OffsitePoolRecordManager {
    /// 创建新的记录管理器
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    /// 添加申购记录
    pub fn add_purchase_record(
        &mut self,
        transaction_time: Option<DateTime<Local>>,
        pool_name: String,
        amount: Decimal,
        total_balance: Decimal,
        personal_balance: Decimal,
        company_balance: Decimal,
        personal_amount: Decimal,
        company_amount: Decimal,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        cumulative_purchase: Decimal,
        cumulative_redemption: Decimal,
    ) {
        let record = OffsitePoolRecord::new_purchase(
            transaction_time,
            pool_name,
            amount,
            total_balance,
            personal_balance,
            company_balance,
            personal_amount,
            company_amount,
            personal_ratio,
            company_ratio,
            cumulative_purchase,
            cumulative_redemption,
        );
        
        self.records.push(record);
    }
    
    /// 添加赎回记录
    pub fn add_redemption_record(
        &mut self,
        transaction_time: Option<DateTime<Local>>,
        pool_name: String,
        amount: Decimal,
        total_balance: Decimal,
        personal_balance: Decimal,
        company_balance: Decimal,
        personal_return: Decimal,
        company_return: Decimal,
        profit: Decimal,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        cumulative_purchase: Decimal,
        cumulative_redemption: Decimal,
    ) {
        let record = OffsitePoolRecord::new_redemption(
            transaction_time,
            pool_name,
            amount,
            total_balance,
            personal_balance,
            company_balance,
            personal_return,
            company_return,
            profit,
            personal_ratio,
            company_ratio,
            cumulative_purchase,
            cumulative_redemption,
        );
        
        self.records.push(record);
    }
    
    /// 获取记录数量
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
    
    /// 清空所有记录
    pub fn clear(&mut self) {
        self.records.clear();
    }
    
    /// 按资金池名称分组记录
    pub fn group_by_pool(&self) -> HashMap<String, Vec<&OffsitePoolRecord>> {
        let mut grouped = HashMap::new();
        
        for record in &self.records {
            grouped
                .entry(record.pool_name.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        // 每组内按时间排序
        for records in grouped.values_mut() {
            records.sort_by(|a, b| a.transaction_time.cmp(&b.transaction_time));
        }
        
        grouped
    }
    
    /// 获取指定资金池的记录
    pub fn get_pool_records(&self, pool_name: &str) -> Vec<&OffsitePoolRecord> {
        self.records
            .iter()
            .filter(|record| record.pool_name == pool_name)
            .collect()
    }
    
    /// 计算指定资金池的统计信息
    pub fn calculate_pool_stats(&self, pool_name: &str) -> Option<PoolStatistics> {
        let pool_records = self.get_pool_records(pool_name);
        
        if pool_records.is_empty() {
            return None;
        }
        
        let mut total_purchase = Decimal::ZERO;
        let mut total_redemption = Decimal::ZERO;
        let mut final_balance = Decimal::ZERO;
        let mut final_personal_balance = Decimal::ZERO;
        let mut final_company_balance = Decimal::ZERO;
        let cumulative_profit_loss = Decimal::ZERO;
        let mut cumulative_personal_profit_loss = Decimal::ZERO;
        let mut cumulative_company_profit_loss = Decimal::ZERO;
        
        for record in &pool_records {
            total_purchase += record.inflow;
            total_redemption += record.outflow;
        }
        
        // 取最后一条记录的余额作为最终余额
        if let Some(last_record) = pool_records.last() {
            final_balance = last_record.total_balance;
            final_personal_balance = last_record.personal_balance;
            final_company_balance = last_record.company_balance;
        }
        
        // 计算总净盈亏：直接使用累计申购赎回数据（最简单最可靠的方法）
        let profit_loss = total_redemption - total_purchase;
        
        // 新的通用逻辑：区分盈利和亏损状态的不同计算方式
        if profit_loss > Decimal::ZERO {
            // 盈利状态：累加所有负余额（已实现收益）
            let mut i = 0;
            while i < pool_records.len() {
                let record = pool_records[i];
                
                // 寻找负余额期间
                if record.total_balance < Decimal::ZERO {
                    let mut last_negative_personal = record.personal_balance;
                    let mut last_negative_company = record.company_balance;
                    
                    // 继续寻找这个负余额期间的最后一笔记录
                    let mut j = i + 1;
                    while j < pool_records.len() && pool_records[j].total_balance < Decimal::ZERO {
                        last_negative_personal = pool_records[j].personal_balance;
                        last_negative_company = pool_records[j].company_balance;
                        j += 1;
                    }
                    
                    // 检查是否有资金池重置（余额变为0或从负数变正数）
                    let has_reset = j < pool_records.len() && 
                        (pool_records[j].total_balance >= Decimal::ZERO || 
                         pool_records[j].behavior_nature.contains("资金池清空"));
                    
                    if has_reset {
                        // 这是一次完整的负余额周期，累加最后一笔负余额的绝对值
                        cumulative_personal_profit_loss += last_negative_personal.abs();
                        cumulative_company_profit_loss += last_negative_company.abs();
                    }
                    
                    // 跳到这个负余额期间的结束
                    i = j;
                } else {
                    i += 1;
                }
            }
            
            // 特殊情况：如果最后一条记录是负余额且没有后续重置，也要计入收益
            if let Some(last_record) = pool_records.last() {
                if last_record.total_balance < Decimal::ZERO {
                    // 检查这个最终负余额是否已经被上面的逻辑计算过了
                    let already_counted = pool_records.len() >= 2 && 
                        pool_records[pool_records.len() - 2].total_balance < Decimal::ZERO;
                    
                    if !already_counted {
                        // 这是一个独立的最终负余额，需要单独计入
                        cumulative_personal_profit_loss += last_record.personal_balance.abs();
                        cumulative_company_profit_loss += last_record.company_balance.abs();
                    }
                }
            }
        } else if profit_loss < Decimal::ZERO {
            // 亏损状态：按最终余额比例分配亏损
            if let Some(last_record) = pool_records.last() {
                if last_record.total_balance > Decimal::ZERO {
                    // 最终余额是正数，按比例分配亏损
                    let personal_ratio = last_record.personal_balance / last_record.total_balance;
                    let company_ratio = last_record.company_balance / last_record.total_balance;
                    
                    let total_loss = profit_loss.abs(); // 将负数转为正数表示损失
                    cumulative_personal_profit_loss = total_loss * personal_ratio;
                    cumulative_company_profit_loss = total_loss * company_ratio;
                }
            }
        }
        
        let status = if profit_loss > Decimal::ZERO {
            "盈利"
        } else if profit_loss < Decimal::ZERO {
            "亏损"
        } else {
            "持平"
        };
        
        Some(PoolStatistics {
            pool_name: pool_name.to_string(),
            total_purchase,
            total_redemption,
            final_balance,
            final_personal_balance,
            final_company_balance,
            profit_loss,
            cumulative_personal_profit_loss,
            cumulative_company_profit_loss,
            status: status.to_string(),
            record_count: pool_records.len(),
        })
    }
    
    /// 解析赎回行为性质描述，提取个人和公司的赎回金额
    /// 解析格式：出金（个人304200，公司2155799，收益0）
    fn parse_redemption_amounts(behavior_nature: &str) -> (Decimal, Decimal) {
        // 使用正则表达式解析个人和公司金额
        if let Some(start) = behavior_nature.find("出金（") {
            let content = &behavior_nature[start + "出金（".len()..];
            if let Some(end) = content.find('）') {
                let amounts_str = &content[..end];
                
                let mut personal_amount = Decimal::ZERO;
                let mut company_amount = Decimal::ZERO;
                
                // 解析个人金额
                if let Some(personal_start) = amounts_str.find("个人") {
                    let personal_part = &amounts_str[personal_start + "个人".len()..];
                    if let Some(comma_pos) = personal_part.find('，') {
                        let amount_str = &personal_part[..comma_pos];
                        if let Ok(amount) = amount_str.parse::<f64>() {
                            personal_amount = Decimal::from_f64_retain(amount).unwrap_or(Decimal::ZERO);
                        }
                    }
                }
                
                // 解析公司金额
                if let Some(company_start) = amounts_str.find("公司") {
                    let company_part = &amounts_str[company_start + "公司".len()..];
                    if let Some(comma_pos) = company_part.find('，') {
                        let amount_str = &company_part[..comma_pos];
                        if let Ok(amount) = amount_str.parse::<f64>() {
                            company_amount = Decimal::from_f64_retain(amount).unwrap_or(Decimal::ZERO);
                        }
                    }
                }
                
                return (personal_amount, company_amount);
            }
        }
        
        // 如果解析失败，返回0
        (Decimal::ZERO, Decimal::ZERO)
    }
}

/// 资金池统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatistics {
    /// 资金池名称
    pub pool_name: String,
    
    /// 总申购金额
    pub total_purchase: Decimal,
    
    /// 总赎回金额
    pub total_redemption: Decimal,
    
    /// 最终余额
    pub final_balance: Decimal,
    
    /// 最终个人余额
    pub final_personal_balance: Decimal,
    
    /// 最终公司余额
    pub final_company_balance: Decimal,
    
    /// 净盈亏
    pub profit_loss: Decimal,
    
    /// 累计个人盈亏
    pub cumulative_personal_profit_loss: Decimal,
    
    /// 累计公司盈亏
    pub cumulative_company_profit_loss: Decimal,
    
    /// 盈亏状态
    pub status: String,
    
    /// 记录数量
    pub record_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    
    #[test]
    fn test_create_purchase_record() {
        let now = Local::now();
        let record = OffsitePoolRecord::new_purchase(
            Some(now),
            "测试资金池".to_string(),
            Decimal::from(10000),
            Decimal::from(10000),
            Decimal::from(6000),
            Decimal::from(4000),
            Decimal::from(6000),
            Decimal::from(4000),
            Decimal::new(6, 1), // 0.6
            Decimal::new(4, 1), // 0.4
            Decimal::from(10000),
            Decimal::ZERO,
        );
        
        assert_eq!(record.pool_name, "测试资金池");
        assert_eq!(record.inflow, Decimal::from(10000));
        assert_eq!(record.outflow, Decimal::ZERO);
        assert!(record.fund_ratio.contains("个人60.0%"));
        assert!(record.fund_ratio.contains("公司40.0%"));
    }
    
    #[test]
    fn test_record_manager() {
        let mut manager = OffsitePoolRecordManager::new();
        
        // 添加申购记录
        manager.add_purchase_record(
            Some(Local::now()),
            "测试池A".to_string(),
            Decimal::from(5000),
            Decimal::from(5000),
            Decimal::from(3000),
            Decimal::from(2000),
            Decimal::from(3000),
            Decimal::from(2000),
            Decimal::new(6, 1), // 0.6
            Decimal::new(4, 1), // 0.4
            Decimal::from(5000),
            Decimal::ZERO,
        );
        
        assert_eq!(manager.record_count(), 1);
        
        // 测试分组功能
        let grouped = manager.group_by_pool();
        assert!(grouped.contains_key("测试池A"));
        assert_eq!(grouped["测试池A"].len(), 1);
    }
    
    #[test]
    fn test_pool_statistics() {
        let mut manager = OffsitePoolRecordManager::new();
        
        // 添加多条记录
        manager.add_purchase_record(
            Some(Local::now()),
            "测试池".to_string(),
            Decimal::from(1000),
            Decimal::from(1000),
            Decimal::from(600),
            Decimal::from(400),
            Decimal::from(600),
            Decimal::from(400),
            Decimal::new(6, 1),
            Decimal::new(4, 1),
            Decimal::from(1000),
            Decimal::ZERO,
        );
        
        manager.add_redemption_record(
            Some(Local::now()),
            "测试池".to_string(),
            Decimal::from(500),
            Decimal::from(-100), // 盈利100
            Decimal::from(-60),
            Decimal::from(-40),
            Decimal::from(300),
            Decimal::from(200),
            Decimal::from(100),
            Decimal::new(6, 1),
            Decimal::new(4, 1),
            Decimal::from(1000),
            Decimal::from(500),
        );
        
        let stats = manager.calculate_pool_stats("测试池").unwrap();
        assert_eq!(stats.pool_name, "测试池");
        assert_eq!(stats.total_purchase, Decimal::from(1000));
        assert_eq!(stats.total_redemption, Decimal::from(500));
        assert_eq!(stats.record_count, 2);
        assert_eq!(stats.status, "盈利");
    }
}