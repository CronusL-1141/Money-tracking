//! # 性能优化模块
//! 
//! 提供SIMD加速、内存池管理、并行处理、缓存优化等性能优化功能。
//! 
//! ## 优化策略
//! 
//! - **SIMD优化**: 使用SIMD指令加速数值计算
//! - **内存池**: 减少内存分配开销
//! - **并行处理**: 充分利用多核CPU
//! - **缓存优化**: 智能缓存热点数据
//! - **数据局部性**: 优化数据访问模式

// TODO: 暂时注释掉子模块，等实现时再启用
// pub mod simd;
// pub mod memory_pool;
// pub mod parallel;
// pub mod cache;

// // 重新导出主要类型
// pub use simd::SimdCalculator;
// pub use memory_pool::{MemoryPool, PooledObject};
// pub use parallel::ParallelProcessor;
// pub use cache::{LruCache, CacheManager};

// use std::sync::Arc;
// use parking_lot::Mutex;

/// 性能统计信息
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// 处理总数
    pub total_processed: u64,
    
    /// 总处理时间（微秒）
    pub total_time_us: u64,
    
    /// 缓存命中次数
    pub cache_hits: u64,
    
    /// 缓存未命中次数
    pub cache_misses: u64,
    
    /// SIMD加速使用次数
    pub simd_operations: u64,
    
    /// 并行处理使用次数
    pub parallel_operations: u64,
    
    /// 内存池分配次数
    pub pool_allocations: u64,
    
    /// 内存池复用次数
    pub pool_reuses: u64,
}

impl PerformanceStats {
    /// 创建新的性能统计
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加处理统计
    pub fn add_processing(&mut self, count: u64, time_us: u64) {
        self.total_processed += count;
        self.total_time_us += time_us;
    }

    /// 添加缓存统计
    pub fn add_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    pub fn add_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// 计算平均处理时间（微秒）
    pub fn average_time_us(&self) -> f64 {
        if self.total_processed == 0 {
            0.0
        } else {
            self.total_time_us as f64 / self.total_processed as f64
        }
    }

    /// 计算缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// 计算每秒处理数
    pub fn throughput_per_second(&self) -> f64 {
        if self.total_time_us == 0 {
            0.0
        } else {
            self.total_processed as f64 / (self.total_time_us as f64 / 1_000_000.0)
        }
    }

    /// 获取格式化的统计报告
    pub fn format_report(&self) -> String {
        format!(
            "性能统计报告:\n\
            总处理数: {}\n\
            总时间: {:.2}s\n\
            平均时间: {:.2}μs\n\
            吞吐量: {:.0}/s\n\
            缓存命中率: {:.2}%\n\
            SIMD操作: {}\n\
            并行操作: {}\n\
            内存池命中率: {:.2}%",
            self.total_processed,
            self.total_time_us as f64 / 1_000_000.0,
            self.average_time_us(),
            self.throughput_per_second(),
            self.cache_hit_rate() * 100.0,
            self.simd_operations,
            self.parallel_operations,
            if self.pool_allocations > 0 {
                self.pool_reuses as f64 / (self.pool_allocations + self.pool_reuses) as f64 * 100.0
            } else {
                0.0
            }
        )
    }
}

/// 全局性能统计
static GLOBAL_STATS: parking_lot::Mutex<PerformanceStats> = parking_lot::Mutex::new(PerformanceStats {
    total_processed: 0,
    total_time_us: 0,
    cache_hits: 0,
    cache_misses: 0,
    simd_operations: 0,
    parallel_operations: 0,
    pool_allocations: 0,
    pool_reuses: 0,
});

/// 获取全局性能统计
pub fn get_global_stats() -> PerformanceStats {
    GLOBAL_STATS.lock().clone()
}

/// 重置全局性能统计
pub fn reset_global_stats() {
    *GLOBAL_STATS.lock() = PerformanceStats::new();
}

/// 更新全局性能统计
pub fn update_global_stats<F>(updater: F) 
where 
    F: FnOnce(&mut PerformanceStats),
{
    let mut stats = GLOBAL_STATS.lock();
    updater(&mut stats);
}

/// 性能监控器
pub struct PerformanceMonitor {
    start_time: std::time::Instant,
    name: String,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            name: name.into(),
        }
    }

    /// 停止监控并返回耗时
    pub fn stop(self) -> std::time::Duration {
        let duration = self.start_time.elapsed();
        tracing::debug!(
            "Performance: {} took {:?}",
            self.name,
            duration
        );
        duration
    }

    /// 停止监控并更新全局统计
    pub fn stop_and_update(self, processed_count: u64) -> std::time::Duration {
        let duration = self.stop();
        update_global_stats(|stats| {
            stats.add_processing(processed_count, duration.as_micros() as u64);
        });
        duration
    }
}

/// 批处理优化器
pub struct BatchProcessor<T> {
    batch_size: usize,
    buffer: Vec<T>,
    processor: Box<dyn Fn(&[T]) -> crate::utils::Result<()> + Send + Sync>,
}

impl<T> BatchProcessor<T> {
    /// 创建新的批处理优化器
    pub fn new<F>(batch_size: usize, processor: F) -> Self 
    where
        F: Fn(&[T]) -> crate::utils::Result<()> + Send + Sync + 'static,
    {
        Self {
            batch_size,
            buffer: Vec::with_capacity(batch_size),
            processor: Box::new(processor),
        }
    }

    /// 添加项目到批处理队列
    pub fn push(&mut self, item: T) -> crate::utils::Result<()> {
        self.buffer.push(item);
        
        if self.buffer.len() >= self.batch_size {
            self.flush()?;
        }
        
        Ok(())
    }

    /// 刷新批处理队列
    pub fn flush(&mut self) -> crate::utils::Result<()> {
        if !self.buffer.is_empty() {
            (self.processor)(&self.buffer)?;
            self.buffer.clear();
        }
        Ok(())
    }

    /// 获取当前缓冲区大小
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}

impl<T> Drop for BatchProcessor<T> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// 自适应批处理器
pub struct AdaptiveBatchProcessor<T> {
    min_batch_size: usize,
    max_batch_size: usize,
    current_batch_size: usize,
    buffer: Vec<T>,
    processor: Box<dyn Fn(&[T]) -> crate::utils::Result<()> + Send + Sync>,
    performance_history: Vec<f64>, // throughput history
    adaptation_threshold: usize,
}

impl<T> AdaptiveBatchProcessor<T> {
    /// 创建自适应批处理器
    pub fn new<F>(
        min_batch_size: usize, 
        max_batch_size: usize,
        processor: F
    ) -> Self 
    where
        F: Fn(&[T]) -> crate::utils::Result<()> + Send + Sync + 'static,
    {
        let initial_size = (min_batch_size + max_batch_size) / 2;
        
        Self {
            min_batch_size,
            max_batch_size,
            current_batch_size: initial_size,
            buffer: Vec::with_capacity(initial_size),
            processor: Box::new(processor),
            performance_history: Vec::new(),
            adaptation_threshold: 10,
        }
    }

    /// 添加项目并可能触发自适应调整
    pub fn push(&mut self, item: T) -> crate::utils::Result<()> {
        self.buffer.push(item);
        
        if self.buffer.len() >= self.current_batch_size {
            let start = std::time::Instant::now();
            self.flush_internal()?;
            let duration = start.elapsed();
            
            // 记录性能并可能调整批大小
            let throughput = self.current_batch_size as f64 / duration.as_secs_f64();
            self.performance_history.push(throughput);
            
            if self.performance_history.len() >= self.adaptation_threshold {
                self.adapt_batch_size();
            }
        }
        
        Ok(())
    }

    fn flush_internal(&mut self) -> crate::utils::Result<()> {
        if !self.buffer.is_empty() {
            (self.processor)(&self.buffer)?;
            self.buffer.clear();
        }
        Ok(())
    }

    fn adapt_batch_size(&mut self) {
        let recent_avg = self.performance_history
            .iter()
            .rev()
            .take(5)
            .sum::<f64>() / 5.0;

        let older_avg = if self.performance_history.len() > 5 {
            self.performance_history
                .iter()
                .rev()
                .skip(5)
                .take(5)
                .sum::<f64>() / 5.0
        } else {
            recent_avg
        };

        // 如果性能提升，增加批大小；如果性能下降，减少批大小
        if recent_avg > older_avg * 1.1 {
            self.current_batch_size = (self.current_batch_size * 2).min(self.max_batch_size);
        } else if recent_avg < older_avg * 0.9 {
            self.current_batch_size = (self.current_batch_size / 2).max(self.min_batch_size);
        }

        // 调整缓冲区容量
        self.buffer.reserve(self.current_batch_size.saturating_sub(self.buffer.capacity()));
        
        // 限制历史记录长度
        if self.performance_history.len() > 50 {
            self.performance_history.drain(0..25);
        }

        tracing::debug!(
            "Adapted batch size to {} (throughput: {:.2}/s)", 
            self.current_batch_size, 
            recent_avg
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_stats() {
        let mut stats = PerformanceStats::new();
        stats.add_processing(100, 1000);
        stats.add_cache_hit();
        stats.add_cache_miss();

        assert_eq!(stats.total_processed, 100);
        assert_eq!(stats.average_time_us(), 10.0);
        assert_eq!(stats.cache_hit_rate(), 0.5);
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new("test");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let duration = monitor.stop();
        assert!(duration.as_millis() >= 1);
    }

    #[test]
    fn test_batch_processor() {
        use std::sync::{Arc, Mutex};
        
        let processed = Arc::new(Mutex::new(Vec::new()));
        let processed_clone = processed.clone();
        
        let mut processor = BatchProcessor::new(3, move |batch: &[i32]| {
            processed_clone.lock().unwrap().extend_from_slice(batch);
            Ok(())
        });

        processor.push(1).unwrap();
        processor.push(2).unwrap();
        assert_eq!(processed.lock().unwrap().len(), 0); // Not flushed yet
        
        processor.push(3).unwrap(); // Should trigger flush
        assert_eq!(processed.lock().unwrap().len(), 3);
        
        processor.push(4).unwrap();
        processor.flush().unwrap(); // Manual flush
        assert_eq!(processed.lock().unwrap().len(), 4);
    }
}
