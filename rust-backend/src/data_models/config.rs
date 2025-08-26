//! 配置管理数据模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 全局配置
/// 
/// 管理系统的所有配置参数和业务规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 数值处理配置
    pub numeric: NumericConfig,
    
    /// 资金属性识别规则
    pub fund_attributes: FundAttributeConfig,
    
    /// 投资产品规则
    pub investment_products: InvestmentProductConfig,
    
    /// 文件路径配置
    pub file_paths: FilePathConfig,
    
    /// Excel列映射配置
    pub excel_columns: ExcelColumnConfig,
}

impl Config {
    /// 创建默认配置
    pub fn new() -> Self {
        Self {
            numeric: NumericConfig::new(),
            fund_attributes: FundAttributeConfig::new(),
            investment_products: InvestmentProductConfig::new(),
            file_paths: FilePathConfig::new(),
            excel_columns: ExcelColumnConfig::new(),
        }
    }
    
    /// 从配置文件加载
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// 保存到配置文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// 判断是否为个人资金
    pub fn is_personal_fund(&self, fund_attribute: &str) -> bool {
        self.fund_attributes.personal_fund_keywords.iter()
            .any(|keyword| fund_attribute.contains(keyword))
    }
    
    /// 判断是否为公司资金
    pub fn is_company_fund(&self, fund_attribute: &str) -> bool {
        self.fund_attributes.company_fund_keywords.iter()
            .any(|keyword| fund_attribute.contains(keyword))
    }
    
    /// 判断是否为投资产品
    pub fn is_investment_product(&self, fund_attribute: &str) -> bool {
        self.investment_products.product_prefixes.iter()
            .any(|prefix| fund_attribute.starts_with(prefix))
    }
    
    /// 格式化数值精度
    pub fn format_number(&self, value: Decimal) -> Decimal {
        value.round_dp(self.numeric.decimal_places)
    }
    
    /// 检查余额容差
    pub fn is_balance_within_tolerance(&self, balance1: Decimal, balance2: Decimal) -> bool {
        (balance1 - balance2).abs() <= self.numeric.balance_tolerance
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// 数值处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericConfig {
    /// 余额计算容差
    pub balance_tolerance: Decimal,
    
    /// 小数位精度
    pub decimal_places: u32,
    
    /// 最小有效金额（小于此金额视为0）
    pub minimum_amount: Decimal,
}

impl NumericConfig {
    /// 创建默认数值配置
    pub fn new() -> Self {
        Self {
            balance_tolerance: Decimal::from_f64_retain(0.01).unwrap(),
            decimal_places: 2,
            minimum_amount: Decimal::from_f64_retain(0.01).unwrap(),
        }
    }
}

impl Default for NumericConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 资金属性配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAttributeConfig {
    /// 个人资金关键词
    pub personal_fund_keywords: HashSet<String>,
    
    /// 公司资金关键词
    pub company_fund_keywords: HashSet<String>,
}

impl FundAttributeConfig {
    /// 创建默认资金属性配置
    pub fn new() -> Self {
        let mut personal_keywords = HashSet::new();
        personal_keywords.insert("个人".to_string());
        personal_keywords.insert("个人应收".to_string());
        personal_keywords.insert("个人应付".to_string());
        
        let mut company_keywords = HashSet::new();
        company_keywords.insert("公司".to_string());
        company_keywords.insert("公司应收".to_string());
        company_keywords.insert("公司应付".to_string());
        
        Self {
            personal_fund_keywords: personal_keywords,
            company_fund_keywords: company_keywords,
        }
    }
}

impl Default for FundAttributeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 投资产品配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentProductConfig {
    /// 投资产品前缀列表
    pub product_prefixes: Vec<String>,
}

impl InvestmentProductConfig {
    /// 创建默认投资产品配置
    pub fn new() -> Self {
        Self {
            product_prefixes: vec![
                "理财-".to_string(),
                "投资-".to_string(),
                "保险-".to_string(),
                "关联银行卡-".to_string(),
                "资金池-".to_string(),
            ],
        }
    }
}

impl Default for InvestmentProductConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 文件路径配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePathConfig {
    /// 默认输入文件路径
    pub default_input_path: String,
    
    /// 默认输出目录
    pub default_output_dir: String,
    
    /// 日志目录
    pub log_dir: String,
    
    /// 配置文件目录
    pub config_dir: String,
}

impl FilePathConfig {
    /// 创建默认文件路径配置
    pub fn new() -> Self {
        Self {
            default_input_path: "流水.xlsx".to_string(),
            default_output_dir: "./".to_string(),
            log_dir: "logs/".to_string(),
            config_dir: "config/".to_string(),
        }
    }
}

impl Default for FilePathConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Excel列映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelColumnConfig {
    /// 交易日期列名
    pub transaction_date_column: String,
    
    /// 交易时间列名
    pub transaction_time_column: String,
    
    /// 交易收入金额列名
    pub income_amount_column: String,
    
    /// 交易支出金额列名
    pub expense_amount_column: String,
    
    /// 余额列名
    pub balance_column: String,
    
    /// 资金属性列名
    pub fund_attribute_column: String,
}

impl ExcelColumnConfig {
    /// 创建默认Excel列配置
    pub fn new() -> Self {
        Self {
            transaction_date_column: "交易日期".to_string(),
            transaction_time_column: "交易时间".to_string(),
            income_amount_column: "交易收入金额".to_string(),
            expense_amount_column: "交易支出金额".to_string(),
            balance_column: "余额".to_string(),
            fund_attribute_column: "资金属性".to_string(),
        }
    }
    
    /// 获取所有必需列名
    pub fn get_required_columns(&self) -> Vec<String> {
        vec![
            self.transaction_date_column.clone(),
            self.transaction_time_column.clone(),
            self.income_amount_column.clone(),
            self.expense_amount_column.clone(),
            self.balance_column.clone(),
            self.fund_attribute_column.clone(),
        ]
    }
}

impl Default for ExcelColumnConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        let config = Config::new();
        
        assert_eq!(config.numeric.decimal_places, 2);
        assert!(config.fund_attributes.personal_fund_keywords.contains("个人"));
        assert!(config.fund_attributes.company_fund_keywords.contains("公司"));
        assert!(config.investment_products.product_prefixes.contains(&"理财-".to_string()));
    }
    
    #[test]
    fn test_fund_classification() {
        let config = Config::new();
        
        assert!(config.is_personal_fund("个人应收"));
        assert!(config.is_company_fund("公司应付"));
        assert!(config.is_investment_product("理财-SL100613100620"));
        
        assert!(!config.is_personal_fund("公司应收"));
        assert!(!config.is_company_fund("个人应付"));
        assert!(!config.is_investment_product("其他资金"));
    }
    
    #[test]
    fn test_number_formatting() {
        let config = Config::new();
        let value = Decimal::from_f64_retain(123.456789).unwrap();
        let formatted = config.format_number(value);
        
        // 使用字符串比较来避免浮点精度问题
        assert_eq!(formatted.to_string(), "123.46");
    }
    
    #[test]
    fn test_balance_tolerance() {
        let config = Config::new();
        let balance1 = Decimal::from_f64_retain(100.00).unwrap();
        let balance2 = Decimal::from_f64_retain(100.005).unwrap();
        let balance3 = Decimal::from_f64_retain(100.02).unwrap();
        
        assert!(config.is_balance_within_tolerance(balance1, balance2));
        assert!(!config.is_balance_within_tolerance(balance1, balance3));
    }
    
    #[test]
    fn test_required_columns() {
        let excel_config = ExcelColumnConfig::new();
        let columns = excel_config.get_required_columns();
        
        assert_eq!(columns.len(), 6);
        assert!(columns.contains(&"交易日期".to_string()));
        assert!(columns.contains(&"交易时间".to_string()));
        assert!(columns.contains(&"资金属性".to_string()));
    }
}