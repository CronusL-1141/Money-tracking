//! # 资金工具模块
//! 
//! 提供资金属性判断和处理功能。

/// 判断是否为个人资金
pub fn is_personal_fund(fund_attr: &str) -> bool {
    let attr = fund_attr.trim().to_lowercase();
    attr.contains("个人") || attr == "personal"
}

/// 判断是否为公司资金
pub fn is_company_fund(fund_attr: &str) -> bool {
    let attr = fund_attr.trim().to_lowercase();
    attr.contains("公司") || attr == "company"
}

/// 判断是否为投资产品
pub fn is_investment_product(fund_attr: &str) -> bool {
    let attr = fund_attr.trim().to_lowercase();
    attr.contains("理财") || attr.contains("投资") || attr.contains("产品")
}

/// 格式化金额显示
pub fn format_amount(amount: f64) -> String {
    if amount.abs() >= 10000.0 {
        format!("{:.2}万", amount / 10000.0)
    } else {
        format!("{:.2}", amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fund_classification() {
        assert!(is_personal_fund("个人"));
        assert!(is_company_fund("公司"));
        assert!(is_investment_product("理财产品"));
        
        assert_eq!(format_amount(15000.0), "1.50万");
        assert_eq!(format_amount(500.0), "500.00");
    }
}
