//! 审计服务层
//! 
//! 提供统一的业务API，协调各层组件完成资金追踪分析
//! 支持进度回调和用户反馈机制

use crate::data_models::{
    Config, AuditSummary, Transaction, 
    TauriAuditConfig, TauriAuditResult, TauriProcessStatus
};
use crate::utils::{ExcelProcessor, UnifiedValidator};
use crate::algorithms::{FifoTracker, BalanceMethodTracker};
use crate::errors::{AuditError, AuditResult};
use log::info;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

/// 进度报告信息
#[derive(Debug, Clone)]
pub struct ProgressReport {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub percentage: f64,
    pub message: String,
}

/// 阶段状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStage {
    DataPreprocessing,
    FlowValidation,
    DataValidation,
    InitialBalanceCalculation,
    AlgorithmProcessing,
    ResultGeneration,
    ResultExport,
}

/// 进度回调函数类型
pub type ProgressCallback = Arc<dyn Fn(ProgressReport) + Send + Sync>;

/// 阶段回调函数类型
pub type StageCallback = Arc<dyn Fn(ProcessingStage, &str) + Send + Sync>;

/// 审计服务 - 核心业务服务接口
pub struct AuditService {
    config: Config,
    progress_callback: Option<ProgressCallback>,
    stage_callback: Option<StageCallback>,
    suppress_output: bool,
    // GUI状态管理
    current_status: Arc<Mutex<TauriProcessStatus>>,
    output_log: Arc<Mutex<Vec<String>>>,
}

impl AuditService {
    /// 创建审计服务实例
    pub fn new() -> Self {
        Self {
            config: Config::new(),
            progress_callback: None,
            stage_callback: None,
            suppress_output: false,
            current_status: Arc::new(Mutex::new(TauriProcessStatus::idle())),
            output_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// 使用自定义配置创建审计服务
    pub fn with_config(config: Config) -> Self {
        Self { 
            config,
            progress_callback: None,
            stage_callback: None,
            suppress_output: false,
            current_status: Arc::new(Mutex::new(TauriProcessStatus::idle())),
            output_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// 设置进度回调
    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }
    
    /// 设置阶段回调
    pub fn with_stage_callback(mut self, callback: StageCallback) -> Self {
        self.stage_callback = Some(callback);
        self
    }
    
    /// 设置是否抑制输出
    pub fn with_suppress_output(mut self, suppress: bool) -> Self {
        self.suppress_output = suppress;
        self
    }
    
    
    
    /// 报告进度
    fn report_progress(&self, stage: &str, current: usize, total: usize, message: &str) {
        let percentage = if total > 0 { (current as f64 / total as f64) * 100.0 } else { 0.0 };
        
        let report = ProgressReport {
            stage: stage.to_string(),
            current,
            total,
            percentage,
            message: message.to_string(),
        };
        
        // 更新GUI状态
        if let Ok(mut status) = self.current_status.lock() {
            *status = TauriProcessStatus::running(percentage, message.to_string());
        }
        
        // 添加日志
        self.add_output_log(&format!("⏳ {}: {}/{} ({:.1}%) - {}", stage, current, total, percentage, message));
        
        if let Some(ref callback) = self.progress_callback {
            callback(report);
        } else if !self.suppress_output {
            println!("⏳ {}: {}/{} ({:.1}%) - {}", stage, current, total, percentage, message);
        }
    }
    
    /// 报告阶段状态
    fn report_stage(&self, stage: ProcessingStage, message: &str) {
        let emoji = match stage {
            ProcessingStage::DataPreprocessing => "📊",
            ProcessingStage::FlowValidation => "🔍",
            ProcessingStage::DataValidation => "🔎",
            ProcessingStage::InitialBalanceCalculation => "💰",
            ProcessingStage::AlgorithmProcessing => "🚀",
            ProcessingStage::ResultGeneration => "📈",
            ProcessingStage::ResultExport => "💾",
        };
        
        let log_message = format!("{} {}", emoji, message);
        
        // 添加日志
        self.add_output_log(&log_message);
        
        if let Some(ref callback) = self.stage_callback {
            callback(stage, message);
        } else if !self.suppress_output {
            println!("{}", log_message);
        }
    }
    
    /// 添加输出日志
    fn add_output_log(&self, message: &str) {
        if let Ok(mut log) = self.output_log.lock() {
            log.push(message.to_string());
            // 限制日志数量防止内存溢出
            if log.len() > 1000 {
                log.drain(..500); // 保留最后500条
            }
        }
    }
    
    /// 执行完整的审计分析 - 主要业务API
    /// 
    /// # Arguments  
    /// * `algorithm` - 算法类型 ("FIFO" 或 "BALANCE_METHOD")
    /// * `input_file` - 输入Excel文件路径
    /// * `output_file` - 输出文件路径（可选）
    /// 
    /// # Returns
    /// 返回审计摘要结果
    pub async fn analyze<P: AsRef<Path>>(
        &self,
        algorithm: &str,
        input_file: P,
        output_file: Option<P>,
    ) -> AuditResult<AuditSummary> {
        info!("开始{}算法审计分析", algorithm);
        
        // 步骤1: 数据加载和验证
        let transactions = self.load_and_validate_data(input_file).await?;
        
        // 步骤2: 执行算法分析
        let (summary, processed_transactions) = self.execute_algorithm(algorithm, &transactions).await?;
        
        // 步骤3: 导出结果（可选）
        if let Some(output_path) = output_file {
            self.export_results(&processed_transactions, &summary, output_path)?;
        }
        
        info!("审计分析完成");
        Ok(summary)
    }
    
    /// 数据加载和验证
    async fn load_and_validate_data<P: AsRef<Path>>(&self, input_file: P) -> AuditResult<Vec<Transaction>> {
        info!("加载和验证数据");
        
        // 1. 数据预处理
        self.report_stage(ProcessingStage::DataPreprocessing, "开始数据预处理...");
        let excel_processor = ExcelProcessor::new(self.config.clone());
        let transactions = excel_processor.read_transactions(input_file)?;
        
        let transaction_count = transactions.len();
        self.report_stage(
            ProcessingStage::DataPreprocessing, 
            &format!("数据预处理完成，共加载 {} 条记录", transaction_count)
        );
        
        // 2. 流水完整性验证
        self.report_stage(ProcessingStage::FlowValidation, "开始流水完整性验证...");
        let mut validator = UnifiedValidator::new();
        let validation_result = validator.validate_transactions(&transactions);
        
        match validation_result {
            Ok(result) => {
                // 显示详细的验证和修复信息
                if result.optimizations_count > 0 {
                    // 总发现错误数 = 成功修复数 + 未修复错误数
                    let total_issues_found = result.optimizations_count + result.errors_count;
                    self.report_stage(
                        ProcessingStage::FlowValidation, 
                        &format!("流水完整性验证: 发现{}处顺序错误，贪心算法成功修复{}处", 
                            total_issues_found, result.optimizations_count)
                    );
                } else if result.errors_count > 0 {
                    self.report_stage(
                        ProcessingStage::FlowValidation, 
                        &format!("流水完整性验证: 发现{}处错误，无需修复", result.errors_count)
                    );
                } else {
                    self.report_stage(ProcessingStage::FlowValidation, "流水完整性验证通过，数据完整无错误");
                }
                
                // 使用修复后的数据（如果有修复的话）
                Ok(result.fixed_transactions.unwrap_or(transactions))
            }
            Err(e) => {
                self.report_stage(
                    ProcessingStage::FlowValidation, 
                    &format!("流水完整性验证失败: {}", e)
                );
                Err(e)
            }
        }
    }
    
    /// 执行算法分析
    async fn execute_algorithm(
        &self, 
        algorithm: &str, 
        transactions: &[Transaction]
    ) -> AuditResult<(AuditSummary, Vec<Transaction>)> {
        match algorithm {
            "FIFO" => self.run_fifo_algorithm(transactions).await,
            "BALANCE_METHOD" => self.run_balance_method_algorithm(transactions).await,
            _ => Err(AuditError::validation_error(&format!("不支持的算法: {}", algorithm))),
        }
    }
    
    /// 运行FIFO算法
    async fn run_fifo_algorithm(&self, transactions: &[Transaction]) -> AuditResult<(AuditSummary, Vec<Transaction>)> {
        info!("执行FIFO算法分析");
        
        let mut tracker = FifoTracker::new(self.config.clone());
        let processed_transactions = self.process_transactions_with_tracker(&mut tracker, transactions, "FIFO").await?;
        let summary = tracker.get_summary()?;
        
        Ok((summary, processed_transactions))
    }
    
    /// 运行差额计算法
    async fn run_balance_method_algorithm(&self, transactions: &[Transaction]) -> AuditResult<(AuditSummary, Vec<Transaction>)> {
        info!("执行差额计算法分析");
        
        let mut tracker = BalanceMethodTracker::new(self.config.clone());
        let processed_transactions = self.process_transactions_with_tracker(&mut tracker, transactions, "BALANCE_METHOD").await?;
        let summary = tracker.get_summary()?;
        
        Ok((summary, processed_transactions))
    }
    
    /// 通用交易处理逻辑 - 使用trait对象避免重复代码
    async fn process_transactions_with_tracker<T>(
        &self,
        tracker: &mut T,
        transactions: &[Transaction],
        algorithm_name: &str,
    ) -> AuditResult<Vec<Transaction>> 
    where
        T: TransactionProcessor,
    {
        if transactions.is_empty() {
            return Err(AuditError::validation_error("没有交易数据"));
        }
        
        let total_count = transactions.len();
        
        // 智能初始化
        self.report_stage(
            ProcessingStage::InitialBalanceCalculation,
            &format!("计算初始余额...")
        );
        tracker.smart_initialize(&transactions[0])?;
        
        // 开始算法处理
        self.report_stage(
            ProcessingStage::AlgorithmProcessing,
            &format!("开始 {} 资金追踪分析...", algorithm_name)
        );
        
        let log_message = format!("📋 总共需要处理 {} 条交易记录", total_count);
        
        // 添加到GUI日志
        self.add_output_log(&log_message);
        
        if !self.suppress_output {
            println!("{}", log_message);
        }
        
        // 处理所有交易 - 每1000条显示一次具体进度
        let mut processed_transactions = Vec::with_capacity(transactions.len());
        
        for (index, tx) in transactions.iter().enumerate() {
            let processed_tx = tracker.process_transaction(tx)?;
            processed_transactions.push(processed_tx);
            
            // 每1000条报告一次进度（显示实际处理条数）
            if (index + 1) % 1000 == 0 || (index + 1) == total_count {
                let progress_percentage = (index + 1) as f64 / total_count as f64 * 100.0;
                self.add_output_log(&format!("⏳ 交易处理: {}/{} ({:.1}%) - 处理 {} 算法交易", 
                    index + 1, total_count, progress_percentage, algorithm_name));
            }
        }
        
        let completion_message = format!("✅ 所有 {} 条交易记录处理完成", total_count);
        
        // 添加到GUI日志
        self.add_output_log(&completion_message);
        
        if !self.suppress_output {
            println!("{}", completion_message);
        }
        
        Ok(processed_transactions)
    }
    
    /// 导出分析结果
    fn export_results<P: AsRef<Path>>(
        &self,
        transactions: &[Transaction],
        summary: &AuditSummary,
        output_path: P,
    ) -> AuditResult<()> {
        self.report_stage(ProcessingStage::ResultExport, "生成分析结果...");
        
        let excel_processor = ExcelProcessor::new(self.config.clone());
        excel_processor.export_analysis_results(transactions, summary, &output_path)?;
        
        let output_file = output_path.as_ref().display().to_string();
        self.report_stage(
            ProcessingStage::ResultExport,
            &format!("分析结果已保存到: {}", output_file)
        );
        
        info!("结果已导出到: {}", output_file);
        Ok(())
    }
    
    /// 获取算法信息
    pub fn get_algorithms_info(&self) -> HashMap<&'static str, &'static str> {
        let mut info = HashMap::new();
        info.insert("FIFO", "先进先出算法 - 按时间顺序追踪资金流向");
        info.insert("BALANCE_METHOD", "差额计算法 - 基于余额变化计算资金占比");
        info
    }
    
    /// 分析财务数据（兼容Python接口）
    pub async fn analyze_financial_data<P: AsRef<Path>>(
        &self,
        algorithm: &str,
        input_file: P,
        output_file: Option<P>,
    ) -> AuditResult<(AuditSummary, Vec<Transaction>, String)> {
        let start_time = std::time::Instant::now();
        
        // 步骤1: 数据加载和验证
        let transactions = self.load_and_validate_data(&input_file).await?;
        let total_records = transactions.len() as u32;
        
        // 步骤2: 执行算法分析
        let (summary, processed_transactions) = self.execute_algorithm(algorithm, &transactions).await?;
        
        // 步骤3: 生成输出文件路径（默认使用临时目录）
        let output_path = if let Some(output_path) = output_file {
            output_path.as_ref().to_path_buf()
        } else {
            // 生成临时文件路径
            self.generate_temp_output_path(algorithm, &input_file)?
        };
        
        // 步骤4: 导出结果
        self.export_results(&processed_transactions, &summary, &output_path)?;
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // 记录统计信息到service中以便GUI获取
        
        // 确保返回绝对路径
        let absolute_path = if output_path.is_absolute() {
            output_path.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(&output_path)
        };
        
        Ok((summary, processed_transactions, absolute_path.display().to_string()))
    }
    
    /// 生成临时输出文件路径
    fn generate_temp_output_path<P: AsRef<Path>>(&self, algorithm: &str, input_file: P) -> AuditResult<std::path::PathBuf> {
        use std::fs;
        use std::path::PathBuf;
        
        // 创建临时目录
        let temp_dir = PathBuf::from("temp_analysis_results");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir)
                .map_err(|e| AuditError::config_error(&format!("创建临时目录失败: {}", e)))?;
        }
        
        // 获取输入文件名
        let input_name = input_file.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed");
            
        // 生成输出文件名（带时间戳）
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let algorithm_name = match algorithm {
            "FIFO" => "FIFO",
            "BALANCE_METHOD" => "差额计算法",
            _ => algorithm
        };
        
        let filename = format!("{}_{}_{}_{}.xlsx", algorithm_name, input_name, timestamp, rand::random::<u32>() % 10000);
        let output_path = temp_dir.join(filename);
        
        Ok(output_path)
    }
    
    /// Tauri GUI接口: 运行审计分析
    pub async fn run_audit_for_gui(&self, config: TauriAuditConfig) -> TauriAuditResult {
        let start_time = Instant::now();
        
        // 重置状态
        if let Ok(mut status) = self.current_status.lock() {
            *status = TauriProcessStatus::running(0.0, "开始审计分析...".to_string());
        }
        // 注意：不要清空 output_log，因为我们需要保留详细的处理日志供GUI使用
        // 只在真正需要的时候清空
        
        let result = self.analyze_financial_data(
            &config.algorithm,
            &config.input_file,
            config.output_file.as_ref()
        ).await;
        
        match result {
            Ok((summary, transactions, output_file_path)) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                
                // 输出文件路径（现在一定会有实际文件生成）
                let output_files = vec![output_file_path];
                
                // 更新为完成状态
                if let Ok(mut status) = self.current_status.lock() {
                    *status = TauriProcessStatus::idle();
                }
                
                TauriAuditResult::success(
                    summary,
                    transactions.len(),
                    processing_time,
                    config.algorithm,
                    output_files,
                )
            }
            Err(e) => {
                // 更新为错误状态
                if let Ok(mut status) = self.current_status.lock() {
                    *status = TauriProcessStatus::idle();
                }
                
                TauriAuditResult::failure(format!("审计分析失败: {}", e))
            }
        }
    }
    
    /// 获取当前进程状态
    pub fn get_process_status(&self) -> TauriProcessStatus {
        if let Ok(status) = self.current_status.lock() {
            let mut result = status.clone();
            // 添加日志
            if let Ok(log) = self.output_log.lock() {
                result.output_log = log.clone();
            }
            result
        } else {
            TauriProcessStatus::idle()
        }
    }
    
    /// 清空输出日志
    pub fn clear_output_log(&self) {
        if let Ok(mut log) = self.output_log.lock() {
            log.clear();
        }
    }
    
    /// 获取支持的算法列表
    pub fn get_supported_algorithms(&self) -> Vec<&'static str> {
        vec!["FIFO", "BALANCE_METHOD"]
    }
    
    /// 获取当前的输出日志（用于GUI同步）
    pub fn get_output_logs(&self) -> Vec<String> {
        if let Ok(log) = self.output_log.lock() {
            log.clone()
        } else {
            Vec::new()
        }
    }
    
    /// 清空输出日志
    pub fn clear_output_logs(&self) {
        if let Ok(mut log) = self.output_log.lock() {
            log.clear();
        }
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new()
    }
}

/// 交易处理器trait - 统一不同算法的接口
trait TransactionProcessor {
    /// 智能初始化
    fn smart_initialize(&mut self, first_transaction: &Transaction) -> AuditResult<()>;
    
    /// 处理单个交易
    fn process_transaction(&mut self, transaction: &Transaction) -> AuditResult<Transaction>;
    
    /// 获取汇总结果
    fn get_summary(&self) -> AuditResult<AuditSummary>;
}

/// 为FifoTracker实现TransactionProcessor
impl TransactionProcessor for FifoTracker {
    fn smart_initialize(&mut self, first_transaction: &Transaction) -> AuditResult<()> {
        // 基于第一笔交易智能分配初始余额
        let pre_balance = first_transaction.balance - first_transaction.income_amount + first_transaction.expense_amount;
        
        if first_transaction.fund_attribute.contains("个人") {
            self.initialize_balance(pre_balance, "个人")?;
        } else {
            self.initialize_balance(rust_decimal::Decimal::ZERO, "个人")?;
            if pre_balance > rust_decimal::Decimal::ZERO {
                self.process_inflow(pre_balance, "公司初始余额", Some(first_transaction.transaction_date))?;
            }
        }
        
        Ok(())
    }
    
    fn process_transaction(&mut self, transaction: &Transaction) -> AuditResult<Transaction> {
        let mut processed_tx = transaction.clone();
        
        // 根据交易类型调用相应的处理方法
        let result = if transaction.income_amount > rust_decimal::Decimal::ZERO {
            if transaction.fund_attribute.contains('-') {
                self.process_investment_redemption(
                    transaction.income_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            } else {
                self.process_inflow(
                    transaction.income_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            }
        } else if transaction.expense_amount > rust_decimal::Decimal::ZERO {
            if transaction.fund_attribute.contains('-') {
                self.process_investment_purchase(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            } else {
                self.process_outflow(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            }
        } else {
            Ok((rust_decimal::Decimal::ZERO, rust_decimal::Decimal::ZERO, "无变化".to_string()))
        };
        
        // 更新交易字段
        match result {
            Ok((personal_ratio, company_ratio, behavior)) => {
                self.update_transaction_fields(&mut processed_tx, personal_ratio, company_ratio, &behavior)?;
            }
            Err(_) => {
                // 处理失败时保持原始数据
            }
        }
        
        Ok(processed_tx)
    }
    
    fn get_summary(&self) -> AuditResult<AuditSummary> {
        self.get_summary()
    }
}

/// 为BalanceMethodTracker实现TransactionProcessor
impl TransactionProcessor for BalanceMethodTracker {
    fn smart_initialize(&mut self, first_transaction: &Transaction) -> AuditResult<()> {
        // 基于第一笔交易智能分配初始余额
        let pre_balance = first_transaction.balance - first_transaction.income_amount + first_transaction.expense_amount;
        
        if first_transaction.fund_attribute.contains("个人") {
            self.initialize_balance(pre_balance, "个人")?;
        } else {
            self.initialize_balance(rust_decimal::Decimal::ZERO, "个人")?;
            if pre_balance > rust_decimal::Decimal::ZERO {
                self.process_inflow(pre_balance, "公司初始余额", Some(first_transaction.transaction_date))?;
            }
        }
        
        Ok(())
    }
    
    fn process_transaction(&mut self, transaction: &Transaction) -> AuditResult<Transaction> {
        let mut processed_tx = transaction.clone();
        
        // 根据交易类型调用相应的处理方法
        let result = if transaction.income_amount > rust_decimal::Decimal::ZERO {
            if transaction.fund_attribute.contains('-') {
                self.process_investment_redemption(
                    transaction.income_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            } else {
                self.process_inflow(
                    transaction.income_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            }
        } else if transaction.expense_amount > rust_decimal::Decimal::ZERO {
            if transaction.fund_attribute.contains('-') {
                self.process_investment_purchase(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            } else {
                self.process_outflow(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            }
        } else {
            Ok((rust_decimal::Decimal::ZERO, rust_decimal::Decimal::ZERO, "无变化".to_string()))
        };
        
        // 更新交易字段
        match result {
            Ok((personal_ratio, company_ratio, behavior)) => {
                self.update_transaction_fields(&mut processed_tx, personal_ratio, company_ratio, &behavior)?;
            }
            Err(_) => {
                // 处理失败时保持原始数据
            }
        }
        
        Ok(processed_tx)
    }
    
    fn get_summary(&self) -> AuditResult<AuditSummary> {
        self.get_summary()
    }
}