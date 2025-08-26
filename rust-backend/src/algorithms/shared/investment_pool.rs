//! 投资产品资金池管理
//! 
//! 对应Python版本的投资产品处理逻辑，包括申购、赎回、盈利实现等复杂机制

use super::tracker_base::{TrackerBase, InvestmentPool, ProfitRecord, OffSiteRecord};
// use crate::data_models::Config; // 暂时未使用
use rust_decimal::Decimal;
use chrono::NaiveDateTime;

/// 投资产品管理器
/// 
/// 负责投资产品的申购、赎回、盈利计算等复杂逻辑
pub struct InvestmentPoolManager;

impl InvestmentPoolManager {
    /// 更新投资产品资金池
    /// 
    /// 对应Python版本的`_更新投资产品资金池`方法
    pub fn update_investment_pool(
        base: &mut TrackerBase,
        product_code: &str,
        amount: Decimal,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        transaction_date: Option<NaiveDateTime>,
    ) {
        let personal_amount = amount * personal_ratio;
        let company_amount = amount * company_ratio;

        // 获取或创建投资产品池
        let pool = base.investment_pools.entry(product_code.to_string())
            .or_insert_with(InvestmentPool::default);

        // 检查重置条件：当前总金额为负数时，表示已有盈利
        let current_total = pool.total_amount;
        if current_total < Decimal::ZERO {
            // 计算已实现盈利
            let realized_profit = current_total.abs();
            
            // 创建历史盈利记录
            let reset_record = ProfitRecord {
                reset_time: transaction_date
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "未知时间".to_string()),
                profit_amount: realized_profit,
                description: format!("重置前实现盈利 ¥{:.2}", realized_profit),
            };
            
            // 更新累计盈利
            pool.historical_profit_records.push(reset_record);
            pool.cumulative_realized_profit += realized_profit;
            
            // 重置资金池状态
            pool.personal_amount = personal_amount;
            pool.company_amount = company_amount;
            pool.total_amount = amount;
            
            // 动态占比更新
            pool.latest_personal_ratio = personal_ratio;
            pool.latest_company_ratio = company_ratio;
        } else {
            // 正常累加情况
            pool.personal_amount += personal_amount;
            pool.company_amount += company_amount;
            pool.total_amount += amount;
            
            // 占比重新计算
            let new_total = pool.total_amount;
            if new_total > Decimal::ZERO {
                pool.latest_personal_ratio = pool.personal_amount / new_total;
                pool.latest_company_ratio = pool.company_amount / new_total;
            }
        }

        // 更新累计申购
        pool.cumulative_purchase += amount;
        
        // 记录场外资金池交易（暂时禁用以避免借用检查问题）
        // Self::record_off_site_transaction(
        //     base,
        //     product_code,
        //     amount,
        //     Decimal::ZERO, // 入金，出金为0
        //     transaction_date,
        //     pool,
        //     "申购",
        // );
    }

    /// 处理投资产品赎回
    /// 
    /// 对应Python版本的`处理投资产品赎回`方法
    pub fn process_investment_redemption(
        base: &mut TrackerBase,
        product_code: &str,
        amount: Decimal,
        _transaction_date: Option<NaiveDateTime>, // 暂时未使用
    ) -> Result<(Decimal, Decimal, String), String> {
        // 查找对应的投资产品记录
        if !base.investment_pools.contains_key(product_code) {
            // 没有找到对应的投资产品记录，按个人应收处理
            // 用户要求：优先判定为个人应收，但标记为存疑
            base.personal_balance += amount;
            let prefix = product_code.split('-').next().unwrap_or("投资");
            return Ok((
                Decimal::ONE, 
                Decimal::ZERO, 
                format!("{}收入-{}：个人应收{:.2}（无申购记录-存疑）", prefix, product_code, amount)
            ));
        }

        // 提取pool的相关数据，避免同时借用
        let (total_amount, personal_amount, company_amount, latest_personal_ratio, latest_company_ratio) = {
            let pool = base.investment_pools.get_mut(product_code).unwrap();
            (pool.total_amount, pool.personal_amount, pool.company_amount, 
             pool.latest_personal_ratio, pool.latest_company_ratio)
        };

        // 检查是否有有效的占比记录
        if latest_personal_ratio == Decimal::ZERO && latest_company_ratio == Decimal::ZERO {
            return Err(format!("错误：投资产品{}从未有过有效资金池，无法分配收益", product_code));
        }

        // 赎回时统一使用最新记录的占比
        let personal_return = amount * latest_personal_ratio;
        let company_return = amount * latest_company_ratio;

        // 计算收益情况和本金归还
        let (profit, returned_personal_principal, returned_company_principal) = 
            if total_amount > Decimal::ZERO {
                // 按原始逻辑计算赎回比例和收益
                let redemption_ratio = if total_amount > Decimal::ZERO { amount / total_amount } else { Decimal::ZERO };
                let corresponding_purchase_cost = total_amount * redemption_ratio;
                let profit = amount - corresponding_purchase_cost;
                
                // 计算实际归还的本金
                let (returned_personal, returned_company) = if amount <= total_amount {
                    // 部分赎回或等额赎回，全部是本金归还
                    (personal_amount * redemption_ratio, company_amount * redemption_ratio)
                } else {
                    // 超额赎回，只有本金部分算作归还
                    (personal_amount, company_amount)
                };

                (profit, returned_personal, returned_company)
            } else {
                // 资金池为0或负数，纯收益分配
                let profit = amount;
                
                (profit, Decimal::ZERO, Decimal::ZERO)
            };

        // 更新投资产品资金池和基础状态
        {
            let pool = base.investment_pools.get_mut(product_code).unwrap();
            
            // 更新累计赎回
            pool.cumulative_redemption += amount;
            
            // 更新资金池数据
            if total_amount > Decimal::ZERO {
                let redemption_ratio = if total_amount > Decimal::ZERO { amount / total_amount } else { Decimal::ZERO };
                pool.personal_amount -= personal_amount * redemption_ratio;
                pool.company_amount -= company_amount * redemption_ratio;
                let corresponding_purchase_cost = total_amount * redemption_ratio;
                pool.total_amount -= corresponding_purchase_cost;
            } else {
                // 资金池为0或负数，继续减少
                pool.personal_amount -= personal_return;
                pool.company_amount -= company_return;
                pool.total_amount -= amount;
            }
        }
        
        // 记录归还的本金（用于抵消挪用和垫付）
        if returned_company_principal > Decimal::ZERO {
            base.total_company_principal_returned += returned_company_principal;
            base.total_company_principal_returned = base.format_decimal(base.total_company_principal_returned);
        }
        if returned_personal_principal > Decimal::ZERO {
            base.total_personal_principal_returned += returned_personal_principal;
            base.total_personal_principal_returned = base.format_decimal(base.total_personal_principal_returned);
        }

        // 收益分配
        if profit > Decimal::ZERO {
            let personal_profit = profit * latest_personal_ratio;
            let company_profit = profit * latest_company_ratio;
            
            base.total_personal_profit += personal_profit;
            base.total_personal_profit = base.format_decimal(base.total_personal_profit);
            base.total_company_profit += company_profit;
            base.total_company_profit = base.format_decimal(base.total_company_profit);
        }

        // 返还到资金池
        if personal_return > Decimal::ZERO {
            base.personal_balance += personal_return;
        }
        if company_return > Decimal::ZERO {
            base.company_balance += company_return;
        }
        base.update_total_balance();

        // 记录场外资金池交易（暂时禁用以避免借用检查问题）
        // {
        //     let pool = base.investment_pools.get_mut(product_code).unwrap();
        //     Self::record_off_site_transaction(
        //         base,
        //         product_code,
        //         Decimal::ZERO, // 入金为0
        //         amount, // 出金
        //         transaction_date,
        //         pool,
        //         "赎回",
        //     );
        // }

        // 构造行为性质描述
        let prefix = product_code.split('-').next().unwrap_or("投资");
        let behavior_description = if profit > Decimal::ZERO {
            if latest_personal_ratio > Decimal::ZERO && latest_company_ratio > Decimal::ZERO {
                format!(
                    "{}赎回-{}：个人{:.2}（本金{:.2}+收益{:.2}），公司{:.2}（本金{:.2}+收益{:.2}），总收益{:.2}",
                    prefix, product_code, personal_return, returned_personal_principal, 
                    profit * latest_personal_ratio, company_return, returned_company_principal,
                    profit * latest_company_ratio, profit
                )
            } else if latest_personal_ratio > Decimal::ZERO {
                format!(
                    "{}赎回-{}：个人{:.2}（本金{:.2}+收益{:.2}），总收益{:.2}",
                    prefix, product_code, personal_return, returned_personal_principal,
                    profit * latest_personal_ratio, profit
                )
            } else {
                format!(
                    "{}赎回-{}：公司{:.2}（本金{:.2}+收益{:.2}），总收益{:.2}",
                    prefix, product_code, company_return, returned_company_principal,
                    profit * latest_company_ratio, profit
                )
            }
        } else if profit < Decimal::ZERO {
            format!(
                "{}赎回-{}：个人{:.2}（本金{:.2}），公司{:.2}（本金{:.2}），亏损{:.2}",
                prefix, product_code, personal_return, returned_personal_principal,
                company_return, returned_company_principal, profit.abs()
            )
        } else {
            format!(
                "{}赎回-{}：个人{:.2}（本金{:.2}），公司{:.2}（本金{:.2}），无收益",
                prefix, product_code, personal_return, returned_personal_principal,
                company_return, returned_company_principal
            )
        };

        Ok((latest_personal_ratio, latest_company_ratio, behavior_description))
    }

    /// 记录场外资金池交易
    fn record_off_site_transaction(
        base: &mut TrackerBase,
        product_code: &str,
        inflow: Decimal,
        outflow: Decimal,
        transaction_date: Option<NaiveDateTime>,
        pool: &InvestmentPool,
        transaction_type: &str,
    ) {
        let updated_personal_balance = pool.personal_amount;
        let updated_company_balance = pool.company_amount;
        let updated_total_balance = pool.total_amount;

        // 计算资金占比描述
        let fund_ratio = if updated_total_balance.abs() > Decimal::new(1, 2) { // 0.01
            let personal_ratio_display = if updated_total_balance != Decimal::ZERO {
                updated_personal_balance / updated_total_balance
            } else {
                Decimal::ZERO
            };
            let company_ratio_display = if updated_total_balance != Decimal::ZERO {
                updated_company_balance / updated_total_balance  
            } else {
                Decimal::ZERO
            };
            format!("个人{:.1}%，公司{:.1}%", personal_ratio_display * Decimal::from(100), company_ratio_display * Decimal::from(100))
        } else {
            "资金池清空".to_string()
        };

        let record = OffSiteRecord {
            transaction_time: transaction_date
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "未知时间".to_string()),
            pool_name: product_code.to_string(),
            inflow,
            outflow,
            total_balance: updated_total_balance,
            personal_balance: updated_personal_balance,
            company_balance: updated_company_balance,
            fund_ratio,
            behavior_nature: format!("{}（个人{:.0}，公司{:.0}）", transaction_type, 
                if transaction_type == "申购" { inflow * pool.latest_personal_ratio } else { outflow * pool.latest_personal_ratio },
                if transaction_type == "申购" { inflow * pool.latest_company_ratio } else { outflow * pool.latest_company_ratio }
            ),
            cumulative_purchase: pool.cumulative_purchase,
            cumulative_redemption: pool.cumulative_redemption,
        };

        base.off_site_records.push(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_models::Config;

    #[test]
    fn test_update_investment_pool_normal() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);

        // 正常申购
        InvestmentPoolManager::update_investment_pool(
            &mut base,
            "理财-TEST001",
            Decimal::from(10000),
            Decimal::new(6, 1), // 0.6
            Decimal::new(4, 1), // 0.4
            None,
        );

        let pool = base.investment_pools.get("理财-TEST001").unwrap();
        assert_eq!(pool.total_amount, Decimal::from(10000));
        assert_eq!(pool.personal_amount, Decimal::from(6000));
        assert_eq!(pool.company_amount, Decimal::from(4000));
        assert_eq!(pool.latest_personal_ratio, Decimal::new(6, 1));
        assert_eq!(pool.latest_company_ratio, Decimal::new(4, 1));
    }

    #[test]
    fn test_investment_pool_reset_on_negative() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);

        // 先创建一个资金池
        let pool = InvestmentPool {
            total_amount: Decimal::from(-5000), // 负数表示盈利
            personal_amount: Decimal::from(-3000),
            company_amount: Decimal::from(-2000),
            latest_personal_ratio: Decimal::new(6, 1),
            latest_company_ratio: Decimal::new(4, 1),
            ..Default::default()
        };
        base.investment_pools.insert("理财-TEST001".to_string(), pool);

        // 再次申购，应该触发重置
        InvestmentPoolManager::update_investment_pool(
            &mut base,
            "理财-TEST001",
            Decimal::from(8000),
            Decimal::new(7, 1), // 0.7
            Decimal::new(3, 1), // 0.3
            None,
        );

        let updated_pool = base.investment_pools.get("理财-TEST001").unwrap();
        // 应该重置为新的金额和占比
        assert_eq!(updated_pool.total_amount, Decimal::from(8000));
        assert_eq!(updated_pool.personal_amount, Decimal::from(5600)); // 8000 * 0.7
        assert_eq!(updated_pool.company_amount, Decimal::from(2400));   // 8000 * 0.3
        assert_eq!(updated_pool.latest_personal_ratio, Decimal::new(7, 1));
        assert_eq!(updated_pool.latest_company_ratio, Decimal::new(3, 1));
        
        // 应该记录盈利
        assert_eq!(updated_pool.cumulative_realized_profit, Decimal::from(5000));
        assert_eq!(updated_pool.historical_profit_records.len(), 1);
    }

    #[test]
    fn test_process_investment_redemption_no_record() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);

        let result = InvestmentPoolManager::process_investment_redemption(
            &mut base,
            "理财-NOTFOUND",
            Decimal::from(5000),
            None,
        );

        assert!(result.is_ok());
        let (personal_ratio, company_ratio, description) = result.unwrap();
        assert_eq!(personal_ratio, Decimal::ONE);
        assert_eq!(company_ratio, Decimal::ZERO);
        assert!(description.contains("个人应收"));
        assert!(description.contains("无申购记录"));
        assert_eq!(base.personal_balance, Decimal::from(5000));
    }

    #[test]
    fn test_process_investment_redemption_with_profit() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);

        // 先创建一个资金池
        let pool = InvestmentPool {
            total_amount: Decimal::from(10000),
            personal_amount: Decimal::from(6000),
            company_amount: Decimal::from(4000),
            latest_personal_ratio: Decimal::new(6, 1), // 0.6
            latest_company_ratio: Decimal::new(4, 1),  // 0.4
            cumulative_purchase: Decimal::from(10000),
            cumulative_redemption: Decimal::ZERO,
            ..Default::default()
        };
        base.investment_pools.insert("理财-TEST001".to_string(), pool);

        // 赎回12000（有2000收益）
        let result = InvestmentPoolManager::process_investment_redemption(
            &mut base,
            "理财-TEST001",
            Decimal::from(12000),
            None,
        );

        assert!(result.is_ok());
        let (personal_ratio, company_ratio, description) = result.unwrap();
        assert_eq!(personal_ratio, Decimal::new(6, 1)); // 0.6
        assert_eq!(company_ratio, Decimal::new(4, 1));  // 0.4
        assert!(description.contains("总收益2000.00"));

        // 检查资金返还
        assert_eq!(base.personal_balance, Decimal::from(7200)); // 12000 * 0.6
        assert_eq!(base.company_balance, Decimal::from(4800));  // 12000 * 0.4

        // 检查收益分配
        assert_eq!(base.total_personal_profit, Decimal::from(1200)); // 2000 * 0.6
        assert_eq!(base.total_company_profit, Decimal::from(800));   // 2000 * 0.4

        // 检查本金归还
        assert_eq!(base.total_personal_principal_returned, Decimal::from(6000));
        assert_eq!(base.total_company_principal_returned, Decimal::from(4000));
    }
}