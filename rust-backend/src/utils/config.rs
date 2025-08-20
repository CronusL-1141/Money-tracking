//! # 配置管理模块
//! 
//! 管理系统配置参数。

use serde::{Deserialize, Serialize};

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 处理器配置
    pub processor: ProcessorConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 性能配置
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// 最大处理行数
    pub max_rows: usize,
    /// 批处理大小
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 是否输出到文件
    pub file_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 工作线程数
    pub worker_threads: usize,
    /// 是否启用SIMD
    pub enable_simd: bool,
    /// 是否启用缓存
    pub enable_cache: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            processor: ProcessorConfig {
                max_rows: 1_000_000,
                batch_size: 1000,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_output: false,
            },
            performance: PerformanceConfig {
                worker_threads: num_cpus::get(),
                enable_simd: true,
                enable_cache: true,
            },
        }
    }
}
