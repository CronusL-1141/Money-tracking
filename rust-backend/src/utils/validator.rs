//! # 数据验证模块
//! 
//! 提供数据验证功能。

/// 数据验证器
pub struct Validator;

impl Validator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self
    }

    /// 验证金额有效性
    pub fn validate_amount(&self, amount: f64) -> bool {
        amount.is_finite() && amount >= 0.0
    }

    /// 验证日期字符串
    pub fn validate_date(&self, date_str: &str) -> bool {
        !date_str.trim().is_empty()
    }

    /// 验证资金属性
    pub fn validate_fund_attribute(&self, attr: &str) -> bool {
        !attr.trim().is_empty()
    }
}
