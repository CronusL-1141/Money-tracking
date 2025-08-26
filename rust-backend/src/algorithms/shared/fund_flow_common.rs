//! 共同资金流处理模块
//! 
//! 包含FIFO和差额计算法共同的资金流入处理逻辑
//! 对应Python版本的资金流入分配规则

use super::tracker_base::TrackerBase;
use super::behavior_analyzer::BehaviorAnalyzer;
use super::investment_pool::InvestmentPoolManager;
// use crate::data_models::Config; // 暂时未使用
use rust_decimal::Decimal;
use chrono::NaiveDateTime;

/// 资金流向处理器
/// 
/// 处理两种算法共同的资金流向逻辑
pub struct FundFlowCommon;

impl FundFlowCommon {
    /// 处理资金流入
    /// 
    /// 对应Python版本的`处理资金流入`方法
    /// 收入分配规则与FIFO和差额计算法完全相同
    /// 
    /// # Arguments
    /// * `base` - 追踪器基础状态
    /// * `amount` - 流入金额
    /// * `fund_attribute` - 资金属性描述
    /// * `transaction_date` - 交易日期
    /// 
    /// # Returns
    /// (个人占比, 公司占比, 行为性质)
    pub fn process_fund_inflow(
        base: &mut TrackerBase,
        amount: Decimal,
        fund_attribute: &str,
        _transaction_date: Option<NaiveDateTime>, // 暂时未使用
    ) -> (Decimal, Decimal, String) {
        if amount <= Decimal::ZERO {
            return (Decimal::ZERO, Decimal::ZERO, "".to_string());
        }

        // 判断资金属性类型
        let fund_attribute_str = fund_attribute.trim();

        if base.config.is_personal_fund(fund_attribute_str) {
            // 个人资金
            base.personal_balance += amount;
            base.update_total_balance();
            return (Decimal::ONE, Decimal::ZERO, format!("个人资金流入：{:.2}", amount));
            
        } else if base.config.is_company_fund(fund_attribute_str) {
            // 公司资金
            base.company_balance += amount;
            base.update_total_balance();
            return (Decimal::ZERO, Decimal::ONE, format!("公司资金流入：{:.2}", amount));
            
        } else {
            // 其他情况，按当前余额比例分配
            let total_balance = base.personal_balance + base.company_balance;
            if total_balance == Decimal::ZERO {
                // 如果总余额为0，按默认规则处理
                // 默认按1:1分配
                let personal_amount = amount / Decimal::from(2);
                let company_amount = amount / Decimal::from(2);
                base.personal_balance += personal_amount;
                base.company_balance += company_amount;
                base.update_total_balance();
                return (
                    Decimal::new(5, 1), // 0.5
                    Decimal::new(5, 1), // 0.5
                    format!("混合资金流入：个人{:.2}，公司{:.2}", personal_amount, company_amount)
                );
            } else {
                let personal_ratio = base.personal_balance / total_balance;
                let company_ratio = base.company_balance / total_balance;
                let personal_amount = amount * personal_ratio;
                let company_amount = amount * company_ratio;
                base.personal_balance += personal_amount;
                base.company_balance += company_amount;
                base.update_total_balance();
                return (
                    personal_ratio,
                    company_ratio,
                    format!("混合资金流入：个人{:.2}，公司{:.2}", personal_amount, company_amount)
                );
            }
        }
    }

    /// 处理投资产品申购
    /// 
    /// 对应Python版本的投资产品申购逻辑，两种算法完全相同
    /// 
    /// # Arguments
    /// * `base` - 追踪器基础状态
    /// * `analyzer` - 行为分析器（用于投资挪用检测）
    /// * `amount` - 申购金额
    /// * `fund_attribute` - 投资产品属性
    /// * `transaction_date` - 交易日期
    /// * `deduction_fn` - 资金扣除函数（FIFO用队列，差额计算法用余额优先）
    /// 
    /// # Returns
    /// (个人占比, 公司占比, 行为性质)
    pub fn process_investment_purchase<F>(
        base: &mut TrackerBase,
        _analyzer: &BehaviorAnalyzer, // 暂时未使用
        amount: Decimal,
        fund_attribute: &str,
        _transaction_date: Option<NaiveDateTime>, // 暂时未使用
        deduction_fn: F,
    ) -> Result<(Decimal, Decimal, String), String>
    where
        F: FnOnce(&mut TrackerBase, Decimal) -> (Decimal, Decimal), // 返回(个人扣除, 公司扣除)
    {
        // 检查是否有足够的资金
        let total_available = base.personal_balance + base.company_balance;
        if total_available <= Decimal::ZERO {
            return Err(format!("资金池已空，无法申购{:.2}", amount));
        }

        // 添加误差容忍度处理
        let balance_tolerance = Decimal::new(1, 2); // 0.01
        let funding_gap = amount - total_available;
        let actual_amount = if funding_gap > balance_tolerance {
            total_available // 只扣除现有余额
        } else {
            amount
        };

        // 通过传入的扣除函数获取个人和公司扣除金额
        let (personal_deduction, company_deduction) = deduction_fn(base, actual_amount);

        // 计算占比
        let total_deducted = personal_deduction + company_deduction;
        let (personal_ratio, company_ratio) = if total_deducted > Decimal::ZERO {
            (personal_deduction / total_deducted, company_deduction / total_deducted)
        } else {
            (Decimal::ZERO, Decimal::ZERO)
        };

        // 投资是个人行为，使用公司资金就是挪用
        let mut _investment_misappropriation = Decimal::ZERO; // 暂时未使用
        if company_deduction > Decimal::ZERO {
            _investment_misappropriation = company_deduction;
            base.total_misappropriation += company_deduction;
            base.total_misappropriation = base.format_decimal(base.total_misappropriation);
        }

        // 构造行为性质描述
        let mut behavior_descriptions = Vec::new();
        if company_deduction > Decimal::ZERO {
            behavior_descriptions.push(format!("投资挪用：{:.2}", company_deduction));
        }
        if personal_deduction > Decimal::ZERO {
            behavior_descriptions.push(format!("个人投资：{:.2}", personal_deduction));
        }

        let behavior_nature = if behavior_descriptions.is_empty() {
            "无投资".to_string()
        } else {
            behavior_descriptions.join("；")
        };

        // 更新投资产品资金池
        InvestmentPoolManager::update_investment_pool(
            base,
            fund_attribute,
            actual_amount,
            personal_ratio,
            company_ratio,
            _transaction_date,
        );

        let prefix = fund_attribute.split('-').next().unwrap_or("投资");
        Ok((
            personal_ratio, 
            company_ratio, 
            format!("{}申购-{}：{}", prefix, fund_attribute, behavior_nature)
        ))
    }

    /// 处理普通资金流出（共同的行为分析逻辑）
    /// 
    /// 对应Python版本的行为分析器集成机制
    /// 
    /// # Arguments
    /// * `base` - 追踪器基础状态
    /// * `analyzer` - 行为分析器
    /// * `fund_attribute` - 资金属性
    /// * `personal_deduction` - 个人资金扣除金额
    /// * `company_deduction` - 公司资金扣除金额
    /// * `original_amount` - 原始交易金额
    /// 
    /// # Returns
    /// 完整的行为性质描述
    pub fn analyze_common_outflow_behavior(
        base: &mut TrackerBase,
        analyzer: &mut BehaviorAnalyzer,
        fund_attribute: &str,
        personal_deduction: Decimal,
        company_deduction: Decimal,
        original_amount: Decimal,
    ) -> String {
        // 分析行为性质
        let base_behavior = analyzer.analyze_behavior_nature(
            fund_attribute,
            personal_deduction,
            company_deduction,
            original_amount,
            &base.config,
        );

        // 处理行为分析器增量累计（避免重复计算）
        base.process_analyzer_incremental(
            analyzer.total_misappropriation,
            analyzer.total_advance_payment,
        );

        // 计算资金缺口
        let funding_gap = original_amount - personal_deduction - company_deduction;
        let balance_tolerance = Decimal::new(1, 2); // 0.01

        // 添加资金不足的说明
        if funding_gap > balance_tolerance {
            format!("{}；资金缺口：{:.2}", base_behavior, funding_gap)
        } else {
            base_behavior
        }
    }

    /// 检查可用资金并处理不足情况
    /// 
    /// # Arguments
    /// * `base` - 追踪器基础状态
    /// * `amount` - 请求金额
    /// 
    /// # Returns
    /// (实际可用金额, 资金缺口)
    pub fn check_available_funds(base: &TrackerBase, amount: Decimal) -> (Decimal, Decimal) {
        let total_available = base.personal_balance + base.company_balance;
        if total_available <= Decimal::ZERO {
            return (Decimal::ZERO, amount);
        }

        let actual_amount = amount.min(total_available);
        let funding_gap = amount - actual_amount;
        
        (actual_amount, funding_gap)
    }

    /// 更新余额并处理边界情况
    /// 
    /// # Arguments
    /// * `base` - 追踪器基础状态
    /// * `personal_deduction` - 个人资金扣除金额
    /// * `company_deduction` - 公司资金扣除金额
    pub fn update_balances_with_deduction(
        base: &mut TrackerBase,
        personal_deduction: Decimal,
        company_deduction: Decimal,
    ) {
        // 更新余额，确保不会出现负数
        base.personal_balance = (base.personal_balance - personal_deduction).max(Decimal::ZERO);
        base.company_balance = (base.company_balance - company_deduction).max(Decimal::ZERO);
        base.update_total_balance();
    }

    /// 计算占比（基于原始金额）
    /// 
    /// # Arguments
    /// * `personal_deduction` - 个人扣除金额
    /// * `company_deduction` - 公司扣除金额
    /// * `original_amount` - 原始金额
    /// 
    /// # Returns
    /// (个人占比, 公司占比)
    pub fn calculate_ratios(
        personal_deduction: Decimal,
        company_deduction: Decimal,
        original_amount: Decimal,
    ) -> (Decimal, Decimal) {
        if original_amount > Decimal::ZERO {
            (personal_deduction / original_amount, company_deduction / original_amount)
        } else {
            (Decimal::ZERO, Decimal::ZERO)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_fund_inflow_personal() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);

        let (personal_ratio, company_ratio, behavior) = FundFlowCommon::process_fund_inflow(
            &mut base,
            Decimal::from(10000),
            "个人应收",
            None,
        );

        assert_eq!(personal_ratio, Decimal::ONE);
        assert_eq!(company_ratio, Decimal::ZERO);
        assert!(behavior.contains("个人资金流入：10000.00"));
        assert_eq!(base.personal_balance, Decimal::from(10000));
        assert_eq!(base.total_balance, Decimal::from(10000));
    }

    #[test]
    fn test_process_fund_inflow_company() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);

        let (personal_ratio, company_ratio, behavior) = FundFlowCommon::process_fund_inflow(
            &mut base,
            Decimal::from(8000),
            "公司应收",
            None,
        );

        assert_eq!(personal_ratio, Decimal::ZERO);
        assert_eq!(company_ratio, Decimal::ONE);
        assert!(behavior.contains("公司资金流入：8000.00"));
        assert_eq!(base.company_balance, Decimal::from(8000));
        assert_eq!(base.total_balance, Decimal::from(8000));
    }

    #[test]
    fn test_process_fund_inflow_mixed_empty_pool() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);

        let (personal_ratio, company_ratio, behavior) = FundFlowCommon::process_fund_inflow(
            &mut base,
            Decimal::from(6000),
            "其他资金",
            None,
        );

        assert_eq!(personal_ratio, Decimal::new(5, 1)); // 0.5
        assert_eq!(company_ratio, Decimal::new(5, 1));  // 0.5
        assert!(behavior.contains("混合资金流入"));
        assert_eq!(base.personal_balance, Decimal::from(3000));
        assert_eq!(base.company_balance, Decimal::from(3000));
        assert_eq!(base.total_balance, Decimal::from(6000));
    }

    #[test]
    fn test_process_fund_inflow_mixed_with_existing_balance() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);
        base.personal_balance = Decimal::from(6000);
        base.company_balance = Decimal::from(4000);
        base.update_total_balance();

        let (personal_ratio, company_ratio, behavior) = FundFlowCommon::process_fund_inflow(
            &mut base,
            Decimal::from(5000),
            "其他资金",
            None,
        );

        // 应该按6:4的比例分配
        assert_eq!(personal_ratio, Decimal::new(6, 1)); // 0.6
        assert_eq!(company_ratio, Decimal::new(4, 1));  // 0.4
        assert!(behavior.contains("混合资金流入"));
        assert_eq!(base.personal_balance, Decimal::from(9000)); // 6000 + 3000
        assert_eq!(base.company_balance, Decimal::from(6000));  // 4000 + 2000
    }

    #[test]
    fn test_check_available_funds() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);
        base.personal_balance = Decimal::from(3000);
        base.company_balance = Decimal::from(2000);
        base.update_total_balance();

        let (actual_amount, funding_gap) = FundFlowCommon::check_available_funds(&base, Decimal::from(8000));
        assert_eq!(actual_amount, Decimal::from(5000)); // 可用总额
        assert_eq!(funding_gap, Decimal::from(3000));   // 缺口

        let (actual_amount2, funding_gap2) = FundFlowCommon::check_available_funds(&base, Decimal::from(3000));
        assert_eq!(actual_amount2, Decimal::from(3000)); // 足够
        assert_eq!(funding_gap2, Decimal::ZERO);         // 无缺口
    }

    #[test]
    fn test_calculate_ratios() {
        let (personal_ratio, company_ratio) = FundFlowCommon::calculate_ratios(
            Decimal::from(3000),
            Decimal::from(2000),
            Decimal::from(5000),
        );

        assert_eq!(personal_ratio, Decimal::new(6, 1)); // 0.6
        assert_eq!(company_ratio, Decimal::new(4, 1));  // 0.4

        // 测试除零情况
        let (personal_ratio2, company_ratio2) = FundFlowCommon::calculate_ratios(
            Decimal::from(1000),
            Decimal::from(2000),
            Decimal::ZERO,
        );

        assert_eq!(personal_ratio2, Decimal::ZERO);
        assert_eq!(company_ratio2, Decimal::ZERO);
    }

    #[test]
    fn test_update_balances_with_deduction() {
        let config = Config::new();
        let mut base = TrackerBase::new(config);
        base.personal_balance = Decimal::from(5000);
        base.company_balance = Decimal::from(3000);
        base.update_total_balance();

        FundFlowCommon::update_balances_with_deduction(
            &mut base,
            Decimal::from(2000),
            Decimal::from(1000),
        );

        assert_eq!(base.personal_balance, Decimal::from(3000));
        assert_eq!(base.company_balance, Decimal::from(2000));
        assert_eq!(base.total_balance, Decimal::from(5000));

        // 测试不会出现负数
        FundFlowCommon::update_balances_with_deduction(
            &mut base,
            Decimal::from(5000),
            Decimal::from(5000),
        );

        assert_eq!(base.personal_balance, Decimal::ZERO);
        assert_eq!(base.company_balance, Decimal::ZERO);
        assert_eq!(base.total_balance, Decimal::ZERO);
    }
}