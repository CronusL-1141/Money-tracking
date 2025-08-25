//! 简化的Excel处理器
//! 
//! 提供基础的Excel读写功能，避免复杂依赖

use crate::errors::{AuditError, AuditResult};
use crate::models::{Transaction, AuditSummary, Config};
use calamine::{Reader, Xlsx, open_workbook, DataType};
use chrono::{NaiveDateTime, NaiveDate};
use rust_decimal::Decimal;
use std::path::Path;
use log::{info, warn, error};
use std::fs::File;
use std::io::Write;

/// 简化Excel处理器
#[derive(Debug)]
pub struct SimpleExcelProcessor {
    config: Config,
}

impl SimpleExcelProcessor {
    /// 创建新的Excel处理器
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 从Excel文件读取交易记录
    pub fn read_transactions<P: AsRef<Path>>(&self, file_path: P) -> AuditResult<Vec<Transaction>> {
        let path = file_path.as_ref();
        info!("开始读取Excel文件: {}", path.display());
        
        // 打开Excel工作簿
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| AuditError::excel_error(format!("无法打开Excel文件: {}", e)))?;
        
        // 获取第一个工作表
        let sheet_names = workbook.sheet_names();
        if sheet_names.is_empty() {
            return Err(AuditError::excel_error("Excel文件中没有工作表"));
        }
        
        let sheet_name = &sheet_names[0];
        info!("读取工作表: {}", sheet_name);
        
        let range = workbook.worksheet_range(sheet_name)
            .map_err(|e| AuditError::excel_error(format!("无法读取工作表: {}", e)))?;
        
        // 解析数据
        self.parse_transactions_from_range(range)
    }
    
    /// 从Excel范围解析交易记录
    fn parse_transactions_from_range(
        &self,
        range: calamine::Range<calamine::Data>
    ) -> AuditResult<Vec<Transaction>> {
        let mut transactions = Vec::new();
        let rows: Vec<_> = range.rows().collect();
        
        if rows.is_empty() {
            return Err(AuditError::excel_error("Excel工作表为空"));
        }
        
        // 查找表头
        let header_row = rows.get(0).ok_or_else(|| {
            AuditError::excel_error("无法获取表头行")
        })?;
        
        let column_indices = self.find_column_indices(header_row)?;
        info!("找到列索引: {:?}", column_indices);
        
        // 解析数据行
        let data_rows = &rows[1..]; // 跳过表头
        info!("开始解析 {} 行数据", data_rows.len());
        
        for (row_idx, row) in data_rows.iter().enumerate() {
            match self.parse_transaction_row(row, &column_indices) {
                Ok(transaction) => {
                    transactions.push(transaction);
                }
                Err(e) => {
                    warn!("解析第{}行数据失败: {}", row_idx + 2, e);
                    // 继续处理其他行，不中断整个流程
                }
            }
            
            // 定期报告进度
            if (row_idx + 1) % 1000 == 0 {
                info!("⏳ 处理进度: {}/{} ({:.1}%)", 
                    row_idx + 1, 
                    data_rows.len(),
                    (row_idx + 1) as f64 / data_rows.len() as f64 * 100.0
                );
            }
        }
        
        info!("✅ Excel数据读取完成，共解析 {} 条交易记录", transactions.len());
        Ok(transactions)
    }
    
    /// 查找列索引
    fn find_column_indices(
        &self,
        header_row: &[calamine::Data]
    ) -> AuditResult<ColumnIndices> {
        let mut indices = ColumnIndices::new();
        
        for (idx, cell) in header_row.iter().enumerate() {
            if let Some(column_name) = cell.get_string() {
                match column_name {
                    name if name == self.config.excel_columns.transaction_date_column => {
                        indices.transaction_date = Some(idx);
                    }
                    name if name == self.config.excel_columns.transaction_time_column => {
                        indices.transaction_time = Some(idx);
                    }
                    name if name == self.config.excel_columns.income_amount_column => {
                        indices.income_amount = Some(idx);
                    }
                    name if name == self.config.excel_columns.expense_amount_column => {
                        indices.expense_amount = Some(idx);
                    }
                    name if name == self.config.excel_columns.balance_column => {
                        indices.balance = Some(idx);
                    }
                    name if name == self.config.excel_columns.fund_attribute_column => {
                        indices.fund_attribute = Some(idx);
                    }
                    _ => {} // 忽略其他列
                }
            }
        }
        
        // 验证必需列是否都找到了
        indices.validate()?;
        Ok(indices)
    }
    
    /// 解析单行交易数据
    fn parse_transaction_row(
        &self,
        row: &[calamine::Data],
        indices: &ColumnIndices
    ) -> AuditResult<Transaction> {
        // 解析交易日期
        let transaction_date = self.parse_date(
            row.get(indices.transaction_date.unwrap()).unwrap_or(&calamine::Data::Empty)
        )?;
        
        // 解析交易时间
        let transaction_time = self.parse_time_string(
            row.get(indices.transaction_time.unwrap()).unwrap_or(&calamine::Data::Empty)
        )?;
        
        // 解析金额字段
        let income_amount = self.parse_decimal(
            row.get(indices.income_amount.unwrap()).unwrap_or(&calamine::Data::Empty)
        ).unwrap_or(Decimal::ZERO);
        
        let expense_amount = self.parse_decimal(
            row.get(indices.expense_amount.unwrap()).unwrap_or(&calamine::Data::Empty)
        ).unwrap_or(Decimal::ZERO);
        
        let balance = self.parse_decimal(
            row.get(indices.balance.unwrap()).unwrap_or(&calamine::Data::Empty)
        )?;
        
        // 解析资金属性
        let fund_attribute = row.get(indices.fund_attribute.unwrap())
            .and_then(|cell| cell.get_string())
            .unwrap_or("")
            .to_string();
        
        if fund_attribute.is_empty() {
            return Err(AuditError::validation_error("资金属性不能为空"));
        }
        
        Ok(Transaction::new(
            transaction_date,
            transaction_time,
            income_amount,
            expense_amount,
            balance,
            fund_attribute,
        ))
    }
    
    /// 解析日期
    fn parse_date(&self, cell: &calamine::Data) -> AuditResult<NaiveDateTime> {
        match cell {
            calamine::Data::DateTime(dt) => {
                // Excel日期时间格式转换
                // Excel的日期是从1900年1月1日开始的天数
                let excel_epoch = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
                let days = dt.as_f64() as i64;
                let nanos = ((dt.as_f64() - days as f64) * 86_400_000_000_000f64) as i64;
                
                // Excel有个bug：1900年2月29日不存在，但Excel认为存在，所以要减2天
                let actual_days = if days > 59 { days - 2 } else { days - 1 };
                
                let date = excel_epoch + chrono::Duration::days(actual_days);
                let time = chrono::Duration::nanoseconds(nanos);
                Ok(date.and_hms_opt(0, 0, 0).unwrap() + time)
            },
            calamine::Data::String(date_str) => {
                // 尝试多种日期格式
                let formats = [
                    "%Y-%m-%d",
                    "%Y/%m/%d",
                    "%Y年%m月%d日",
                    "%m/%d/%Y",
                    "%d/%m/%Y",
                ];
                
                for format in &formats {
                    if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                        return Ok(date.and_hms_opt(0, 0, 0).unwrap());
                    }
                }
                
                Err(AuditError::time_parse_error(format!("无法解析日期: {}", date_str)))
            }
            calamine::Data::Float(days) => {
                // Excel日期数字格式
                let base_date = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
                let target_date = base_date + chrono::Duration::days(*days as i64 - 2);
                Ok(target_date.and_hms_opt(0, 0, 0).unwrap())
            }
            _ => Err(AuditError::time_parse_error("不支持的日期格式"))
        }
    }
    
    /// 解析时间字符串
    fn parse_time_string(&self, cell: &calamine::Data) -> AuditResult<String> {
        match cell {
            calamine::Data::String(time_str) => Ok(time_str.clone()),
            calamine::Data::Int(time_int) => Ok(time_int.to_string()),
            calamine::Data::Float(time_float) => Ok(format!("{:.0}", time_float)),
            _ => Ok("".to_string()) // 时间字段可以为空
        }
    }
    
    /// 解析十进制数
    fn parse_decimal(&self, cell: &calamine::Data) -> AuditResult<Decimal> {
        match cell {
            calamine::Data::Float(f) => {
                Decimal::from_f64_retain(*f)
                    .ok_or_else(|| AuditError::calculation_error(format!("无法转换数值: {}", f)))
            }
            calamine::Data::Int(i) => {
                Ok(Decimal::from(*i))
            }
            calamine::Data::String(s) => {
                // 移除千位分隔符和其他非数字字符（除了小数点和负号）
                let cleaned = s.chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect::<String>();
                
                if cleaned.is_empty() {
                    return Ok(Decimal::ZERO);
                }
                
                cleaned.parse::<Decimal>()
                    .map_err(|_| AuditError::calculation_error(format!("无法解析数值: {}", s)))
            }
            calamine::Data::Empty => Ok(Decimal::ZERO),
            _ => Err(AuditError::calculation_error("不支持的数值格式"))
        }
    }
    
    /// 导出结果为CSV格式（简化版本）
    pub fn export_analysis_results_csv<P: AsRef<Path>>(
        &self,
        transactions: &[Transaction],
        summary: &AuditSummary,
        output_path: P,
    ) -> AuditResult<()> {
        let path = output_path.as_ref();
        info!("开始导出分析结果到CSV: {}", path.display());
        
        let mut file = File::create(path)
            .map_err(|e| AuditError::excel_error(format!("创建文件失败: {}", e)))?;
        
        // 写入UTF-8 BOM（Excel兼容）
        file.write_all(&[0xEF, 0xBB, 0xBF])
            .map_err(|e| AuditError::excel_error(format!("写入BOM失败: {}", e)))?;
        
        // 写入表头
        writeln!(file, "交易日期,交易时间,交易收入金额,交易支出金额,余额,资金属性,个人资金占比,公司资金占比,行为性质,累计挪用,累计垫付,个人余额,公司余额")
            .map_err(|e| AuditError::excel_error(format!("写入表头失败: {}", e)))?;
        
        // 写入数据
        for tx in transactions {
            let personal_ratio = tx.personal_ratio.unwrap_or(Decimal::ZERO);
            let company_ratio = tx.company_ratio.unwrap_or(Decimal::ZERO);
            let behavior = tx.behavior_nature.as_deref().unwrap_or("");
            let cum_misap = tx.cumulative_misappropriation.unwrap_or(Decimal::ZERO);
            let cum_advance = tx.cumulative_advance.unwrap_or(Decimal::ZERO);
            let personal_balance = tx.personal_balance.unwrap_or(Decimal::ZERO);
            let company_balance = tx.company_balance.unwrap_or(Decimal::ZERO);
            
            writeln!(file, "{},{},{},{},{},{},{},{},\"{}\",{},{},{},{}",
                tx.transaction_date.format("%Y-%m-%d"),
                tx.transaction_time,
                tx.income_amount,
                tx.expense_amount,
                tx.balance,
                tx.fund_attribute,
                personal_ratio,
                company_ratio,
                behavior,
                cum_misap,
                cum_advance,
                personal_balance,
                company_balance
            ).map_err(|e| AuditError::excel_error(format!("写入数据失败: {}", e)))?;
        }
        
        info!("✅ CSV结果导出完成");
        Ok(())
    }
    
    /// 导出摘要为文本格式
    pub fn export_summary_text<P: AsRef<Path>>(
        &self,
        summary: &AuditSummary,
        output_path: P,
    ) -> AuditResult<()> {
        let path = output_path.as_ref();
        info!("开始导出摘要到文本文件: {}", path.display());
        
        let mut file = File::create(path)
            .map_err(|e| AuditError::excel_error(format!("创建摘要文件失败: {}", e)))?;
        
        writeln!(file, "=== FIFO资金追踪审计摘要 ===")?;
        writeln!(file, "个人余额: {:.2}", summary.personal_balance)?;
        writeln!(file, "公司余额: {:.2}", summary.company_balance)?;
        writeln!(file, "总余额: {:.2}", summary.total_balance)?;
        writeln!(file, "累计挪用金额: {:.2}", summary.total_misappropriation)?;
        writeln!(file, "累计垫付金额: {:.2}", summary.total_advance_payment)?;
        writeln!(file, "累计归还公司本金: {:.2}", summary.total_company_principal_returned)?;
        writeln!(file, "累计归还个人本金: {:.2}", summary.total_personal_principal_returned)?;
        writeln!(file, "总计个人应分配利润: {:.2}", summary.total_personal_profit)?;
        writeln!(file, "总计公司应分配利润: {:.2}", summary.total_company_profit)?;
        writeln!(file, "资金缺口: {:.2}", summary.funding_gap)?;
        writeln!(file, "投资产品数量: {}", summary.investment_product_count)?;
        writeln!(file, "================================")?;
        
        info!("✅ 摘要文本导出完成");
        Ok(())
    }
}

/// 列索引结构
#[derive(Debug, Clone)]
struct ColumnIndices {
    transaction_date: Option<usize>,
    transaction_time: Option<usize>,
    income_amount: Option<usize>,
    expense_amount: Option<usize>,
    balance: Option<usize>,
    fund_attribute: Option<usize>,
}

impl ColumnIndices {
    fn new() -> Self {
        Self {
            transaction_date: None,
            transaction_time: None,
            income_amount: None,
            expense_amount: None,
            balance: None,
            fund_attribute: None,
        }
    }
    
    fn validate(&self) -> AuditResult<()> {
        let required_columns = [
            ("交易日期", self.transaction_date),
            ("交易时间", self.transaction_time),
            ("交易收入金额", self.income_amount),
            ("交易支出金额", self.expense_amount),
            ("余额", self.balance),
            ("资金属性", self.fund_attribute),
        ];
        
        for (name, index) in &required_columns {
            if index.is_none() {
                return Err(AuditError::validation_error(
                    format!("找不到必需的列: {}", name)
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_column_indices_validation() {
        let mut indices = ColumnIndices::new();
        assert!(indices.validate().is_err());
        
        indices.transaction_date = Some(0);
        indices.transaction_time = Some(1);
        indices.income_amount = Some(2);
        indices.expense_amount = Some(3);
        indices.balance = Some(4);
        indices.fund_attribute = Some(5);
        
        assert!(indices.validate().is_ok());
    }
}