//! 日志管理工具

use log::{info, warn, error, debug};
use rust_decimal::Decimal;
use std::fs;
use std::path::Path;

/// 审计日志管理器
#[derive(Debug)]
pub struct AuditLogger;

impl AuditLogger {
    /// 创建新的日志实例
    pub fn new(_name: &str) -> Self {
        Self
    }
}

impl AuditLogger {
    /// 初始化日志系统
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        // 创建日志目录
        let log_dir = Path::new("logs");
        if !log_dir.exists() {
            fs::create_dir_all(log_dir)?;
        }
        
        // 使用env_logger，可以通过环境变量控制日志级别
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .format_timestamp_secs()
            .init();
        
        info!("📝 日志系统初始化完成");
        Ok(())
    }
    
    /// 记录算法分析开始
    pub fn log_analysis_start(algorithm: &str, input_file: &str) {
        info!("🚀 启动算法: {}", algorithm);
        info!("📂 输入文件: {}", input_file);
    }
    
    /// 记录处理进度
    pub fn log_progress(current: usize, total: usize) {
        let percentage = if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        info!("⏳ 处理进度: {}/{} ({:.1}%)", current, total, percentage);
    }
    
    /// 记录算法完成
    pub fn log_analysis_complete(algorithm: &str, record_count: usize) {
        info!("✅ {}算法分析完成", algorithm);
        info!("📊 处理记录数: {}", record_count);
    }
    
    /// 记录错误信息
    pub fn log_error(context: &str, error: &dyn std::error::Error) {
        error!("❌ {}: {}", context, error);
    }
    
    /// 记录警告信息
    pub fn log_warning(message: &str) {
        warn!("⚠️ {}", message);
    }
    
    /// 记录调试信息
    pub fn log_debug(message: &str) {
        debug!("🔍 {}", message);
    }
    
    /// 记录交易处理
    pub fn log_transaction(
        row_idx: usize,
        transaction_type: &str,
        amount: Decimal,
        fund_attribute: &str,
        personal_ratio: Decimal,
        company_ratio: Decimal,
        behavior: &str,
    ) {
        debug!(
            "📝 [行{}] {}: 金额={}, 资金属性={}, 个人占比={:.2}%, 公司占比={:.2}%, 行为={}",
            row_idx + 2,  // Excel行号从2开始（跳过表头）
            transaction_type,
            amount,
            fund_attribute,
            personal_ratio * Decimal::from(100),
            company_ratio * Decimal::from(100),
            behavior
        );
    }
    
    /// 记录资金池操作
    pub fn log_fund_pool_operation(
        product_name: &str,
        operation: &str,
        amount: Decimal,
        personal_amount: Decimal,
        company_amount: Decimal,
    ) {
        info!(
            "💰 资金池[{}] {}: 总金额={}, 个人={}, 公司={}",
            product_name,
            operation,
            amount,
            personal_amount,
            company_amount
        );
    }
    
    /// 记录统计汇总
    pub fn log_summary(
        total_misappropriation: Decimal,
        total_advance: Decimal,
        total_illegal_gains: Decimal,
    ) {
        info!("📊 ========== 统计汇总 ==========");
        info!("💸 累计挪用金额: {}", total_misappropriation);
        info!("💳 累计垫付金额: {}", total_advance);
        info!("🚫 累计非法所得: {}", total_illegal_gains);
        info!("📊 ==============================");
    }
}