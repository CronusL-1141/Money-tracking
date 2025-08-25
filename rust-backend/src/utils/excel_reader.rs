//! Excel读取专用模块
//! 
//! 专门负责从Excel文件读取数据，不包含业务逻辑

use crate::errors::{AuditError, AuditResult};
use crate::models::Config;
use calamine::{Reader, Xlsx, open_workbook, DataType};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use std::path::Path;
use log::{info, warn, error};

/// Excel读取器
#[derive(Debug)]
pub struct ExcelReader {
    config: Config,
}

impl ExcelReader {
    /// 创建新的Excel读取器
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 从Excel文件读取原始数据
    pub fn read_raw_data<P: AsRef<Path>>(&self, file_path: P) -> AuditResult<Vec<RawTransactionRow>> {
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
        self.parse_raw_data_from_range(range)
    }
    
    /// 从Excel范围解析原始数据
    fn parse_raw_data_from_range(
        &self,
        range: calamine::Range<calamine::Data>
    ) -> AuditResult<Vec<RawTransactionRow>> {
        let mut raw_rows = Vec::new();
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
            match self.parse_raw_row(row, &column_indices, row_idx + 2) {
                Ok(raw_row) => {
                    raw_rows.push(raw_row);
                }
                Err(e) => {
                    warn!("解析第{}行数据失败: {}", row_idx + 2, e);
                    // 继续处理其他行，不中断整个流程
                }
            }
            
            // 定期报告进度
            if (row_idx + 1) % 1000 == 0 {
                info!("⏳ 读取进度: {}/{} ({:.1}%)", 
                    row_idx + 1, 
                    data_rows.len(),
                    (row_idx + 1) as f64 / data_rows.len() as f64 * 100.0
                );
            }
        }
        
        info!("✅ Excel原始数据读取完成，共解析 {} 行", raw_rows.len());
        Ok(raw_rows)
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
    
    /// 解析单行原始数据
    fn parse_raw_row(
        &self,
        row: &[calamine::Data],
        indices: &ColumnIndices,
        row_number: usize
    ) -> AuditResult<RawTransactionRow> {
        // 提取原始字段值
        let date_cell = row.get(indices.transaction_date.unwrap()).unwrap_or(&calamine::Data::Empty);
        let time_cell = row.get(indices.transaction_time.unwrap()).unwrap_or(&calamine::Data::Empty);
        let income_cell = row.get(indices.income_amount.unwrap()).unwrap_or(&calamine::Data::Empty);
        let expense_cell = row.get(indices.expense_amount.unwrap()).unwrap_or(&calamine::Data::Empty);
        let balance_cell = row.get(indices.balance.unwrap()).unwrap_or(&calamine::Data::Empty);
        let fund_attr_cell = row.get(indices.fund_attribute.unwrap()).unwrap_or(&calamine::Data::Empty);
        
        Ok(RawTransactionRow {
            row_number,
            date_cell: date_cell.clone(),
            time_cell: time_cell.clone(),
            income_cell: income_cell.clone(),
            expense_cell: expense_cell.clone(),
            balance_cell: balance_cell.clone(),
            fund_attribute_cell: fund_attr_cell.clone(),
        })
    }
    
    /// 解析十进制数（公共工具方法）
    pub fn parse_decimal(cell: &calamine::Data) -> AuditResult<Decimal> {
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
    
    /// 解析字符串（公共工具方法）
    pub fn parse_string(cell: &calamine::Data) -> String {
        match cell {
            calamine::Data::String(s) => s.clone(),
            calamine::Data::Int(i) => i.to_string(),
            calamine::Data::Float(f) => f.to_string(),
            _ => String::new()
        }
    }
}

/// 原始交易行数据（未经业务处理）
#[derive(Debug, Clone)]
pub struct RawTransactionRow {
    pub row_number: usize,
    pub date_cell: calamine::Data,
    pub time_cell: calamine::Data,
    pub income_cell: calamine::Data,
    pub expense_cell: calamine::Data,
    pub balance_cell: calamine::Data,
    pub fund_attribute_cell: calamine::Data,
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
    
    #[test]
    fn test_parse_decimal() {
        // 测试浮点数
        let float_data = calamine::Data::Float(12345.67);
        assert_eq!(ExcelReader::parse_decimal(&float_data).unwrap(), Decimal::from_f64_retain(12345.67).unwrap());
        
        // 测试整数
        let int_data = calamine::Data::Int(12345);
        assert_eq!(ExcelReader::parse_decimal(&int_data).unwrap(), Decimal::from(12345));
        
        // 测试空值
        let empty_data = calamine::Data::Empty;
        assert_eq!(ExcelReader::parse_decimal(&empty_data).unwrap(), Decimal::ZERO);
        
        // 测试字符串
        let string_data = calamine::Data::String("12,345.67".to_string());
        assert_eq!(ExcelReader::parse_decimal(&string_data).unwrap(), Decimal::from_f64_retain(12345.67).unwrap());
    }
    
    #[test]
    fn test_parse_string() {
        assert_eq!(ExcelReader::parse_string(&calamine::Data::String("test".to_string())), "test");
        assert_eq!(ExcelReader::parse_string(&calamine::Data::Int(123)), "123");
        assert_eq!(ExcelReader::parse_string(&calamine::Data::Float(123.45)), "123.45");
        assert_eq!(ExcelReader::parse_string(&calamine::Data::Empty), "");
    }
}