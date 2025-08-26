use audit_backend::{
    AuditError, Transaction, Config, AuditSummary,
    ExcelProcessor, UnifiedValidator
};
use calamine::{Reader, open_workbook, Xlsx};
use rust_xlsxwriter::{Workbook, Worksheet, Format};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use std::path::Path;
use std::collections::HashMap;
use std::str::FromStr;
use colored::*;

/// 算法验证结果比较器
pub struct AlgorithmValidator {
    pub algorithm_name: String,
    pub test_data_path: String,
    pub expected_result_path: String,
    pub actual_result_path: String,
}

impl AlgorithmValidator {
    pub fn new(algorithm_name: &str) -> Self {
        let base_path = "C:\\Users\\TUF\\Desktop\\资金追踪";
        let algorithm_upper = algorithm_name.to_uppercase();
        
        Self {
            algorithm_name: algorithm_name.to_string(),
            test_data_path: format!("{}\\流水.xlsx", base_path),
            expected_result_path: format!("{}\\{}_资金追踪结果.xlsx", base_path, algorithm_upper),
            actual_result_path: format!("{}\\independent_tests\\algorithm_validation_tests\\output\\{}_rust_output.xlsx", 
                                       base_path, algorithm_upper),
        }
    }

    /// 加载测试数据
    pub fn load_test_data(&self) -> Result<Vec<Transaction>, AuditError> {
        println!("{}", format!("📂 加载测试数据: {}", self.test_data_path).blue());
        
        let excel_processor = ExcelProcessor::new();
        let mut transactions = excel_processor.read_transactions(&self.test_data_path)?;
        
        // 数据验证和修复
        let validator = UnifiedValidator::new();
        validator.validate_and_repair_flow(&mut transactions)?;
        
        println!("{}", format!("✅ 成功加载 {} 条交易记录", transactions.len()).green());
        Ok(transactions)
    }

    /// 加载预期结果Excel文件
    pub fn load_expected_results(&self) -> Result<ExpectedResults, AuditError> {
        println!("{}", format!("📊 加载预期结果: {}", self.expected_result_path).blue());
        
        let mut workbook: Xlsx<_> = open_workbook(&self.expected_result_path)
            .map_err(|e| AuditError::validation_error(format!("无法打开预期结果文件: {}", e)))?;
        
        // 读取详细流水工作表
        let range = workbook.worksheet_range("详细流水")
            .map_err(|e| AuditError::validation_error(format!("无法读取详细流水工作表: {}", e)))?;
        
        let mut enhanced_transactions = Vec::new();
        let headers: Vec<String> = range.rows().next().unwrap_or_default()
            .iter()
            .map(|cell| cell.to_string())
            .collect();
        
        // 解析每一行数据
        for (row_idx, row) in range.rows().enumerate().skip(1) {
            if let Some(enhanced_transaction) = self.parse_enhanced_transaction_row(&headers, row)? {
                enhanced_transactions.push(enhanced_transaction);
            }
        }
        
        // 读取审计摘要工作表
        let summary = self.parse_audit_summary(&mut workbook)?;
        
        println!("{}", format!("✅ 成功解析预期结果: {} 条增强记录", enhanced_transactions.len()).green());
        
        Ok(ExpectedResults {
            enhanced_transactions,
            audit_summary: summary,
        })
    }

    /// 解析增强交易记录行
    fn parse_enhanced_transaction_row(&self, headers: &[String], row: &[calamine::DataType]) -> Result<Option<EnhancedTransaction>, AuditError> {
        if row.is_empty() {
            return Ok(None);
        }

        let mut transaction = EnhancedTransaction {
            // 基础交易信息
            transaction_time: Default::default(),
            summary: String::new(),
            income: Decimal::ZERO,
            expense: Decimal::ZERO,
            balance: Decimal::ZERO,
            counterparty: String::new(),
            // 增强字段
            personal_balance: Decimal::ZERO,
            company_balance: Decimal::ZERO,
            misappropriation: Decimal::ZERO,
            advance_payment: Decimal::ZERO,
            fund_nature: String::new(),
            behavior_nature: String::new(),
            investment_details: String::new(),
        };

        // 根据标题映射解析各字段
        for (col_idx, header) in headers.iter().enumerate() {
            if let Some(cell) = row.get(col_idx) {
                match header.as_str() {
                    "交易时间" => {
                        transaction.transaction_time = self.parse_datetime_cell(cell)?;
                    }
                    "摘要" => {
                        transaction.summary = cell.to_string();
                    }
                    "收入" => {
                        transaction.income = self.parse_decimal_cell(cell)?;
                    }
                    "支出" => {
                        transaction.expense = self.parse_decimal_cell(cell)?;
                    }
                    "余额" => {
                        transaction.balance = self.parse_decimal_cell(cell)?;
                    }
                    "对方" => {
                        transaction.counterparty = cell.to_string();
                    }
                    "个人余额" => {
                        transaction.personal_balance = self.parse_decimal_cell(cell)?;
                    }
                    "公司余额" => {
                        transaction.company_balance = self.parse_decimal_cell(cell)?;
                    }
                    "挪用金额" => {
                        transaction.misappropriation = self.parse_decimal_cell(cell)?;
                    }
                    "垫付金额" => {
                        transaction.advance_payment = self.parse_decimal_cell(cell)?;
                    }
                    "资金性质" => {
                        transaction.fund_nature = cell.to_string();
                    }
                    "行为性质" => {
                        transaction.behavior_nature = cell.to_string();
                    }
                    "投资详情" => {
                        transaction.investment_details = cell.to_string();
                    }
                    _ => {} // 忽略其他列
                }
            }
        }

        Ok(Some(transaction))
    }

    /// 解析审计摘要
    fn parse_audit_summary(&self, workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<ExpectedAuditSummary, AuditError> {
        // 尝试读取摘要工作表
        let range = workbook.worksheet_range("摘要")
            .or_else(|_| workbook.worksheet_range("审计摘要"))
            .or_else(|_| workbook.worksheet_range("Summary"))
            .map_err(|e| AuditError::validation_error(format!("无法找到摘要工作表: {}", e)))?;

        let mut summary = ExpectedAuditSummary {
            total_misappropriation: Decimal::ZERO,
            total_advance_payment: Decimal::ZERO,
            final_personal_balance: Decimal::ZERO,
            final_company_balance: Decimal::ZERO,
            investment_summary: HashMap::new(),
        };

        // 解析摘要数据（简化版本，根据实际Excel格式调整）
        for row in range.rows() {
            if let (Some(key), Some(value)) = (row.get(0), row.get(1)) {
                let key_str = key.to_string();
                match key_str.as_str() {
                    "总挪用金额" | "挪用金额合计" => {
                        summary.total_misappropriation = self.parse_decimal_cell(value)?;
                    }
                    "总垫付金额" | "垫付金额合计" => {
                        summary.total_advance_payment = self.parse_decimal_cell(value)?;
                    }
                    "最终个人余额" => {
                        summary.final_personal_balance = self.parse_decimal_cell(value)?;
                    }
                    "最终公司余额" => {
                        summary.final_company_balance = self.parse_decimal_cell(value)?;
                    }
                    _ => {}
                }
            }
        }

        Ok(summary)
    }

    /// 解析日期时间单元格
    fn parse_datetime_cell(&self, cell: &calamine::DataType) -> Result<NaiveDateTime, AuditError> {
        match cell {
            calamine::DataType::DateTime(dt) => {
                Ok(NaiveDateTime::new(*dt, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
            }
            calamine::DataType::String(s) => {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y/%m/%d %H:%M:%S"))
                    .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d"))
                    .map_err(|e| AuditError::validation_error(format!("无法解析日期时间: {} ({})", s, e)))
            }
            _ => Err(AuditError::validation_error("无效的日期时间格式".to_string()))
        }
    }

    /// 解析数值单元格
    fn parse_decimal_cell(&self, cell: &calamine::DataType) -> Result<Decimal, AuditError> {
        match cell {
            calamine::DataType::Float(f) => {
                Decimal::from_f64_retain(*f)
                    .ok_or_else(|| AuditError::validation_error("无法转换浮点数".to_string()))
            }
            calamine::DataType::Int(i) => {
                Ok(Decimal::from(*i))
            }
            calamine::DataType::String(s) => {
                if s.trim().is_empty() {
                    Ok(Decimal::ZERO)
                } else {
                    s.parse::<Decimal>()
                        .map_err(|e| AuditError::validation_error(format!("无法解析数值: {} ({})", s, e)))
                }
            }
            calamine::DataType::Empty => Ok(Decimal::ZERO),
            _ => Err(AuditError::validation_error("无效的数值格式".to_string()))
        }
    }

    /// 比较算法结果
    pub fn compare_results(&self, expected: &ExpectedResults, actual: &AlgorithmResults) -> ValidationReport {
        println!("{}", "🔍 开始结果比较...".yellow());
        
        let mut report = ValidationReport {
            algorithm_name: self.algorithm_name.clone(),
            transaction_comparison: self.compare_transactions(&expected.enhanced_transactions, &actual.enhanced_transactions),
            summary_comparison: self.compare_summaries(&expected.audit_summary, &actual.audit_summary),
            overall_passed: false,
        };

        report.overall_passed = report.transaction_comparison.passed && report.summary_comparison.passed;
        
        if report.overall_passed {
            println!("{}", format!("✅ {} 算法验证通过!", self.algorithm_name).bright_green());
        } else {
            println!("{}", format!("❌ {} 算法验证失败!", self.algorithm_name).bright_red());
        }

        report
    }

    /// 比较交易记录
    fn compare_transactions(&self, expected: &[EnhancedTransaction], actual: &[EnhancedTransaction]) -> TransactionComparison {
        let mut comparison = TransactionComparison {
            total_expected: expected.len(),
            total_actual: actual.len(),
            matched_count: 0,
            mismatched_records: Vec::new(),
            passed: false,
        };

        if expected.len() != actual.len() {
            comparison.mismatched_records.push(format!(
                "记录数量不匹配: 预期 {} 条, 实际 {} 条",
                expected.len(), actual.len()
            ));
        }

        let min_len = expected.len().min(actual.len());
        for i in 0..min_len {
            if self.transactions_match(&expected[i], &actual[i]) {
                comparison.matched_count += 1;
            } else {
                comparison.mismatched_records.push(format!(
                    "第 {} 条记录不匹配: 预期挪用={}, 实际挪用={}",
                    i + 1, expected[i].misappropriation, actual[i].misappropriation
                ));
            }
        }

        comparison.passed = comparison.matched_count == min_len && expected.len() == actual.len();
        comparison
    }

    /// 检查两个交易记录是否匹配
    fn transactions_match(&self, expected: &EnhancedTransaction, actual: &EnhancedTransaction) -> bool {
        // 关键字段比较（允许小的数值差异）
        let balance_match = (expected.balance - actual.balance).abs() < Decimal::from_str("0.01").unwrap();
        let misappropriation_match = (expected.misappropriation - actual.misappropriation).abs() < Decimal::from_str("0.01").unwrap();
        let advance_payment_match = (expected.advance_payment - actual.advance_payment).abs() < Decimal::from_str("0.01").unwrap();

        balance_match && misappropriation_match && advance_payment_match
    }

    /// 比较摘要信息
    fn compare_summaries(&self, expected: &ExpectedAuditSummary, actual: &ExpectedAuditSummary) -> SummaryComparison {
        let tolerance = Decimal::from_str("0.01").unwrap();
        
        let misappropriation_match = (expected.total_misappropriation - actual.total_misappropriation).abs() < tolerance;
        let advance_payment_match = (expected.total_advance_payment - actual.total_advance_payment).abs() < tolerance;
        let personal_balance_match = (expected.final_personal_balance - actual.final_personal_balance).abs() < tolerance;
        let company_balance_match = (expected.final_company_balance - actual.final_company_balance).abs() < tolerance;

        let mut differences = Vec::new();
        if !misappropriation_match {
            differences.push(format!(
                "总挪用金额不匹配: 预期 {}, 实际 {}",
                expected.total_misappropriation, actual.total_misappropriation
            ));
        }
        if !advance_payment_match {
            differences.push(format!(
                "总垫付金额不匹配: 预期 {}, 实际 {}",
                expected.total_advance_payment, actual.total_advance_payment
            ));
        }

        SummaryComparison {
            passed: misappropriation_match && advance_payment_match && personal_balance_match && company_balance_match,
            differences,
        }
    }

    /// 生成验证报告
    pub fn generate_report(&self, report: &ValidationReport) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("\n{} 算法验证报告\n", "=".repeat(50)));
        output.push_str(&format!("算法: {}\n", report.algorithm_name));
        output.push_str(&format!("整体结果: {}\n", 
            if report.overall_passed { "✅ 通过" } else { "❌ 失败" }
        ));
        
        output.push_str("\n交易记录比较:\n");
        output.push_str(&format!("- 预期记录数: {}\n", report.transaction_comparison.total_expected));
        output.push_str(&format!("- 实际记录数: {}\n", report.transaction_comparison.total_actual));
        output.push_str(&format!("- 匹配记录数: {}\n", report.transaction_comparison.matched_count));
        
        if !report.transaction_comparison.mismatched_records.is_empty() {
            output.push_str("\n不匹配记录:\n");
            for mismatch in &report.transaction_comparison.mismatched_records {
                output.push_str(&format!("  - {}\n", mismatch));
            }
        }
        
        output.push_str("\n摘要比较:\n");
        if report.summary_comparison.passed {
            output.push_str("- ✅ 摘要数据全部匹配\n");
        } else {
            for diff in &report.summary_comparison.differences {
                output.push_str(&format!("  - ❌ {}\n", diff));
            }
        }
        
        output
    }
}

/// 预期结果结构
#[derive(Debug)]
pub struct ExpectedResults {
    pub enhanced_transactions: Vec<EnhancedTransaction>,
    pub audit_summary: ExpectedAuditSummary,
}

/// 实际算法结果
#[derive(Debug)]
pub struct AlgorithmResults {
    pub enhanced_transactions: Vec<EnhancedTransaction>,
    pub audit_summary: ExpectedAuditSummary,
}

/// 增强交易记录（包含算法计算结果）
#[derive(Debug, Clone)]
pub struct EnhancedTransaction {
    // 基础交易信息
    pub transaction_time: NaiveDateTime,
    pub summary: String,
    pub income: Decimal,
    pub expense: Decimal,
    pub balance: Decimal,
    pub counterparty: String,
    // 算法增强字段
    pub personal_balance: Decimal,
    pub company_balance: Decimal,
    pub misappropriation: Decimal,
    pub advance_payment: Decimal,
    pub fund_nature: String,
    pub behavior_nature: String,
    pub investment_details: String,
}

/// 预期审计摘要
#[derive(Debug)]
pub struct ExpectedAuditSummary {
    pub total_misappropriation: Decimal,
    pub total_advance_payment: Decimal,
    pub final_personal_balance: Decimal,
    pub final_company_balance: Decimal,
    pub investment_summary: HashMap<String, Decimal>,
}

/// 验证报告
#[derive(Debug)]
pub struct ValidationReport {
    pub algorithm_name: String,
    pub transaction_comparison: TransactionComparison,
    pub summary_comparison: SummaryComparison,
    pub overall_passed: bool,
}

/// 交易记录比较结果
#[derive(Debug)]
pub struct TransactionComparison {
    pub total_expected: usize,
    pub total_actual: usize,
    pub matched_count: usize,
    pub mismatched_records: Vec<String>,
    pub passed: bool,
}

/// 摘要比较结果
#[derive(Debug)]
pub struct SummaryComparison {
    pub passed: bool,
    pub differences: Vec<String>,
}