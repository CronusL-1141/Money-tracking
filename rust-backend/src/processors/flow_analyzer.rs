//! 资金流向分析器
//! 
//! 对应Python版本的FlowAnalyzer，专门负责分析资金流向和交易性质

use crate::errors::{AuditError, AuditResult};
use crate::models::{Transaction, Config};
use rust_decimal::Decimal;
use log::{info, debug};

/// 资金流向分析器
#[derive(Debug)]
pub struct FlowAnalyzer {
    config: Config,
}

impl FlowAnalyzer {
    /// 创建新的流向分析器
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 分析交易方向和实际金额
    /// 对应Python版本FlowAnalyzer.分析交易方向方法
    pub fn analyze_transaction_direction(&self, income_amount: Decimal, expense_amount: Decimal) -> (Decimal, TransactionDirection) {
        // 确定实际交易金额和方向
        if income_amount > Decimal::ZERO && expense_amount == Decimal::ZERO {
            // 只有收入有值，是收入
            (income_amount, TransactionDirection::Income)
        } else if income_amount == Decimal::ZERO && expense_amount > Decimal::ZERO {
            // 只有支出有值，是支出
            (expense_amount, TransactionDirection::Expense)
        } else if income_amount > Decimal::ZERO && expense_amount > Decimal::ZERO {
            // 两个都有值，取较大的作为实际金额
            if income_amount > expense_amount {
                (income_amount, TransactionDirection::Income)
            } else {
                (expense_amount, TransactionDirection::Expense)
            }
        } else {
            // 两个都为零或都为负
            (Decimal::ZERO, TransactionDirection::None)
        }
    }
    
    /// 分析资金流向类型
    /// 对应Python版本FlowAnalyzer.分析资金流向类型方法
    pub fn analyze_fund_flow_type(&self, direction: &TransactionDirection, fund_attribute: &str) -> String {
        match direction {
            TransactionDirection::Income => {
                // 检查是否为投资产品赎回
                if self.config.is_investment_product(fund_attribute) {
                    let prefix = fund_attribute.split('-').next().unwrap_or(fund_attribute);
                    format!("{}赎回", prefix)
                } else {
                    "资金流入".to_string()
                }
            }
            TransactionDirection::Expense => {
                // 检查是否为投资产品申购
                if self.config.is_investment_product(fund_attribute) {
                    let prefix = fund_attribute.split('-').next().unwrap_or(fund_attribute);
                    format!("{}申购", prefix)
                } else {
                    "资金支出".to_string()
                }
            }
            TransactionDirection::None => "无交易".to_string(),
        }
    }
    
    /// 解析投资产品编号
    /// 对应Python版本FlowAnalyzer.解析投资产品编号方法
    pub fn parse_investment_product_code(&self, fund_attribute: &str) -> Option<String> {
        if self.config.is_investment_product(fund_attribute) {
            Some(fund_attribute.to_string()) // 返回完整的产品编号
        } else {
            None
        }
    }
    
    /// 分析异常交易
    /// 对应Python版本FlowAnalyzer.分析异常交易方法
    pub fn analyze_abnormal_transactions(&self, transactions: &[Transaction]) -> AbnormalTransactionAnalysis {
        let mut analysis = AbnormalTransactionAnalysis::new();
        
        for (index, tx) in transactions.iter().enumerate() {
            // 分析大额交易
            if tx.income_amount > self.config.large_amount_threshold || tx.expense_amount > self.config.large_amount_threshold {
                analysis.large_transactions.push(index + 1);
            }
            
            // 分析同时收支
            if tx.income_amount > Decimal::ZERO && tx.expense_amount > Decimal::ZERO {
                analysis.simultaneous_income_expense.push(index + 1);
            }
            
            // 分析异常金额（负数）
            if tx.income_amount < Decimal::ZERO || tx.expense_amount < Decimal::ZERO {
                analysis.abnormal_amounts.push(index + 1);
            }
            
            // 分析零交易
            if tx.income_amount == Decimal::ZERO && tx.expense_amount == Decimal::ZERO {
                analysis.zero_transactions.push(index + 1);
            }
        }
        
        info!("🔍 异常交易分析完成: 大额交易{}笔, 同时收支{}笔, 异常金额{}笔, 零交易{}笔",
            analysis.large_transactions.len(),
            analysis.simultaneous_income_expense.len(), 
            analysis.abnormal_amounts.len(),
            analysis.zero_transactions.len()
        );
        
        analysis
    }
    
    /// 生成资金流向统计
    /// 对应Python版本FlowAnalyzer.生成流向统计方法
    pub fn generate_flow_statistics(&self, transactions: &[Transaction]) -> FlowStatistics {
        let mut stats = FlowStatistics::new();
        
        let mut total_income = Decimal::ZERO;
        let mut total_expense = Decimal::ZERO;
        let mut flow_type_counts = std::collections::HashMap::new();
        
        for tx in transactions {
            // 统计总收入和总支出
            if tx.income_amount > Decimal::ZERO {
                total_income += tx.income_amount;
            }
            if tx.expense_amount > Decimal::ZERO {
                total_expense += tx.expense_amount;
            }
            
            // 分析并统计流向类型
            let (_, direction) = self.analyze_transaction_direction(tx.income_amount, tx.expense_amount);
            let flow_type = self.analyze_fund_flow_type(&direction, &tx.fund_attribute);
            
            *flow_type_counts.entry(flow_type).or_insert(0) += 1;
        }
        
        stats.total_income = total_income;
        stats.total_expense = total_expense;
        stats.net_inflow = total_income - total_expense;
        stats.transaction_count = transactions.len();
        stats.flow_type_distribution = flow_type_counts;
        
        debug!("📊 流向统计: 总收入{:.2}, 总支出{:.2}, 净流入{:.2}, 交易笔数{}", 
            stats.total_income, stats.total_expense, stats.net_inflow, stats.transaction_count);
        
        stats
    }
    
    /// 分析资金流向趋势
    /// 对应Python版本FlowAnalyzer.分析资金流向趋势方法
    pub fn analyze_flow_trends(&self, transactions: &[Transaction]) -> FlowTrends {
        let mut trends = FlowTrends::new();
        let mut daily_stats = std::collections::HashMap::new();
        
        for tx in transactions {
            let date = tx.transaction_date.date();
            let entry = daily_stats.entry(date).or_insert(DailyFlowStats::new());
            
            if tx.income_amount > Decimal::ZERO {
                entry.daily_income += tx.income_amount;
            }
            if tx.expense_amount > Decimal::ZERO {
                entry.daily_expense += tx.expense_amount;
            }
            entry.transaction_count += 1;
        }
        
        // 转换为向量并排序
        let mut daily_records: Vec<_> = daily_stats.into_iter()
            .map(|(date, stats)| DailyFlowRecord {
                date,
                income: stats.daily_income,
                expense: stats.daily_expense,
                net_flow: stats.daily_income - stats.daily_expense,
                transaction_count: stats.transaction_count,
            })
            .collect();
        
        daily_records.sort_by_key(|r| r.date);
        
        // 计算最值
        if !daily_records.is_empty() {
            trends.max_daily_income = daily_records.iter().map(|r| r.income).max().unwrap_or(Decimal::ZERO);
            trends.max_daily_expense = daily_records.iter().map(|r| r.expense).max().unwrap_or(Decimal::ZERO);
            trends.max_daily_net_inflow = daily_records.iter().map(|r| r.net_flow).max().unwrap_or(Decimal::ZERO);
            trends.max_daily_net_outflow = daily_records.iter().map(|r| r.net_flow).min().unwrap_or(Decimal::ZERO);
        }
        
        trends.daily_records = daily_records;
        
        debug!("📈 流向趋势分析完成: {}天记录, 最大日收入{:.2}, 最大日支出{:.2}", 
            trends.daily_records.len(), trends.max_daily_income, trends.max_daily_expense);
        
        trends
    }
    
    /// 处理单笔交易的流向分析
    pub fn process_transaction_flow(&self, transaction: &mut Transaction) -> TransactionFlowResult {
        let (actual_amount, direction) = self.analyze_transaction_direction(
            transaction.income_amount,
            transaction.expense_amount
        );
        
        let flow_type = self.analyze_fund_flow_type(&direction, &transaction.fund_attribute);
        let is_investment = self.config.is_investment_product(&transaction.fund_attribute);
        
        TransactionFlowResult {
            actual_amount,
            direction,
            flow_type,
            is_investment,
            investment_product_code: if is_investment {
                Some(transaction.fund_attribute.clone())
            } else {
                None
            },
        }
    }
}

/// 交易方向枚举
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionDirection {
    Income,    // 收入
    Expense,   // 支出
    None,      // 无交易
}

/// 单笔交易流向分析结果
#[derive(Debug, Clone)]
pub struct TransactionFlowResult {
    pub actual_amount: Decimal,
    pub direction: TransactionDirection,
    pub flow_type: String,
    pub is_investment: bool,
    pub investment_product_code: Option<String>,
}

/// 异常交易分析结果
#[derive(Debug, Clone)]
pub struct AbnormalTransactionAnalysis {
    pub large_transactions: Vec<usize>,          // 大额交易行号
    pub simultaneous_income_expense: Vec<usize>, // 同时收支行号
    pub abnormal_amounts: Vec<usize>,            // 异常金额行号
    pub zero_transactions: Vec<usize>,           // 零交易行号
}

impl AbnormalTransactionAnalysis {
    fn new() -> Self {
        Self {
            large_transactions: Vec::new(),
            simultaneous_income_expense: Vec::new(),
            abnormal_amounts: Vec::new(),
            zero_transactions: Vec::new(),
        }
    }
}

/// 资金流向统计
#[derive(Debug, Clone)]
pub struct FlowStatistics {
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub net_inflow: Decimal,
    pub transaction_count: usize,
    pub flow_type_distribution: std::collections::HashMap<String, usize>,
}

impl FlowStatistics {
    fn new() -> Self {
        Self {
            total_income: Decimal::ZERO,
            total_expense: Decimal::ZERO,
            net_inflow: Decimal::ZERO,
            transaction_count: 0,
            flow_type_distribution: std::collections::HashMap::new(),
        }
    }
}

/// 资金流向趋势分析
#[derive(Debug, Clone)]
pub struct FlowTrends {
    pub daily_records: Vec<DailyFlowRecord>,
    pub max_daily_income: Decimal,
    pub max_daily_expense: Decimal,
    pub max_daily_net_inflow: Decimal,
    pub max_daily_net_outflow: Decimal,
}

impl FlowTrends {
    fn new() -> Self {
        Self {
            daily_records: Vec::new(),
            max_daily_income: Decimal::ZERO,
            max_daily_expense: Decimal::ZERO,
            max_daily_net_inflow: Decimal::ZERO,
            max_daily_net_outflow: Decimal::ZERO,
        }
    }
}

/// 日流向记录
#[derive(Debug, Clone)]
pub struct DailyFlowRecord {
    pub date: chrono::NaiveDate,
    pub income: Decimal,
    pub expense: Decimal,
    pub net_flow: Decimal,
    pub transaction_count: usize,
}

/// 日流向统计（内部使用）
#[derive(Debug, Clone)]
struct DailyFlowStats {
    daily_income: Decimal,
    daily_expense: Decimal,
    transaction_count: usize,
}

impl DailyFlowStats {
    fn new() -> Self {
        Self {
            daily_income: Decimal::ZERO,
            daily_expense: Decimal::ZERO,
            transaction_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_analyze_transaction_direction() {
        let config = Config::new();
        let analyzer = FlowAnalyzer::new(config);
        
        // 测试收入
        let (amount, direction) = analyzer.analyze_transaction_direction(Decimal::from(50000), Decimal::ZERO);
        assert_eq!(amount, Decimal::from(50000));
        assert_eq!(direction, TransactionDirection::Income);
        
        // 测试支出
        let (amount, direction) = analyzer.analyze_transaction_direction(Decimal::ZERO, Decimal::from(30000));
        assert_eq!(amount, Decimal::from(30000));
        assert_eq!(direction, TransactionDirection::Expense);
        
        // 测试无交易
        let (amount, direction) = analyzer.analyze_transaction_direction(Decimal::ZERO, Decimal::ZERO);
        assert_eq!(amount, Decimal::ZERO);
        assert_eq!(direction, TransactionDirection::None);
    }
    
    #[test]
    fn test_analyze_fund_flow_type() {
        let config = Config::new();
        let analyzer = FlowAnalyzer::new(config);
        
        // 测试普通收入
        let flow_type = analyzer.analyze_fund_flow_type(&TransactionDirection::Income, "个人应收");
        assert_eq!(flow_type, "资金流入");
        
        // 测试投资产品赎回
        let flow_type = analyzer.analyze_fund_flow_type(&TransactionDirection::Income, "理财-SL001");
        assert_eq!(flow_type, "理财赎回");
        
        // 测试投资产品申购
        let flow_type = analyzer.analyze_fund_flow_type(&TransactionDirection::Expense, "理财-SL001");
        assert_eq!(flow_type, "理财申购");
        
        // 测试无交易
        let flow_type = analyzer.analyze_fund_flow_type(&TransactionDirection::None, "任意属性");
        assert_eq!(flow_type, "无交易");
    }
    
    #[test]
    fn test_parse_investment_product_code() {
        let config = Config::new();
        let analyzer = FlowAnalyzer::new(config);
        
        // 测试投资产品
        let result = analyzer.parse_investment_product_code("理财-SL001");
        assert_eq!(result, Some("理财-SL001".to_string()));
        
        // 测试非投资产品
        let result = analyzer.parse_investment_product_code("个人应收");
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_process_transaction_flow() {
        let config = Config::new();
        let analyzer = FlowAnalyzer::new(config);
        
        let mut transaction = Transaction::new(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(14, 30, 25).unwrap(),
            "143025".to_string(),
            Decimal::from(50000),
            Decimal::ZERO,
            Decimal::from(120000),
            "理财-SL001".to_string(),
        );
        
        let result = analyzer.process_transaction_flow(&mut transaction);
        
        assert_eq!(result.actual_amount, Decimal::from(50000));
        assert_eq!(result.direction, TransactionDirection::Income);
        assert_eq!(result.flow_type, "理财赎回");
        assert!(result.is_investment);
        assert_eq!(result.investment_product_code, Some("理财-SL001".to_string()));
    }
}