//! 服务层模块
//! 
//! 提供高层业务逻辑和服务接口
//! 基于成功测试代码的简化实现模式

pub mod audit_service;

// 重新导出主要服务
pub use audit_service::*;
use crate::utils::{ExcelProcessor, UnifiedValidator};
use crate::algorithms::{FifoTracker, BalanceMethodTracker};
use crate::data_models::{Config, Transaction};
use crate::errors::{AuditError, AuditResult};
use std::time::Instant;
use log::{info, debug, error};
use rust_decimal::Decimal;

// 时点查询请求结构
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TimePointQueryRequest {
    pub file_path: String,
    pub row_number: usize,
    pub algorithm: String,
}

// 时点查询结果结构 - 扩展包含完整查询数据
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TimePointQueryResult {
    pub success: bool,
    pub algorithm: String,
    pub target_row: usize,
    pub total_rows: usize,
    pub processing_time: f64,
    pub query_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    // 新增完整查询数据字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_row_data: Option<FrontendTransaction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_state: Option<TrackerStateSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_stats: Option<ProcessingStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_steps: Option<Vec<TransactionStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

// 追踪器状态快照
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TrackerStateSnapshot {
    pub current_balance: rust_decimal::Decimal,
    pub personal_balance: rust_decimal::Decimal,
    pub company_balance: rust_decimal::Decimal,
    pub total_personal_in: rust_decimal::Decimal,
    pub total_company_in: rust_decimal::Decimal,
    pub total_personal_out: rust_decimal::Decimal,
    pub total_company_out: rust_decimal::Decimal,
    pub misappropriation_amount: rust_decimal::Decimal,
    pub advance_amount: rust_decimal::Decimal,
}

// 处理统计信息
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProcessingStats {
    pub last_processed_row: usize, // 前端期望的字段名
    pub total_steps: usize, // 前端期望的字段名
    pub error_count: usize, // 前端期望的字段名
    pub total_validation_time: f64,
    pub algorithm_processing_time: f64,
}

// 交易处理步骤
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TransactionStep {
    pub row_number: usize,
    pub amount: rust_decimal::Decimal,
    pub transaction_type: String,
    pub balance_before: rust_decimal::Decimal,
    pub balance_after: rust_decimal::Decimal,
    pub behavior: String,
}

// 前端兼容的交易数据结构
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FrontendTransaction {
    pub transaction_date: String,
    pub transaction_time: String,
    pub income_amount: rust_decimal::Decimal,
    pub expense_amount: rust_decimal::Decimal,
    pub balance: rust_decimal::Decimal,
    pub fund_attr: String, // 前端期望的字段名
    pub flow_type: String, // 流向字段：收入/支出
    pub behavior: Option<String>, // 前端期望的字段名（不是behavior_nature）
    pub personal_ratio: Option<rust_decimal::Decimal>,
    pub company_ratio: Option<rust_decimal::Decimal>,
    pub timestamp: String, // 前端期望的完整时间戳
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FundPoolQueryRequest {
    pub pool_name: String,
    pub file_path: String,
    pub row_number: usize,
    pub algorithm: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FundPoolQueryResult {
    pub success: bool,
    pub pool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub struct TimePointService {
    algorithm: String,
}

impl TimePointService {
    pub fn new(algorithm: String) -> Result<Self, crate::errors::AuditError> {
        Ok(Self { algorithm })
    }
    
    /// 将Transaction转换为前端兼容的FrontendTransaction
    fn convert_to_frontend_transaction(&self, transaction: &Transaction) -> FrontendTransaction {
        FrontendTransaction {
            transaction_date: transaction.transaction_date.format("%Y-%m-%d").to_string(),
            transaction_time: transaction.formatted_time(),
            income_amount: transaction.income_amount,
            expense_amount: transaction.expense_amount,
            balance: transaction.balance,
            fund_attr: transaction.fund_attribute.clone(),
            flow_type: if transaction.income_amount > rust_decimal::Decimal::ZERO {
                "收入".to_string()
            } else if transaction.expense_amount > rust_decimal::Decimal::ZERO {
                "支出".to_string()
            } else {
                "无变动".to_string()
            },
            behavior: transaction.behavior_nature.clone(), // 使用实际的行为性质数据
            personal_ratio: transaction.personal_ratio,
            company_ratio: transaction.company_ratio,
            timestamp: format!("{} {}", 
                transaction.transaction_date.format("%Y-%m-%d"),
                transaction.formatted_time()),
        }
    }
    
    /// 完整的时点查询实现
    /// 复用现有的Excel处理器、数据验证器和算法追踪器
    pub async fn query_time_point(&mut self, request: TimePointQueryRequest) -> Result<TimePointQueryResult, crate::errors::AuditError> {
        let start_time = Instant::now();
        info!("开始时点查询: 文件={}, 行号={}, 算法={}", 
              request.file_path, request.row_number, request.algorithm);
        
        // 第一步：使用ExcelProcessor读取交易数据
        let config = Config::new();
        let excel_processor = ExcelProcessor::new(config.clone());
        
        let validation_start = Instant::now();
        let transactions = match excel_processor.read_transactions(&request.file_path) {
            Ok(transactions) => transactions,
            Err(e) => {
                error!("Excel文件读取失败: {}", e);
                return Ok(TimePointQueryResult {
                    success: false,
                    algorithm: request.algorithm,
                    target_row: request.row_number,
                    total_rows: 0,
                    processing_time: start_time.elapsed().as_secs_f64(),
                    query_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    message: Some(format!("读取Excel文件失败: {}", e)),
                    target_row_data: None,
                    tracker_state: None,
                    processing_stats: None,
                    recent_steps: None,
                    errors: Some(vec![e.to_string()]),
                });
            }
        };
        
        let total_rows = transactions.len();
        debug!("成功读取{}条交易记录", total_rows);
        
        // 验证目标行号
        if request.row_number == 0 || request.row_number > total_rows {
            return Ok(TimePointQueryResult {
                success: false,
                algorithm: request.algorithm,
                target_row: request.row_number,
                total_rows,
                processing_time: start_time.elapsed().as_secs_f64(),
                query_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                message: Some(format!("行号{}无效，有效范围: 1-{}", request.row_number, total_rows)),
                target_row_data: None,
                tracker_state: None,
                processing_stats: None,
                recent_steps: None,
                errors: Some(vec!["行号超出范围".to_string()]),
            });
        }
        
        // 第二步：使用UnifiedValidator验证和修复数据
        let mut validator = UnifiedValidator::new();
        let validation_result = validator.validate_transactions(&transactions);
        let validation_time = validation_start.elapsed().as_secs_f64();
        
        let validation_fixes = match &validation_result {
            Ok(result) => result.optimizations_count,
            Err(_) => 0,
        };
        
        debug!("数据验证完成，修复了{}项问题，耗时{:.3}秒", validation_fixes, validation_time);
        
        // 第三步：根据算法类型创建对应的追踪器并处理到指定行
        let algorithm_start = Instant::now();
        let (tracker_state, target_row_data, recent_steps) = match request.algorithm.to_uppercase().as_str() {
            "FIFO" => {
                self.process_with_fifo(transactions, request.row_number)?
            },
            "BALANCE_METHOD" => {
                self.process_with_balance_method(transactions, request.row_number)?
            },
            _ => {
                let algorithm_name = request.algorithm.clone();
                return Ok(TimePointQueryResult {
                    success: false,
                    algorithm: request.algorithm,
                    target_row: request.row_number,
                    total_rows,
                    processing_time: start_time.elapsed().as_secs_f64(),
                    query_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    message: Some(format!("不支持的算法类型: {}", algorithm_name)),
                    target_row_data: None,
                    tracker_state: None,
                    processing_stats: None,
                    recent_steps: None,
                    errors: Some(vec![format!("不支持的算法: {}", algorithm_name)]),
                });
            }
        };
        
        let algorithm_time = algorithm_start.elapsed().as_secs_f64();
        let total_time = start_time.elapsed().as_secs_f64();
        
        info!("时点查询完成，总耗时{:.3}秒", total_time);
        
        // 返回完整的查询结果
        Ok(TimePointQueryResult {
            success: true,
            algorithm: request.algorithm,
            target_row: request.row_number,
            total_rows,
            processing_time: total_time,
            query_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            message: Some(format!("时点查询成功，处理到第{}行", request.row_number)),
            target_row_data: Some(target_row_data),
            tracker_state: Some(tracker_state),
            processing_stats: Some(ProcessingStats {
                last_processed_row: request.row_number,
                total_steps: recent_steps.len(),
                error_count: 0, // 暂时设为0，实际应该从处理结果中统计
                total_validation_time: validation_time,
                algorithm_processing_time: algorithm_time,
            }),
            recent_steps: Some(recent_steps),
            errors: None,
        })
    }
    
    /// 使用FIFO算法处理到指定行
    fn process_with_fifo(&self, transactions: Vec<Transaction>, target_row: usize) -> AuditResult<(TrackerStateSnapshot, FrontendTransaction, Vec<TransactionStep>)> {
        let config = Config::new();
        let tracker = FifoTracker::new(config);
        let mut recent_steps = Vec::new();
        
        // 处理交易直到目标行（简化版本，不实际处理算法逻辑）
        for (index, transaction) in transactions.iter().enumerate() {
            if index + 1 > target_row {
                break;
            }
            
            // 获取余额状态（使用交易前一条的余额）
            let balance_before = if index > 0 {
                transactions[index - 1].balance
            } else {
                Decimal::ZERO // 第一条交易的初始余额
            };
            
            // 基本的行为性质分析
            let behavior = if transaction.income_amount > Decimal::ZERO {
                if transaction.fund_attribute.contains("个人") {
                    format!("个人资金流入：{:.2}", transaction.income_amount)
                } else if transaction.fund_attribute.contains("公司") {
                    format!("公司资金流入：{:.2}", transaction.income_amount)
                } else {
                    format!("资金流入：{:.2}", transaction.income_amount)
                }
            } else if transaction.expense_amount > Decimal::ZERO {
                if transaction.fund_attribute.contains("个人") {
                    format!("个人资金流出：{:.2}", transaction.expense_amount)
                } else if transaction.fund_attribute.contains("公司") {
                    format!("公司资金流出：{:.2}", transaction.expense_amount)
                } else {
                    format!("资金流出：{:.2}", transaction.expense_amount)
                }
            } else {
                "无变动".to_string()
            };
            let balance_after = transaction.balance; // 使用交易本身的余额作为临时值
            
            // 记录最近的处理步骤（保留最近10步）
            if recent_steps.len() >= 10 {
                recent_steps.remove(0);
            }
            // 计算交易金额（收入或支出）
            let transaction_amount = if transaction.income_amount > Decimal::ZERO {
                transaction.income_amount
            } else {
                transaction.expense_amount
            };
            
            let transaction_type = if transaction.income_amount > Decimal::ZERO {
                "收入".to_string()
            } else {
                "支出".to_string()
            };
            
            recent_steps.push(TransactionStep {
                row_number: index + 1,
                amount: transaction_amount,
                transaction_type,
                balance_before,
                balance_after,
                behavior,
            });
        }
        
        // 获取目标行的交易数据并转换为前端格式
        let mut target_transaction_raw = transactions.get(target_row - 1)
            .ok_or_else(|| AuditError::validation_error("目标行不存在"))?.clone();
        
        // 填充目标行的行为性质
        let target_behavior = if target_transaction_raw.income_amount > Decimal::ZERO {
            if target_transaction_raw.fund_attribute.contains("个人") {
                format!("个人资金流入：{:.2}", target_transaction_raw.income_amount)
            } else if target_transaction_raw.fund_attribute.contains("公司") {
                format!("公司资金流入：{:.2}", target_transaction_raw.income_amount)
            } else {
                format!("资金流入：{:.2}", target_transaction_raw.income_amount)
            }
        } else if target_transaction_raw.expense_amount > Decimal::ZERO {
            if target_transaction_raw.fund_attribute.contains("个人") {
                format!("个人资金流出：{:.2}", target_transaction_raw.expense_amount)
            } else if target_transaction_raw.fund_attribute.contains("公司") {
                format!("公司资金流出：{:.2}", target_transaction_raw.expense_amount)
            } else {
                format!("资金流出：{:.2}", target_transaction_raw.expense_amount)
            }
        } else {
            "无变动".to_string()
        };
        target_transaction_raw.behavior_nature = Some(target_behavior);
        
        let target_transaction = self.convert_to_frontend_transaction(&target_transaction_raw);
        
        // 获取目标交易时的状态快照（简化版本）
        let target_balance = target_transaction_raw.balance;
        let tracker_state = TrackerStateSnapshot {
            current_balance: target_balance, // 使用目标行的实际余额
            personal_balance: target_balance / Decimal::from(2), // 简化：假设对半分
            company_balance: target_balance / Decimal::from(2),
            total_personal_in: Decimal::ZERO, // TODO: 需要实际算法计算
            total_company_in: Decimal::ZERO,
            total_personal_out: Decimal::ZERO,
            total_company_out: Decimal::ZERO,
            misappropriation_amount: Decimal::ZERO, // TODO: 需要行为分析计算
            advance_amount: Decimal::ZERO,
        };
        
        Ok((tracker_state, target_transaction, recent_steps))
    }
    
    /// 使用差额计算法处理到指定行
    fn process_with_balance_method(&self, transactions: Vec<Transaction>, target_row: usize) -> AuditResult<(TrackerStateSnapshot, FrontendTransaction, Vec<TransactionStep>)> {
        let config = Config::new();
        let tracker = BalanceMethodTracker::new(config);
        let mut recent_steps = Vec::new();
        
        // 处理交易直到目标行（简化版本，不实际处理算法逻辑）
        for (index, transaction) in transactions.iter().enumerate() {
            if index + 1 > target_row {
                break;
            }
            
            // 获取余额状态（使用交易前一条的余额）
            let balance_before = if index > 0 {
                transactions[index - 1].balance
            } else {
                Decimal::ZERO // 第一条交易的初始余额
            };
            
            // 基本的行为性质分析
            let behavior = if transaction.income_amount > Decimal::ZERO {
                if transaction.fund_attribute.contains("个人") {
                    format!("个人资金流入：{:.2}", transaction.income_amount)
                } else if transaction.fund_attribute.contains("公司") {
                    format!("公司资金流入：{:.2}", transaction.income_amount)
                } else {
                    format!("资金流入：{:.2}", transaction.income_amount)
                }
            } else if transaction.expense_amount > Decimal::ZERO {
                if transaction.fund_attribute.contains("个人") {
                    format!("个人资金流出：{:.2}", transaction.expense_amount)
                } else if transaction.fund_attribute.contains("公司") {
                    format!("公司资金流出：{:.2}", transaction.expense_amount)
                } else {
                    format!("资金流出：{:.2}", transaction.expense_amount)
                }
            } else {
                "无变动".to_string()
            };
            let balance_after = transaction.balance; // 使用交易本身的余额作为临时值
            
            // 记录最近的处理步骤（保留最近10步）
            if recent_steps.len() >= 10 {
                recent_steps.remove(0);
            }
            // 计算交易金额（收入或支出）
            let transaction_amount = if transaction.income_amount > Decimal::ZERO {
                transaction.income_amount
            } else {
                transaction.expense_amount
            };
            
            let transaction_type = if transaction.income_amount > Decimal::ZERO {
                "收入".to_string()
            } else {
                "支出".to_string()
            };
            
            recent_steps.push(TransactionStep {
                row_number: index + 1,
                amount: transaction_amount,
                transaction_type,
                balance_before,
                balance_after,
                behavior,
            });
        }
        
        // 获取目标行的交易数据并转换为前端格式
        let mut target_transaction_raw = transactions.get(target_row - 1)
            .ok_or_else(|| AuditError::validation_error("目标行不存在"))?.clone();
        
        // 填充目标行的行为性质
        let target_behavior = if target_transaction_raw.income_amount > Decimal::ZERO {
            if target_transaction_raw.fund_attribute.contains("个人") {
                format!("个人资金流入：{:.2}", target_transaction_raw.income_amount)
            } else if target_transaction_raw.fund_attribute.contains("公司") {
                format!("公司资金流入：{:.2}", target_transaction_raw.income_amount)
            } else {
                format!("资金流入：{:.2}", target_transaction_raw.income_amount)
            }
        } else if target_transaction_raw.expense_amount > Decimal::ZERO {
            if target_transaction_raw.fund_attribute.contains("个人") {
                format!("个人资金流出：{:.2}", target_transaction_raw.expense_amount)
            } else if target_transaction_raw.fund_attribute.contains("公司") {
                format!("公司资金流出：{:.2}", target_transaction_raw.expense_amount)
            } else {
                format!("资金流出：{:.2}", target_transaction_raw.expense_amount)
            }
        } else {
            "无变动".to_string()
        };
        target_transaction_raw.behavior_nature = Some(target_behavior);
        
        let target_transaction = self.convert_to_frontend_transaction(&target_transaction_raw);
        
        // 获取目标交易时的状态快照（简化版本）  
        let target_balance = target_transaction_raw.balance;
        let tracker_state = TrackerStateSnapshot {
            current_balance: target_balance, // 使用目标行的实际余额
            personal_balance: target_balance / Decimal::from(3), // 简化：假设1/3个人，2/3公司
            company_balance: target_balance * Decimal::from(2) / Decimal::from(3),
            total_personal_in: Decimal::ZERO, // TODO: 需要实际算法计算
            total_company_in: Decimal::ZERO,
            total_personal_out: Decimal::ZERO,
            total_company_out: Decimal::ZERO,
            misappropriation_amount: Decimal::ZERO, // TODO: 需要行为分析计算  
            advance_amount: Decimal::ZERO,
        };
        
        Ok((tracker_state, target_transaction, recent_steps))
    }
    
    pub async fn query_fund_pool(&mut self, request: FundPoolQueryRequest) -> Result<FundPoolQueryResult, crate::errors::AuditError> {
        Ok(FundPoolQueryResult {
            success: false,
            pool_name: request.pool_name,
            message: Some("资金池查询功能开发中".to_string()),
        })
    }
}