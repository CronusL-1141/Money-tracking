//! 时间处理工具
//! 
//! 专门负责各种时间格式的解析和处理，对应Python版本的FlowAnalyzer中的时间处理部分

use crate::errors::{AuditError, AuditResult};
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, Datelike};
use log::{warn, debug};

/// 时间处理器
#[derive(Debug)]
pub struct TimeProcessor;

impl TimeProcessor {
    /// 解析Excel日期
    /// 对应Python版本的data_processor.py中的日期处理逻辑
    pub fn parse_excel_date(cell: &calamine::Data) -> AuditResult<NaiveDateTime> {
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
                Self::parse_date_string(date_str)
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
    
    /// 解析日期字符串
    fn parse_date_string(date_str: &str) -> AuditResult<NaiveDateTime> {
        // 尝试多种日期格式
        let formats = [
            "%Y-%m-%d",
            "%Y/%m/%d",
            "%Y年%m月%d日",
            "%m/%d/%Y",
            "%d/%m/%Y",
            "%Y-%m-%d %H:%M:%S",
            "%Y/%m/%d %H:%M:%S",
        ];
        
        for format in &formats {
            // 先尝试带时间的格式
            if let Ok(datetime) = NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(datetime);
            }
            // 再尝试只有日期的格式
            if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                return Ok(date.and_hms_opt(0, 0, 0).unwrap());
            }
        }
        
        Err(AuditError::time_parse_error(format!("无法解析日期: {}", date_str)))
    }
    
    /// 解析交易时间
    /// 对应Python版本FlowAnalyzer.解析交易时间方法
    pub fn parse_transaction_time(time_value: &calamine::Data) -> String {
        match time_value {
            calamine::Data::Empty => "00:00:00".to_string(),
            calamine::Data::String(time_str) => {
                Self::format_time_string(time_str)
            }
            calamine::Data::Int(time_int) => {
                Self::format_time_from_number(*time_int as i64)
            }
            calamine::Data::Float(time_float) => {
                Self::format_time_from_number(*time_float as i64)
            }
            _ => {
                warn!("无法解析的时间格式，使用默认值00:00:00");
                "00:00:00".to_string()
            }
        }
    }
    
    /// 格式化时间字符串
    fn format_time_string(time_str: &str) -> String {
        // 如果已经是HH:MM:SS格式，直接返回
        if time_str.contains(':') {
            return time_str.to_string();
        }
        
        // 尝试解析数字格式
        if let Ok(time_num) = time_str.parse::<i64>() {
            return Self::format_time_from_number(time_num);
        }
        
        "00:00:00".to_string()
    }
    
    /// 从数字格式化时间
    fn format_time_from_number(time_num: i64) -> String {
        if time_num == 0 {
            return "00:00:00".to_string();
        }
        
        let time_str = format!("{:06}", time_num); // 补齐到6位
        if time_str.len() >= 6 {
            let hour = &time_str[..2];
            let minute = &time_str[2..4];
            let second = &time_str[4..6];
            
            // 验证时间有效性
            if let (Ok(h), Ok(m), Ok(s)) = (hour.parse::<u32>(), minute.parse::<u32>(), second.parse::<u32>()) {
                if h < 24 && m < 60 && s < 60 {
                    return format!("{}:{}:{}", hour, minute, second);
                }
            }
        }
        
        "00:00:00".to_string()
    }
    
    /// 创建完整时间戳
    /// 对应Python版本FlowAnalyzer.创建完整时间戳方法
    pub fn create_complete_timestamp(date: NaiveDateTime, time_str: &str) -> NaiveDateTime {
        // 解析时间字符串
        if let Ok(time) = Self::parse_time_from_string(time_str) {
            // 合并日期和时间
            return date.date().and_time(time);
        }
        
        // 如果时间解析失败，返回原日期
        warn!("创建时间戳失败，时间字符串: {}", time_str);
        date
    }
    
    /// 从字符串解析时间
    fn parse_time_from_string(time_str: &str) -> AuditResult<NaiveTime> {
        // 处理HH:MM:SS格式
        let time_formats = [
            "%H:%M:%S",
            "%H:%M",
            "%H",
        ];
        
        for format in &time_formats {
            if let Ok(time) = NaiveTime::parse_from_str(time_str, format) {
                return Ok(time);
            }
        }
        
        // 处理数字格式
        if let Ok(time_num) = time_str.parse::<i64>() {
            let formatted_time = Self::format_time_from_number(time_num);
            if let Ok(time) = NaiveTime::parse_from_str(&formatted_time, "%H:%M:%S") {
                return Ok(time);
            }
        }
        
        Err(AuditError::time_parse_error(format!("无法解析时间: {}", time_str)))
    }
    
    /// 验证时间戳合理性
    pub fn validate_timestamp(timestamp: &NaiveDateTime) -> bool {
        // 检查年份是否在合理范围内（2000-2050）
        let year = timestamp.year();
        if year < 2000 || year > 2050 {
            debug!("时间戳年份超出合理范围: {}", year);
            return false;
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_parse_transaction_time() {
        // 测试字符串格式
        let time_str = calamine::Data::String("143025".to_string());
        assert_eq!(TimeProcessor::parse_transaction_time(&time_str), "14:30:25");
        
        // 测试数字格式
        let time_int = calamine::Data::Int(143025);
        assert_eq!(TimeProcessor::parse_transaction_time(&time_int), "14:30:25");
        
        // 测试空值
        let empty = calamine::Data::Empty;
        assert_eq!(TimeProcessor::parse_transaction_time(&empty), "00:00:00");
    }
    
    #[test]
    fn test_format_time_from_number() {
        assert_eq!(TimeProcessor::format_time_from_number(143025), "14:30:25");
        assert_eq!(TimeProcessor::format_time_from_number(0), "00:00:00");
        assert_eq!(TimeProcessor::format_time_from_number(90000), "09:00:00");
    }
    
    #[test]
    fn test_create_complete_timestamp() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let time_str = "14:30:25";
        let timestamp = TimeProcessor::create_complete_timestamp(date, time_str);
        
        assert_eq!(timestamp.date(), date.date());
        assert_eq!(timestamp.time(), NaiveTime::from_hms_opt(14, 30, 25).unwrap());
    }
    
    #[test]
    fn test_parse_date_string() {
        // 测试各种日期格式
        assert!(TimeProcessor::parse_date_string("2023-01-15").is_ok());
        assert!(TimeProcessor::parse_date_string("2023/01/15").is_ok());
        assert!(TimeProcessor::parse_date_string("2023年01月15日").is_ok());
        assert!(TimeProcessor::parse_date_string("01/15/2023").is_ok());
        
        // 测试无效格式
        assert!(TimeProcessor::parse_date_string("invalid").is_err());
    }
    
    #[test]
    fn test_validate_timestamp() {
        let valid_timestamp = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap().and_hms_opt(14, 30, 25).unwrap();
        assert!(TimeProcessor::validate_timestamp(&valid_timestamp));
        
        let invalid_timestamp = NaiveDate::from_ymd_opt(1999, 1, 15).unwrap().and_hms_opt(14, 30, 25).unwrap();
        assert!(!TimeProcessor::validate_timestamp(&invalid_timestamp));
    }
}