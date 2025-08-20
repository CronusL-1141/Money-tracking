//! # 统计数据结构
//! 
//! 定义各种统计信息的数据结构。

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 追踪器统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackerStatistics {
    pub total_transactions: usize,
    pub processing_time: Duration,
    pub memory_usage: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// 处理统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessingStats {
    pub rows_processed: usize,
    pub errors_count: usize,
    pub warnings_count: usize,
    pub start_time: Option<chrono::NaiveDateTime>,
    pub end_time: Option<chrono::NaiveDateTime>,
}

/// 状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSummary {
    pub total_balance: f64,
    pub personal_balance: f64,
    pub company_balance: f64,
    pub active_pools: usize,
    pub last_updated: chrono::NaiveDateTime,
}
