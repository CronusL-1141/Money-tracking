//! # FIFO队列数据结构
//! 
//! FIFO算法的核心队列实现。

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// FIFO队列项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FifoEntry {
    pub amount: f64,
    pub timestamp: chrono::NaiveDateTime,
    pub fund_type: String,
}

/// FIFO队列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FifoQueue {
    queue: VecDeque<FifoEntry>,
}

impl FifoQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    
    pub fn push(&mut self, entry: FifoEntry) {
        self.queue.push_back(entry);
    }
    
    pub fn pop(&mut self) -> Option<FifoEntry> {
        self.queue.pop_front()
    }
    
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// 流出结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutflowResult {
    pub processed_amount: f64,
    pub remaining_entries: Vec<FifoEntry>,
}
