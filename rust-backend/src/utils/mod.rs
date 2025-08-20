//! # 工具模块
//! 
//! 提供系统配置、错误处理、日志记录、数据验证等基础工具功能。
//! 
//! ## 主要组件
//! 
//! - [`Config`]: 系统配置管理
//! - [`AuditError`]: 统一错误类型
//! - [`Logger`]: 日志系统
//! - [`Validator`]: 数据验证器
//! - 资金属性判断函数: `is_personal_fund`, `is_company_fund`, `is_investment_product`

pub mod config;
pub mod error;
pub mod logger;
pub mod validator;
pub mod fund_utils;

// 重新导出主要类型
pub use config::Config;
pub use error::{AuditError, Result};
pub use logger::Logger;
pub use validator::Validator;
pub use fund_utils::{is_personal_fund, is_company_fund, is_investment_product, format_amount};

use once_cell::sync::Lazy;
use std::sync::Arc;

/// 全局配置实例
pub static GLOBAL_CONFIG: Lazy<Arc<Config>> = Lazy::new(|| {
    Arc::new(Config::default())
});

/// 全局日志器实例
pub static GLOBAL_LOGGER: Lazy<Arc<Logger>> = Lazy::new(|| {
    Arc::new(Logger::new())
});

/// 获取全局配置
pub fn get_config() -> Arc<Config> {
    GLOBAL_CONFIG.clone()
}

/// 获取全局日志器
pub fn get_logger() -> Arc<Logger> {
    GLOBAL_LOGGER.clone()
}

/// 格式化文件大小
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let index = (bytes_f.log10() / THRESHOLD.log10()).floor() as usize;
    let index = index.min(UNITS.len() - 1);
    
    let value = bytes_f / THRESHOLD.powi(index as i32);
    
    if index == 0 {
        format!("{} {}", bytes, UNITS[index])
    } else {
        format!("{:.2} {}", value, UNITS[index])
    }
}

/// 格式化持续时间
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let millis = duration.subsec_millis();

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}:{:02}.{:03}", minutes, seconds, millis)
    } else if seconds > 0 {
        format!("{}.{:03}s", seconds, millis)
    } else {
        format!("{}ms", millis)
    }
}

/// 格式化百分比
pub fn format_percentage(value: f64, total: f64) -> String {
    if total == 0.0 {
        return "0.00%".to_string();
    }
    
    let percentage = (value / total) * 100.0;
    format!("{:.2}%", percentage)
}

/// 安全除法，避免除零错误
pub fn safe_divide(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() < f64::EPSILON {
        0.0
    } else {
        numerator / denominator
    }
}

/// 比较浮点数是否相等（考虑精度）
pub fn float_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

/// 将字符串转换为数值，支持中文数字格式
pub fn parse_chinese_number(s: &str) -> Option<f64> {
    let s = s.trim().replace(',', "").replace('，', "");
    
    // 处理中文数字单位
    let s = s
        .replace('万', "0000")
        .replace('亿', "00000000")
        .replace('千', "000")
        .replace('百', "00");
    
    s.parse().ok()
}

/// 生成唯一ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 生成短ID（用于显示）
pub fn generate_short_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    format!("{:x}", hasher.finish())[..8].to_string()
}

/// 时间格式化工具
pub mod time_utils {
    use chrono::{DateTime, Local, NaiveDateTime, Utc};

    /// 获取当前本地时间字符串
    pub fn current_local_time_string() -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// 获取当前UTC时间字符串
    pub fn current_utc_time_string() -> String {
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// 格式化时间戳
    pub fn format_timestamp(timestamp: i64) -> String {
        if let Some(dt) = NaiveDateTime::from_timestamp_opt(timestamp, 0) {
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            "Invalid timestamp".to_string()
        }
    }

    /// 解析日期字符串
    pub fn parse_date_string(date_str: &str) -> Option<NaiveDateTime> {
        use chrono::NaiveDate;
        
        // 尝试多种日期格式
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y/%m/%d %H:%M:%S",
            "%Y-%m-%d",
            "%Y/%m/%d",
            "%m/%d/%Y",
            "%d/%m/%Y",
        ];
        
        for format in &formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, format) {
                return Some(dt);
            }
            if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                return Some(date.and_hms_opt(0, 0, 0)?);
            }
        }
        
        None
    }
}

/// 文件操作工具
pub mod file_utils {
    use std::path::{Path, PathBuf};
    use std::fs;

    /// 确保目录存在
    pub fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// 获取文件扩展名
    pub fn get_file_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// 生成备份文件名
    pub fn generate_backup_filename(original: &Path) -> PathBuf {
        let stem = original.file_stem().unwrap_or_default();
        let extension = original.extension().unwrap_or_default();
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        
        let backup_name = format!(
            "{}_{}.{}",
            stem.to_string_lossy(),
            timestamp,
            extension.to_string_lossy()
        );
        
        original.with_file_name(backup_name)
    }

    /// 检查文件是否为Excel格式
    pub fn is_excel_file(path: &Path) -> bool {
        match get_file_extension(path).as_deref() {
            Some("xlsx") | Some("xls") | Some("xlsm") => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_duration() {
        let duration = std::time::Duration::from_millis(1500);
        assert!(format_duration(duration).contains("1.500"));
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(25.0, 100.0), "25.00%");
        assert_eq!(format_percentage(1.0, 3.0), "33.33%");
        assert_eq!(format_percentage(0.0, 0.0), "0.00%");
    }

    #[test]
    fn test_safe_divide() {
        assert_eq!(safe_divide(10.0, 2.0), 5.0);
        assert_eq!(safe_divide(10.0, 0.0), 0.0);
    }

    #[test]
    fn test_float_eq() {
        assert!(float_eq(0.1 + 0.2, 0.3, 1e-10));
        assert!(!float_eq(1.0, 2.0, 1e-10));
    }

    #[test]
    fn test_parse_chinese_number() {
        assert_eq!(parse_chinese_number("1万"), Some(10000.0));
        assert_eq!(parse_chinese_number("2.5万"), Some(25000.0));
        assert_eq!(parse_chinese_number("1,000"), Some(1000.0));
    }

    #[test]
    fn test_generate_ids() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
        
        let short_id = generate_short_id();
        assert_eq!(short_id.len(), 8);
    }
}
