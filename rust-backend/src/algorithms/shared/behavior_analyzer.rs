//! 行为分析器实现
//! 
//! 负责分析交易行为性质，判断是否构成挪用或垫付
//! 完全对应Python版本的BehaviorAnalyzer类功能

use crate::data_models::Config;
use rust_decimal::Decimal;

/// 行为分析器
/// 
/// 分析交易的行为性质，判断挪用、垫付等情况
/// 完全对应Python版本的BehaviorAnalyzer类
#[derive(Debug, Clone)]
pub struct BehaviorAnalyzer {
    /// 累计挪用金额（个人使用公司资金）
    pub total_misappropriation: Decimal,
    /// 累计垫付金额（公司使用个人资金）
    pub total_advance_payment: Decimal,
}

impl BehaviorAnalyzer {
    /// 创建新的行为分析器
    pub fn new() -> Self {
        Self {
            total_misappropriation: Decimal::ZERO,
            total_advance_payment: Decimal::ZERO,
        }
    }

    /// 分析交易的行为性质
    /// 
    /// 对应Python版本的`分析行为性质`方法
    /// 
    /// # Arguments
    /// * `fund_attribute` - 资金属性描述
    /// * `personal_deduction` - 个人资金扣除金额
    /// * `company_deduction` - 公司资金扣除金额
    /// * `total_amount` - 总交易金额
    /// * `config` - 配置对象
    /// 
    /// # Returns
    /// 行为性质描述
    pub fn analyze_behavior_nature(
        &mut self,
        fund_attribute: &str,
        personal_deduction: Decimal,
        company_deduction: Decimal,
        total_amount: Decimal,
        config: &Config,
    ) -> String {
        if total_amount <= Decimal::ZERO {
            return "无交易".to_string();
        }

        let mut behavior_descriptions = Vec::new();
        
        // 判断资金属性类型
        let fund_type = self.determine_fund_attribute_type(fund_attribute, config);

        match fund_type {
            FundAttributeType::Personal => {
                // 个人应付/支出
                if company_deduction > Decimal::ZERO {
                    // 个人支出使用了公司资金 - 构成挪用
                    self.total_misappropriation += company_deduction;
                    self.total_misappropriation = self.format_number(self.total_misappropriation);
                    behavior_descriptions.push(format!("挪用：{:.2}", company_deduction));
                }
                if personal_deduction > Decimal::ZERO {
                    // 个人支出使用了个人资金 - 正常
                    behavior_descriptions.push(format!("个人支付：{:.2}", personal_deduction));
                }
            }
            FundAttributeType::Company => {
                // 公司应付/支出
                if personal_deduction > Decimal::ZERO {
                    // 公司支出使用了个人资金 - 构成垫付
                    self.total_advance_payment += personal_deduction;
                    self.total_advance_payment = self.format_number(self.total_advance_payment);
                    behavior_descriptions.push(format!("垫付：{:.2}", personal_deduction));
                }
                if company_deduction > Decimal::ZERO {
                    // 公司支出使用了公司资金 - 正常
                    behavior_descriptions.push(format!("公司支付：{:.2}", company_deduction));
                }
            }
            FundAttributeType::Other => {
                // 其他类型
                if personal_deduction > Decimal::ZERO {
                    behavior_descriptions.push(format!("个人支付：{:.2}", personal_deduction));
                }
                if company_deduction > Decimal::ZERO {
                    behavior_descriptions.push(format!("公司支付：{:.2}", company_deduction));
                }
            }
        }

        if behavior_descriptions.is_empty() {
            "无明确行为".to_string()
        } else {
            behavior_descriptions.join("；")
        }
    }

    /// 分析投资行为
    /// 
    /// 对应Python版本的`分析投资行为`方法
    /// 投资是个人行为，使用公司资金就是挪用
    /// 
    /// # Arguments
    /// * `personal_deduction` - 个人资金扣除金额
    /// * `company_deduction` - 公司资金扣除金额
    /// 
    /// # Returns
    /// (行为性质, 挪用金额)
    pub fn analyze_investment_behavior(
        &self,
        personal_deduction: Decimal,
        company_deduction: Decimal,
    ) -> (String, Decimal) {
        let mut misappropriation_amount = Decimal::ZERO;
        let mut behavior_descriptions = Vec::new();

        if company_deduction > Decimal::ZERO {
            // 投资使用公司资金构成挪用
            misappropriation_amount = company_deduction;
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

        (behavior_nature, misappropriation_amount)
    }

    /// 分析收益分配
    /// 
    /// 对应Python版本的`分析收益分配`方法
    /// 
    /// # Arguments
    /// * `profit` - 总收益金额
    /// * `personal_ratio` - 个人资金占比
    /// * `company_ratio` - 公司资金占比
    /// 
    /// # Returns
    /// (个人收益, 公司收益, 非法所得)
    pub fn analyze_profit_distribution(
        &self,
        profit: Decimal,
        personal_ratio: Decimal,
        company_ratio: Decimal,
    ) -> (Decimal, Decimal, Decimal) {
        if profit <= Decimal::ZERO {
            return (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
        }

        let personal_profit = profit * personal_ratio;
        let company_profit = profit * company_ratio;
        let illegal_gain = personal_profit + company_profit;

        let formatted_personal_profit = self.format_number(personal_profit);
        let formatted_company_profit = self.format_number(company_profit);
        let formatted_illegal_gain = self.format_number(illegal_gain);

        (formatted_personal_profit, formatted_company_profit, formatted_illegal_gain)
    }

    /// 获取累计统计
    /// 
    /// 对应Python版本的`获取累计统计`方法
    /// 
    /// # Returns
    /// (累计挪用金额, 累计垫付金额)
    pub fn get_cumulative_stats(&self) -> (Decimal, Decimal) {
        (self.total_misappropriation, self.total_advance_payment)
    }

    /// 重置统计
    /// 
    /// 对应Python版本的`重置统计`方法
    pub fn reset_stats(&mut self) {
        self.total_misappropriation = Decimal::ZERO;
        self.total_advance_payment = Decimal::ZERO;
    }

    /// 判断资金属性类型
    /// 
    /// 对应Python版本的`_判断资金属性类型`方法
    fn determine_fund_attribute_type(&self, fund_attribute: &str, config: &Config) -> FundAttributeType {
        let fund_attribute_str = fund_attribute.trim();

        if config.is_personal_fund(fund_attribute_str) {
            FundAttributeType::Personal
        } else if config.is_company_fund(fund_attribute_str) {
            FundAttributeType::Company
        } else {
            FundAttributeType::Other
        }
    }

    /// 数值格式化处理
    /// 
    /// 对应Python版本的Config.format_number功能
    fn format_number(&self, value: Decimal) -> Decimal {
        // 处理极小值，避免科学计数法显示
        if value.abs() < Decimal::new(1, 10) { // 小于0.0000000001
            Decimal::ZERO
        } else {
            value.round_dp(2) // 保留2位小数
        }
    }
}

/// 资金属性类型
/// 
/// 对应Python版本中的资金属性分类逻辑
#[derive(Debug, Clone, PartialEq)]
enum FundAttributeType {
    /// 个人资金
    Personal,
    /// 公司资金
    Company,
    /// 其他类型
    Other,
}

impl Default for BehaviorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_analyzer_creation() {
        let analyzer = BehaviorAnalyzer::new();
        
        assert_eq!(analyzer.total_misappropriation, Decimal::ZERO);
        assert_eq!(analyzer.total_advance_payment, Decimal::ZERO);
    }

    #[test]
    fn test_analyze_behavior_nature_personal_misappropriation() {
        let mut analyzer = BehaviorAnalyzer::new();
        let config = Config::new();

        let behavior = analyzer.analyze_behavior_nature(
            "个人应付",
            Decimal::from(5000),
            Decimal::from(3000),
            Decimal::from(8000),
            &config,
        );

        assert!(behavior.contains("挪用：3000.00"));
        assert!(behavior.contains("个人支付：5000.00"));
        assert_eq!(analyzer.total_misappropriation, Decimal::from(3000));
    }

    #[test]
    fn test_analyze_behavior_nature_company_advance() {
        let mut analyzer = BehaviorAnalyzer::new();
        let config = Config::new();

        let behavior = analyzer.analyze_behavior_nature(
            "公司应付",
            Decimal::from(2000),
            Decimal::from(6000),
            Decimal::from(8000),
            &config,
        );

        assert!(behavior.contains("垫付：2000.00"));
        assert!(behavior.contains("公司支付：6000.00"));
        assert_eq!(analyzer.total_advance_payment, Decimal::from(2000));
    }

    #[test]
    fn test_analyze_investment_behavior() {
        let analyzer = BehaviorAnalyzer::new();

        let (behavior, misappropriation) = analyzer.analyze_investment_behavior(
            Decimal::from(10000),
            Decimal::from(5000),
        );

        assert!(behavior.contains("个人投资：10000.00"));
        assert!(behavior.contains("投资挪用：5000.00"));
        assert_eq!(misappropriation, Decimal::from(5000));
    }

    #[test]
    fn test_analyze_profit_distribution() {
        let analyzer = BehaviorAnalyzer::new();

        let (personal_profit, company_profit, illegal_gain) = analyzer.analyze_profit_distribution(
            Decimal::from(1000),
            Decimal::new(6, 1), // 0.6
            Decimal::new(4, 1), // 0.4
        );

        assert_eq!(personal_profit, Decimal::from(600));
        assert_eq!(company_profit, Decimal::from(400));
        assert_eq!(illegal_gain, Decimal::from(1000));
    }

    #[test]
    fn test_cumulative_stats() {
        let mut analyzer = BehaviorAnalyzer::new();
        let config = Config::new();

        // 模拟一些交易
        analyzer.analyze_behavior_nature("个人应付", Decimal::from(1000), Decimal::from(2000), Decimal::from(3000), &config);
        analyzer.analyze_behavior_nature("公司应付", Decimal::from(1500), Decimal::from(500), Decimal::from(2000), &config);

        let (misappropriation, advance_payment) = analyzer.get_cumulative_stats();
        assert_eq!(misappropriation, Decimal::from(2000)); // 挪用总额
        assert_eq!(advance_payment, Decimal::from(1500));  // 垫付总额
    }

    #[test]
    fn test_reset_stats() {
        let mut analyzer = BehaviorAnalyzer::new();
        let config = Config::new();

        // 先产生一些统计数据
        analyzer.analyze_behavior_nature("个人应付", Decimal::from(1000), Decimal::from(2000), Decimal::from(3000), &config);
        assert!(analyzer.total_misappropriation > Decimal::ZERO);

        // 重置后应该清零
        analyzer.reset_stats();
        assert_eq!(analyzer.total_misappropriation, Decimal::ZERO);
        assert_eq!(analyzer.total_advance_payment, Decimal::ZERO);
    }
}