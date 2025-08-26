//! 审计摘要生成模块
//! 
//! 负责生成统一的审计摘要和报告

use super::tracker_base::TrackerBase;
// use super::tracker_base::OffSiteRecord; // 暂时未使用
use crate::data_models::AuditSummary;
use rust_decimal::Decimal;

/// 摘要生成器
/// 
/// 负责生成各种格式的审计摘要和报告
pub struct SummaryGenerator;

impl SummaryGenerator {
    /// 生成审计摘要
    /// 
    /// 对应Python版本的`获取状态摘要`方法
    pub fn generate_audit_summary(base: &TrackerBase) -> AuditSummary {
        base.get_audit_summary()
    }

    /// 生成详细的状态摘要文本
    /// 
    /// # Arguments
    /// * `base` - 追踪器基础状态
    /// * `algorithm_name` - 算法名称（"FIFO" 或 "差额计算法"）
    /// 
    /// # Returns
    /// 格式化的状态摘要文本
    pub fn generate_detailed_summary_text(base: &TrackerBase, algorithm_name: &str) -> String {
        let mut summary = Vec::new();
        
        summary.push(format!("=== {} 资金追踪审计摘要 ===", algorithm_name));
        summary.push("".to_string());
        
        // 基础余额信息
        summary.push("【基础余额状态】".to_string());
        summary.push(format!("个人余额: ¥{:.2}", base.personal_balance));
        summary.push(format!("公司余额: ¥{:.2}", base.company_balance));
        summary.push(format!("总余额: ¥{:.2}", base.total_balance));
        summary.push("".to_string());
        
        // 累计统计信息
        summary.push("【累计统计信息】".to_string());
        summary.push(format!("累计挪用金额: ¥{:.2}", base.total_misappropriation));
        summary.push(format!("累计垫付金额: ¥{:.2}", base.total_advance_payment));
        summary.push(format!("资金缺口: ¥{:.2}", base.calculate_funding_gap()));
        summary.push("".to_string());
        
        // 投资相关统计
        summary.push("【投资相关统计】".to_string());
        summary.push(format!("投资产品数量: {}", base.investment_product_count));
        summary.push(format!("累计由资金池回归公司余额本金: ¥{:.2}", base.total_company_principal_returned));
        summary.push(format!("累计由资金池回归个人余额本金: ¥{:.2}", base.total_personal_principal_returned));
        summary.push(format!("总计个人应分配利润: ¥{:.2}", base.total_personal_profit));
        summary.push(format!("总计公司应分配利润: ¥{:.2}", base.total_company_profit));
        summary.push(format!("累计非法所得: ¥{:.2}", base.total_illegal_gain));
        summary.push("".to_string());
        
        // 投资产品详情
        if !base.investment_pools.is_empty() {
            summary.push("【投资产品详情】".to_string());
            for (product_name, pool) in &base.investment_pools {
                summary.push(format!("产品: {}", product_name));
                summary.push(format!("  当前总金额: ¥{:.2}", pool.total_amount));
                summary.push(format!("  个人金额: ¥{:.2} ({:.1}%)", 
                    pool.personal_amount, pool.latest_personal_ratio * Decimal::from(100)));
                summary.push(format!("  公司金额: ¥{:.2} ({:.1}%)", 
                    pool.company_amount, pool.latest_company_ratio * Decimal::from(100)));
                summary.push(format!("  累计申购: ¥{:.2}", pool.cumulative_purchase));
                summary.push(format!("  累计赎回: ¥{:.2}", pool.cumulative_redemption));
                summary.push(format!("  累计已实现盈利: ¥{:.2}", pool.cumulative_realized_profit));
                if !pool.historical_profit_records.is_empty() {
                    summary.push(format!("  历史盈利记录: {}次", pool.historical_profit_records.len()));
                }
                summary.push("".to_string());
            }
        }
        
        // 场外资金池记录统计
        if !base.off_site_records.is_empty() {
            summary.push("【场外资金池记录统计】".to_string());
            summary.push(format!("总记录数: {}", base.off_site_records.len()));
            
            // 按产品分组统计
            let mut product_stats: std::collections::HashMap<String, (usize, Decimal, Decimal)> = 
                std::collections::HashMap::new();
            
            for record in &base.off_site_records {
                let (count, total_inflow, total_outflow) = product_stats
                    .entry(record.pool_name.clone())
                    .or_insert((0, Decimal::ZERO, Decimal::ZERO));
                *count += 1;
                *total_inflow += record.inflow;
                *total_outflow += record.outflow;
            }
            
            for (product, (count, inflow, outflow)) in product_stats {
                summary.push(format!("  {}: {}笔交易, 入金¥{:.2}, 出金¥{:.2}", 
                    product, count, inflow, outflow));
            }
            summary.push("".to_string());
        }
        
        summary.push(format!("=== {} 审计摘要结束 ===", algorithm_name));
        
        summary.join("\n")
    }

    /// 生成场外资金池记录的CSV格式数据
    /// 
    /// # Arguments
    /// * `base` - 追踪器基础状态
    /// 
    /// # Returns
    /// CSV格式的字符串
    pub fn generate_off_site_records_csv(base: &TrackerBase) -> String {
        if base.off_site_records.is_empty() {
            return "无场外资金池记录".to_string();
        }
        
        let mut csv_lines = Vec::new();
        
        // CSV头部
        csv_lines.push("交易时间,资金池名称,入金,出金,总余额,个人余额,公司余额,资金占比,行为性质,累计申购,累计赎回".to_string());
        
        // 数据行
        for record in &base.off_site_records {
            csv_lines.push(format!(
                "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{},{},{:.2},{:.2}",
                record.transaction_time,
                record.pool_name,
                record.inflow,
                record.outflow,
                record.total_balance,
                record.personal_balance,
                record.company_balance,
                record.fund_ratio,
                record.behavior_nature,
                record.cumulative_purchase,
                record.cumulative_redemption
            ));
        }
        
        csv_lines.join("\n")
    }

    /// 生成投资产品盈利报告
    /// 
    /// # Arguments
    /// * `base` - 追踪器基础状态
    /// 
    /// # Returns
    /// 投资产品盈利分析报告
    pub fn generate_investment_profit_report(base: &TrackerBase) -> String {
        if base.investment_pools.is_empty() {
            return "无投资产品数据".to_string();
        }
        
        let mut report = Vec::new();
        report.push("=== 投资产品盈利分析报告 ===".to_string());
        report.push("".to_string());
        
        let mut total_investment = Decimal::ZERO;
        let mut total_redemption = Decimal::ZERO;
        let mut total_current_value = Decimal::ZERO;
        let mut total_realized_profit = Decimal::ZERO;
        
        for (product_name, pool) in &base.investment_pools {
            report.push(format!("【{}】", product_name));
            
            let current_profit_loss = if pool.total_amount < Decimal::ZERO {
                pool.total_amount.abs() // 负数表示盈利
            } else if pool.total_amount > pool.cumulative_purchase - pool.cumulative_redemption {
                pool.total_amount - (pool.cumulative_purchase - pool.cumulative_redemption) // 正收益
            } else {
                (pool.cumulative_purchase - pool.cumulative_redemption) - pool.total_amount // 负收益（亏损）
            };
            
            let total_profit_loss = pool.cumulative_realized_profit + current_profit_loss;
            
            report.push(format!("  累计申购: ¥{:.2}", pool.cumulative_purchase));
            report.push(format!("  累计赎回: ¥{:.2}", pool.cumulative_redemption));
            report.push(format!("  当前价值: ¥{:.2}", pool.total_amount));
            report.push(format!("  当前盈亏: ¥{:.2}", current_profit_loss));
            report.push(format!("  已实现盈利: ¥{:.2}", pool.cumulative_realized_profit));
            report.push(format!("  总盈亏: ¥{:.2}", total_profit_loss));
            
            if !pool.historical_profit_records.is_empty() {
                report.push(format!("  历史盈利记录:"));
                for record in &pool.historical_profit_records {
                    report.push(format!("    {} - ¥{:.2} ({})", 
                        record.reset_time, record.profit_amount, record.description));
                }
            }
            
            report.push("".to_string());
            
            // 累计统计
            total_investment += pool.cumulative_purchase;
            total_redemption += pool.cumulative_redemption;
            total_current_value += pool.total_amount;
            total_realized_profit += pool.cumulative_realized_profit;
        }
        
        // 总体统计
        report.push("【总体统计】".to_string());
        report.push(format!("总投资金额: ¥{:.2}", total_investment));
        report.push(format!("总赎回金额: ¥{:.2}", total_redemption));
        report.push(format!("当前总价值: ¥{:.2}", total_current_value));
        report.push(format!("总已实现盈利: ¥{:.2}", total_realized_profit));
        
        let net_investment = total_investment - total_redemption;
        let unrealized_profit_loss = total_current_value - net_investment;
        let total_profit_loss = total_realized_profit + unrealized_profit_loss;
        
        report.push(format!("净投资金额: ¥{:.2}", net_investment));
        report.push(format!("未实现盈亏: ¥{:.2}", unrealized_profit_loss));
        report.push(format!("总盈亏: ¥{:.2}", total_profit_loss));
        
        let roi = if net_investment > Decimal::ZERO {
            (total_profit_loss / net_investment) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };
        report.push(format!("投资收益率: {:.2}%", roi));
        
        report.push("".to_string());
        report.push("=== 投资产品盈利分析报告结束 ===".to_string());
        
        report.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_models::Config;

    #[test]
    fn test_generate_audit_summary() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);
        base.personal_balance = Decimal::from(5000);
        base.company_balance = Decimal::from(3000);
        base.total_misappropriation = Decimal::from(2000);
        base.total_advance_payment = Decimal::from(1000);
        base.update_total_balance();

        let summary = SummaryGenerator::generate_audit_summary(&base);
        
        assert_eq!(summary.personal_balance, Decimal::from(5000));
        assert_eq!(summary.company_balance, Decimal::from(3000));
        assert_eq!(summary.total_balance, Decimal::from(8000));
        assert_eq!(summary.total_misappropriation, Decimal::from(2000));
        assert_eq!(summary.total_advance_payment, Decimal::from(1000));
        assert_eq!(summary.funding_gap, Decimal::from(1000)); // 2000 - 0 - 1000
    }

    #[test]
    fn test_generate_detailed_summary_text() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);
        base.personal_balance = Decimal::from(6000);
        base.company_balance = Decimal::from(4000);
        base.total_misappropriation = Decimal::from(3000);
        base.investment_product_count = 2;
        base.update_total_balance();

        let summary_text = SummaryGenerator::generate_detailed_summary_text(&base, "FIFO");
        
        assert!(summary_text.contains("=== FIFO 资金追踪审计摘要 ==="));
        assert!(summary_text.contains("个人余额: ¥6000.00"));
        assert!(summary_text.contains("公司余额: ¥4000.00"));
        assert!(summary_text.contains("总余额: ¥10000.00"));
        assert!(summary_text.contains("累计挪用金额: ¥3000.00"));
        assert!(summary_text.contains("投资产品数量: 2"));
    }

    #[test]
    fn test_generate_off_site_records_csv_empty() {
        let config = Config::new();
        let base = TrackerBase::new(config);

        let csv = SummaryGenerator::generate_off_site_records_csv(&base);
        assert_eq!(csv, "无场外资金池记录");
    }

    #[test]
    fn test_generate_investment_profit_report_empty() {
        let config = Config::new();
        let base = TrackerBase::new(config);

        let report = SummaryGenerator::generate_investment_profit_report(&base);
        assert_eq!(report, "无投资产品数据");
    }
}