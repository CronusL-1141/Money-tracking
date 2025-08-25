//! 审计服务实现

use crate::algorithms::TrackerFactory;
use crate::interfaces::Tracker;
use crate::models::{Config, Transaction, AuditSummary, FundPoolManager};
use crate::utils::{ExcelProcessor, UnifiedValidator, AuditLogger};
use crate::errors::{AuditError, AuditResult};
use log::{info, warn, error};
use std::path::Path;

/// 审计服务
/// 
/// 提供完整的审计分析功能，包括数据读取、验证、分析和结果导出
#[derive(Debug)]
pub struct AuditService {
    config: Config,
    excel_processor: ExcelProcessor,
    validator: UnifiedValidator,
    tracker_factory: TrackerFactory,
    fund_pool_manager: FundPoolManager,
}

impl AuditService {
    /// 创建新的审计服务
    pub fn new() -> Self {
        let config = Config::new();
        let excel_processor = ExcelProcessor::new(config.clone());
        let validator = UnifiedValidator::new();
        let tracker_factory = TrackerFactory::new(config.clone());
        let fund_pool_manager = FundPoolManager::new();
        
        Self {
            config,
            excel_processor,
            validator,
            tracker_factory,
            fund_pool_manager,
        }
    }
    
    /// 使用指定配置创建审计服务
    pub fn with_config(config: Config) -> Self {
        let excel_processor = ExcelProcessor::new(config.clone());
        let validator = UnifiedValidator::new();
        let tracker_factory = TrackerFactory::new(config.clone());
        let fund_pool_manager = FundPoolManager::new();
        
        Self {
            config,
            excel_processor,
            validator,
            tracker_factory,
            fund_pool_manager,
        }
    }
    
    /// 执行完整的审计分析
    /// 
    /// # Arguments
    /// * `algorithm` - 使用的算法类型
    /// * `input_file` - 输入Excel文件路径
    /// * `output_file` - 输出文件路径（可选）
    /// 
    /// # Returns
    /// * `AuditResult<AuditSummary>` - 审计摘要结果
    pub async fn analyze_financial_data<P: AsRef<Path>>(
        &mut self,
        algorithm: &str,
        input_file: P,
        output_file: Option<P>,
    ) -> AuditResult<AuditSummary> {
        let input_path = input_file.as_ref();
        
        AuditLogger::log_analysis_start(algorithm, input_path.to_str().unwrap_or("unknown"));
        
        // 步骤1: 读取Excel数据（只读，不修改源文件）
        info!("📊 开始读取Excel数据");
        let original_transactions = self.excel_processor.read_transactions(input_path)?;
        info!("✅ 数据读取完成，共读取 {} 条记录", original_transactions.len());
        
        // 步骤2: 数据验证和修复（在内存副本上操作，不修改源数据）
        info!("🔍 开始流水完整性验证和修复");
        let validation_result = self.validator.validate_transactions(&original_transactions)?;
        
        // 使用修复后的清洁数据进行后续处理
        let clean_transactions = if let Some(fixed_data) = validation_result.fixed_transactions {
            info!("✅ 流水完整性验证完成，应用了 {} 次修复", validation_result.optimizations_count);
            fixed_data
        } else {
            info!("✅ 流水完整性验证完成，数据无需修复");
            original_transactions // 数据本身已经是清洁的
        };
        
        if !validation_result.is_valid {
            warn!("数据验证发现 {} 个问题", validation_result.errors_count);
            for error in &validation_result.errors {
                warn!("验证错误: {}", error.message);
            }
            // 如果验证失败且无法修复，返回错误
            if validation_result.optimization_failed {
                return Err(AuditError::validation_error("流水完整性验证失败，数据质量不符合分析要求"));
            }
        }
        
        // 步骤3: 计算初始余额（使用清洁数据）
        info!("💰 计算初始余额");
        let initial_balance = if let Some(first_tx) = clean_transactions.first() {
            first_tx.balance - first_tx.income_amount + first_tx.expense_amount
        } else {
            return Err(AuditError::validation_error("没有找到交易记录"));
        };
        
        // 步骤4: 创建追踪器并初始化
        info!("🚀 开始{}资金追踪分析", algorithm);
        let mut tracker = self.tracker_factory.create_tracker(algorithm)?;
        
        // 根据第一笔交易的资金属性确定初始余额类型
        let balance_type = &clean_transactions[0].fund_attribute;
        tracker.initialize_balance(initial_balance, balance_type)?;
        
        // 步骤5: 处理所有交易记录（使用清洁数据进行业务分析）
        let total_count = clean_transactions.len();
        info!("📋 总共需要处理 {} 条交易记录", total_count);
        
        // 创建可变副本用于业务处理（不影响清洁数据）
        let mut processed_transactions = clean_transactions.clone();
        
        for (index, transaction) in processed_transactions.iter_mut().enumerate() {
            if let Err(e) = tracker.process_transaction(transaction) {
                error!("处理第{}条交易记录失败: {}", index + 1, e);
                return Err(e);
            }
            
            // 定期报告进度
            if (index + 1) % 1000 == 0 {
                AuditLogger::log_progress(index + 1, total_count);
            }
        }
        
        info!("✅ 所有 {} 条交易记录处理完成", processed_transactions.len());
        
        // 步骤6: 获取审计摘要
        info!("📈 生成分析结果");
        let summary = tracker.get_summary()?;
        
        // 步骤7: 导出结果（使用处理后的数据，不影响源文件）
        if let Some(output_path) = output_file {
            info!("💾 保存分析结果到: {}", output_path.as_ref().display());
            self.excel_processor.export_analysis_results(&processed_transactions, &summary, output_path)?;
        } else {
            // 使用默认输出文件名
            let default_output = format!("{}_资金追踪结果.xlsx", algorithm);
            info!("💾 保存分析结果到: {}", default_output);
            self.excel_processor.export_analysis_results(&processed_transactions, &summary, &default_output)?;
        }
        
        // 步骤9: 导出资金池记录（如果有） - 暂时禁用，等完整Excel处理器实现
        // let pool_records: Vec<_> = self.fund_pool_manager.pools.values().flatten().cloned().collect();
        // if !pool_records.is_empty() {
        //     let pool_output = format!("场外资金池记录_{}.xlsx", algorithm);
        //     info!("📋 生成场外资金池记录: {}", pool_output);
        //     self.excel_processor.export_fund_pool_records(&pool_records, &pool_output)?;
        // }
        
        AuditLogger::log_analysis_complete(algorithm, processed_transactions.len());
        
        Ok(summary)
    }
    
    /// 比较两种算法的结果
    pub async fn compare_algorithms<P: AsRef<Path>>(
        &mut self,
        input_file: P,
    ) -> AuditResult<ComparisonResult> {
        info!("🔄 开始算法对比分析");
        
        let fifo_summary = self.analyze_financial_data("FIFO", &input_file, None).await?;
        let balance_summary = self.analyze_financial_data("BALANCE_METHOD", &input_file, None).await?;
        
        let comparison = ComparisonResult {
            fifo_summary,
            balance_method_summary: balance_summary,
            differences: Vec::new(), // 简化实现，实际应该计算具体差异
        };
        
        info!("✅ 算法对比完成");
        Ok(comparison)
    }
    
    /// 时点查询
    pub async fn query_time_point<P: AsRef<Path>>(
        &mut self,
        input_file: P,
        row_number: usize,
        algorithm: &str,
    ) -> AuditResult<AuditSummary> {
        info!("🔍 执行时点查询: 第{}行", row_number);
        
        // 读取数据
        let mut transactions = self.excel_processor.read_transactions(input_file)?;
        DataProcessor::preprocess_transactions(&mut transactions)?;
        
        if row_number == 0 || row_number > transactions.len() {
            return Err(AuditError::validation_error("行号超出范围"));
        }
        
        // 创建追踪器
        let mut tracker = self.tracker_factory.create_tracker(algorithm)?;
        
        // 初始化
        let initial_balance = if let Some(first_tx) = transactions.first() {
            first_tx.balance - first_tx.income_amount + first_tx.expense_amount
        } else {
            return Err(AuditError::validation_error("没有找到交易记录"));
        };
        
        let balance_type = &transactions[0].fund_attribute;
        tracker.initialize_balance(initial_balance, balance_type)?;
        
        // 处理到指定行
        for transaction in transactions.iter_mut().take(row_number) {
            tracker.process_transaction(transaction)?;
        }
        
        let summary = tracker.get_summary()?;
        info!("✅ 时点查询完成");
        
        Ok(summary)
    }
    
    /// 获取支持的算法列表
    pub fn get_supported_algorithms(&self) -> Vec<&'static str> {
        TrackerFactory::get_supported_algorithms()
    }
    
    /// 获取算法信息
    pub fn get_algorithms_info(&self) -> std::collections::HashMap<&'static str, &'static str> {
        TrackerFactory::get_algorithms_info()
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new()
    }
}

/// 算法比较结果
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    /// FIFO算法结果
    pub fifo_summary: AuditSummary,
    
    /// 差额计算法结果
    pub balance_method_summary: AuditSummary,
    
    /// 差异列表
    pub differences: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_audit_service_creation() {
        let service = AuditService::new();
        let algorithms = service.get_supported_algorithms();
        
        assert!(!algorithms.is_empty());
        assert!(algorithms.contains(&"FIFO"));
        assert!(algorithms.contains(&"BALANCE_METHOD"));
    }
    
    #[test]
    fn test_algorithms_info() {
        let service = AuditService::new();
        let info = service.get_algorithms_info();
        
        assert!(info.contains_key("FIFO"));
        assert!(info.contains_key("BALANCE_METHOD"));
    }
}