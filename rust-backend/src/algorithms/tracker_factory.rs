//! 追踪器工厂

use crate::interfaces::Tracker;
use crate::algorithms::{FifoTracker, BalanceMethodTracker};
use crate::errors::{AuditError, AuditResult};
use crate::models::Config;
use std::collections::HashMap;

/// 追踪器工厂
/// 
/// 负责创建和管理不同类型的追踪器
#[derive(Debug, Clone)]
pub struct TrackerFactory {
    config: Config,
}

impl TrackerFactory {
    /// 创建新的工厂
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 创建指定类型的追踪器
    pub fn create_tracker(&self, algorithm: &str) -> AuditResult<Box<dyn Tracker>> {
        match algorithm.to_uppercase().as_str() {
            "FIFO" => Ok(Box::new(FifoTracker::new(self.config.clone()))),
            "BALANCE_METHOD" => Ok(Box::new(BalanceMethodTracker::new(self.config.clone()))),
            _ => Err(AuditError::unsupported_operation(
                format!("不支持的算法类型: {}", algorithm)
            )),
        }
    }
    
    /// 获取支持的算法列表
    pub fn get_supported_algorithms() -> Vec<&'static str> {
        vec!["FIFO", "BALANCE_METHOD"]
    }
    
    /// 获取算法信息
    pub fn get_algorithms_info() -> HashMap<&'static str, &'static str> {
        let mut info = HashMap::new();
        info.insert("FIFO", "先进先出算法 - 按时间顺序处理资金流动");
        info.insert("BALANCE_METHOD", "差额计算法 - 个人优先扣除的简化算法");
        info
    }
    
    /// 获取算法描述
    pub fn get_algorithm_description(algorithm: &str) -> AuditResult<&'static str> {
        match algorithm.to_uppercase().as_str() {
            "FIFO" => Ok("先进先出算法 - 按时间顺序处理资金流动"),
            "BALANCE_METHOD" => Ok("差额计算法 - 个人优先扣除的简化算法"),
            _ => Err(AuditError::unsupported_operation(
                format!("未知算法: {}", algorithm)
            )),
        }
    }
    
    /// 验证算法名称
    pub fn is_valid_algorithm(algorithm: &str) -> bool {
        Self::get_supported_algorithms().contains(&algorithm.to_uppercase().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_fifo_tracker() {
        let config = Config::new();
        let factory = TrackerFactory::new(config);
        
        let tracker = factory.create_tracker("FIFO").unwrap();
        assert_eq!(tracker.get_name(), "FIFO");
    }
    
    #[test]
    fn test_create_balance_method_tracker() {
        let config = Config::new();
        let factory = TrackerFactory::new(config);
        
        let tracker = factory.create_tracker("BALANCE_METHOD").unwrap();
        assert_eq!(tracker.get_name(), "BALANCE_METHOD");
    }
    
    #[test]
    fn test_unsupported_algorithm() {
        let config = Config::new();
        let factory = TrackerFactory::new(config);
        
        let result = factory.create_tracker("UNKNOWN");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_algorithm_validation() {
        assert!(TrackerFactory::is_valid_algorithm("FIFO"));
        assert!(TrackerFactory::is_valid_algorithm("fifo"));
        assert!(TrackerFactory::is_valid_algorithm("BALANCE_METHOD"));
        assert!(!TrackerFactory::is_valid_algorithm("UNKNOWN"));
    }
    
    #[test]
    fn test_algorithms_info() {
        let info = TrackerFactory::get_algorithms_info();
        assert!(info.contains_key("FIFO"));
        assert!(info.contains_key("BALANCE_METHOD"));
        assert_eq!(info.len(), 2);
    }
}