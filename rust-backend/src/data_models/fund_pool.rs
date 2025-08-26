//! 资金池记录数据模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use chrono::NaiveDateTime; // 暂时未使用

/// 资金池记录
/// 
/// 表示投资产品的单笔交易记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FundPoolRecord {
    /// 交易时间
    #[serde(rename = "交易时间")]
    pub transaction_time: String,
    
    /// 资金池名称（产品名称）
    #[serde(rename = "资金池名称")]
    pub pool_name: String,
    
    /// 入金金额（申购）
    #[serde(rename = "入金")]
    pub inflow: Decimal,
    
    /// 出金金额（赎回）
    #[serde(rename = "出金")]
    pub outflow: Decimal,
    
    /// 当前总余额
    #[serde(rename = "总余额")]
    pub total_balance: Decimal,
    
    /// 单笔资金占比
    #[serde(rename = "单笔资金占比")]
    pub single_fund_ratio: String,
    
    /// 总资金占比
    #[serde(rename = "总资金占比")]
    pub total_fund_ratio: String,
    
    /// 行为性质
    #[serde(rename = "行为性质")]
    pub behavior_nature: String,
    
    /// 累计申购
    #[serde(rename = "累计申购")]
    pub cumulative_purchase: Decimal,
    
    /// 累计赎回
    #[serde(rename = "累计赎回")]
    pub cumulative_redemption: Decimal,
}

impl FundPoolRecord {
    /// 创建新的资金池记录
    pub fn new(
        transaction_time: String,
        pool_name: String,
        inflow: Decimal,
        outflow: Decimal,
        total_balance: Decimal,
        single_fund_ratio: String,
        total_fund_ratio: String,
        behavior_nature: String,
        cumulative_purchase: Decimal,
        cumulative_redemption: Decimal,
    ) -> Self {
        Self {
            transaction_time,
            pool_name,
            inflow,
            outflow,
            total_balance,
            single_fund_ratio,
            total_fund_ratio,
            behavior_nature,
            cumulative_purchase,
            cumulative_redemption,
        }
    }
    
    /// 获取净现金流
    pub fn net_cash_flow(&self) -> Decimal {
        self.inflow - self.outflow
    }
    
    /// 判断是否为申购操作
    pub fn is_purchase(&self) -> bool {
        self.inflow > Decimal::ZERO && self.outflow == Decimal::ZERO
    }
    
    /// 判断是否为赎回操作
    pub fn is_redemption(&self) -> bool {
        self.outflow > Decimal::ZERO && self.inflow == Decimal::ZERO
    }
    
    /// 计算盈亏情况
    pub fn calculate_profit_loss(&self) -> Decimal {
        self.cumulative_redemption - self.cumulative_purchase
    }
}

/// 资金池汇总信息
/// 
/// 包含特定资金池的统计汇总数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FundPoolSummary {
    /// 资金池名称
    pub pool_name: String,
    
    /// 总流入金额
    pub total_inflow: Decimal,
    
    /// 总流出金额
    pub total_outflow: Decimal,
    
    /// 当前余额
    pub current_balance: Decimal,
    
    /// 记录数量
    pub record_count: usize,
    
    /// 净盈亏
    pub net_profit_loss: Decimal,
    
    /// 状态（盈利/亏损/持平）
    pub status: String,
    
    /// 首次交易时间
    pub first_transaction_time: Option<String>,
    
    /// 最后交易时间
    pub last_transaction_time: Option<String>,
}

impl FundPoolSummary {
    /// 从资金池记录列表创建汇总
    pub fn from_records(pool_name: &str, records: &[FundPoolRecord]) -> Self {
        if records.is_empty() {
            return Self {
                pool_name: pool_name.to_string(),
                total_inflow: Decimal::ZERO,
                total_outflow: Decimal::ZERO,
                current_balance: Decimal::ZERO,
                record_count: 0,
                net_profit_loss: Decimal::ZERO,
                status: "持平".to_string(),
                first_transaction_time: None,
                last_transaction_time: None,
            };
        }
        
        let total_inflow = records.iter().map(|r| r.inflow).sum();
        let total_outflow = records.iter().map(|r| r.outflow).sum();
        let net_profit_loss = records.last().map(|r| r.calculate_profit_loss()).unwrap_or(Decimal::ZERO);
        
        let status = if net_profit_loss > Decimal::ZERO {
            "盈利"
        } else if net_profit_loss < Decimal::ZERO {
            "亏损"
        } else {
            "持平"
        }.to_string();
        
        let current_balance = records.last().map(|r| r.total_balance).unwrap_or(Decimal::ZERO);
        
        Self {
            pool_name: pool_name.to_string(),
            total_inflow,
            total_outflow,
            current_balance,
            record_count: records.len(),
            net_profit_loss,
            status,
            first_transaction_time: records.first().map(|r| r.transaction_time.clone()),
            last_transaction_time: records.last().map(|r| r.transaction_time.clone()),
        }
    }
    
    /// 计算投资回报率
    pub fn calculate_return_rate(&self) -> Option<Decimal> {
        if self.total_inflow > Decimal::ZERO {
            Some(self.net_profit_loss / self.total_inflow)
        } else {
            None
        }
    }
}

/// 资金池管理器
/// 
/// 管理所有资金池的记录和汇总信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundPoolManager {
    /// 所有资金池记录，按资金池名称分组
    pub pools: HashMap<String, Vec<FundPoolRecord>>,
}

impl FundPoolManager {
    /// 创建新的资金池管理器
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }
    
    /// 添加资金池记录
    pub fn add_record(&mut self, record: FundPoolRecord) {
        self.pools
            .entry(record.pool_name.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    /// 获取指定资金池的记录
    pub fn get_pool_records(&self, pool_name: &str) -> Option<&Vec<FundPoolRecord>> {
        self.pools.get(pool_name)
    }
    
    /// 获取指定资金池的汇总信息
    pub fn get_pool_summary(&self, pool_name: &str) -> Option<FundPoolSummary> {
        self.pools.get(pool_name).map(|records| {
            FundPoolSummary::from_records(pool_name, records)
        })
    }
    
    /// 获取所有资金池名称
    pub fn get_all_pool_names(&self) -> Vec<String> {
        self.pools.keys().cloned().collect()
    }
    
    /// 获取总体统计信息
    pub fn get_overall_statistics(&self) -> OverallPoolStatistics {
        let total_pools = self.pools.len();
        let total_records: usize = self.pools.values().map(|records| records.len()).sum();
        
        let summaries: Vec<FundPoolSummary> = self.pools.iter()
            .map(|(name, records)| FundPoolSummary::from_records(name, records))
            .collect();
        
        let total_inflow: Decimal = summaries.iter().map(|s| s.total_inflow).sum();
        let total_outflow: Decimal = summaries.iter().map(|s| s.total_outflow).sum();
        let total_balance: Decimal = summaries.iter().map(|s| s.current_balance).sum();
        let total_profit_loss: Decimal = summaries.iter().map(|s| s.net_profit_loss).sum();
        
        let profitable_pools = summaries.iter().filter(|s| s.net_profit_loss > Decimal::ZERO).count();
        let loss_making_pools = summaries.iter().filter(|s| s.net_profit_loss < Decimal::ZERO).count();
        let break_even_pools = summaries.iter().filter(|s| s.net_profit_loss == Decimal::ZERO).count();
        
        OverallPoolStatistics {
            total_pools,
            total_records,
            total_inflow,
            total_outflow,
            total_balance,
            total_profit_loss,
            profitable_pools,
            loss_making_pools,
            break_even_pools,
        }
    }
}

impl Default for FundPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 总体资金池统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallPoolStatistics {
    /// 资金池总数
    pub total_pools: usize,
    
    /// 总记录数
    pub total_records: usize,
    
    /// 总流入金额
    pub total_inflow: Decimal,
    
    /// 总流出金额
    pub total_outflow: Decimal,
    
    /// 总余额
    pub total_balance: Decimal,
    
    /// 总盈亏
    pub total_profit_loss: Decimal,
    
    /// 盈利资金池数量
    pub profitable_pools: usize,
    
    /// 亏损资金池数量
    pub loss_making_pools: usize,
    
    /// 持平资金池数量
    pub break_even_pools: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fund_pool_record() {
        let record = FundPoolRecord::new(
            "2023-01-15 14:30:25".to_string(),
            "理财-SL100613100620".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(50000),
            "个人:100.0%".to_string(),
            "个人:100.0%".to_string(),
            "投资申购".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
        );
        
        assert!(record.is_purchase());
        assert!(!record.is_redemption());
        assert_eq!(record.net_cash_flow(), Decimal::from(50000));
    }
    
    #[test]
    fn test_fund_pool_manager() {
        let mut manager = FundPoolManager::new();
        
        let record1 = FundPoolRecord::new(
            "2023-01-15 14:30:25".to_string(),
            "理财-A".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(50000),
            "个人:100.0%".to_string(),
            "个人:100.0%".to_string(),
            "投资申购".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
        );
        
        let record2 = FundPoolRecord::new(
            "2023-01-20 10:15:30".to_string(),
            "理财-A".to_string(),
            Decimal::ZERO,
            Decimal::from(55000),
            Decimal::ZERO,
            "个人:100.0%".to_string(),
            "个人:100.0%".to_string(),
            "投资赎回".to_string(),
            Decimal::from(50000),
            Decimal::from(55000),
        );
        
        manager.add_record(record1);
        manager.add_record(record2);
        
        let pool_records = manager.get_pool_records("理财-A").unwrap();
        assert_eq!(pool_records.len(), 2);
        
        let summary = manager.get_pool_summary("理财-A").unwrap();
        assert_eq!(summary.total_inflow, Decimal::from(50000));
        assert_eq!(summary.total_outflow, Decimal::from(55000));
        assert_eq!(summary.net_profit_loss, Decimal::from(5000));
        assert_eq!(summary.status, "盈利");
    }
}