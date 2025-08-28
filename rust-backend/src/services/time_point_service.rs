//! 时点查询服务 - 从mod.rs迁移的完整实现
//! 
//! 提供时点查询和资金池查询功能
//! 基于成功测试代码的实现模式

use crate::utils::{ExcelProcessor, UnifiedValidator};
use crate::algorithms::{FifoTracker, BalanceMethodTracker};
use crate::data_models::{Config, Transaction};
use crate::errors::{AuditError, AuditResult};
use std::time::Instant;
use std::collections::HashMap;
use std::fs;
use log::{info, debug, error};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

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
    // 资金池相关字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_fund_pools: Option<Vec<FundPoolInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_pool_records: Option<std::collections::HashMap<String, Vec<serde_json::Value>>>,
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

// 资金池信息结构
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FundPoolInfo {
    pub id: String,
    pub name: String,
    pub total_balance: rust_decimal::Decimal,
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

/// 文件缓存信息
#[derive(Debug, Clone)]
struct FileCacheData {
    pub fingerprint: String,
    pub processed_transactions: Vec<Transaction>,
    pub audit_summary: crate::data_models::AuditSummary,
    pub offsite_pool_records: crate::data_models::OffsitePoolRecordManager,
    pub algorithm: String,
    pub cached_at: std::time::SystemTime,
}

/// 文件缓存管理器
pub struct FileCache {
    cache: HashMap<String, FileCacheData>,
    max_cache_size: usize,
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size: 10, // 最多缓存10个文件的处理结果
        }
    }
    
    /// 生成文件指纹用于缓存键
    pub fn generate_fingerprint(&self, file_path: &str, algorithm: &str) -> AuditResult<String> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| AuditError::validation_error(&format!("无法读取文件元数据 {}: {}", file_path, e)))?;
            
        let modified_time = metadata.modified()
            .map_err(|e| AuditError::validation_error(&format!("无法获取文件修改时间: {}", e)))?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AuditError::validation_error(&format!("时间转换错误: {}", e)))?
            .as_secs();
            
        let size = metadata.len();
        
        // 组合文件路径、算法、修改时间、大小生成唯一指纹
        Ok(format!("{}|{}|{}|{}", file_path, algorithm, modified_time, size))
    }
    
    /// 检查缓存是否存在且有效
    pub fn has_valid_cache(&self, fingerprint: &str) -> bool {
        if let Some(cache_data) = self.cache.get(fingerprint) {
            // 检查缓存是否过期（1小时过期）
            let cache_age = std::time::SystemTime::now()
                .duration_since(cache_data.cached_at)
                .unwrap_or_default()
                .as_secs();
                
            cache_age < 3600 // 1小时 = 3600秒
        } else {
            false
        }
    }
    
    /// 获取缓存数据
    pub fn get_cache(&self, fingerprint: &str) -> Option<&FileCacheData> {
        self.cache.get(fingerprint)
    }
    
    /// 存储缓存数据
    pub fn set_cache(&mut self, fingerprint: String, cache_data: FileCacheData) {
        // 如果缓存已满，移除最老的条目
        if self.cache.len() >= self.max_cache_size {
            if let Some(oldest_key) = self.cache.keys()
                .min_by_key(|k| self.cache.get(*k).map(|d| d.cached_at).unwrap_or(std::time::UNIX_EPOCH))
                .cloned() 
            {
                self.cache.remove(&oldest_key);
                info!("缓存已满，移除最老缓存: {}", oldest_key);
            }
        }
        
        self.cache.insert(fingerprint.clone(), cache_data);
        info!("文件处理结果已缓存: {}", fingerprint);
    }
    
    /// 清理过期缓存
    pub fn cleanup_expired(&mut self) {
        let now = std::time::SystemTime::now();
        let expired_keys: Vec<String> = self.cache.iter()
            .filter(|(_, data)| {
                now.duration_since(data.cached_at).unwrap_or_default().as_secs() >= 3600
            })
            .map(|(k, _)| k.clone())
            .collect();
            
        for key in expired_keys {
            self.cache.remove(&key);
            info!("清理过期缓存: {}", key);
        }
    }
}

pub struct TimePointService {
    pub algorithm: String,  // 公开算法字段，用于缓存判定
    file_cache: FileCache,
}

impl TimePointService {
    pub fn new(algorithm: String) -> Result<Self, crate::errors::AuditError> {
        Ok(Self { 
            algorithm,
            file_cache: FileCache::new(),
        })
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
                transaction.transaction_date.format("%H:%M:%S")),
        }
    }
    
    /// 轻量级缓存时点查询实现
    /// 优先使用缓存数据，避免重复生成临时文件
    pub async fn query_time_point_cached(&mut self, request: TimePointQueryRequest) -> Result<TimePointQueryResult, crate::errors::AuditError> {
        let start_time = Instant::now();
        info!("开始缓存时点查询: 文件={}, 行号={}, 算法={}", 
              request.file_path, request.row_number, request.algorithm);
        
        // 清理过期缓存
        self.file_cache.cleanup_expired();
        
        // 生成文件指纹
        let fingerprint = self.file_cache.generate_fingerprint(&request.file_path, &request.algorithm)?;
        
        // 获取或创建缓存数据
        let cache_data = if self.file_cache.has_valid_cache(&fingerprint) {
            info!("使用缓存数据进行查询");
            self.file_cache.get_cache(&fingerprint).unwrap().clone()
        } else {
            info!("缓存未命中，执行完整算法处理");
            
            // 使用审计服务完整算法处理流程
            use crate::services::AuditService;
            let audit_service = AuditService::new();
            
            let (audit_summary, processed_transactions, _log_messages) = match audit_service.analyze_financial_data(&request.algorithm, &request.file_path, None::<&String>).await {
                Ok((summary, transactions, log_messages)) => (summary, transactions, log_messages),
                Err(e) => {
                    error!("算法处理失败: {}", e);
                    return Ok(TimePointQueryResult {
                        success: false,
                        algorithm: request.algorithm,
                        target_row: request.row_number,
                        total_rows: 0,
                        processing_time: start_time.elapsed().as_secs_f64(),
                        query_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        message: Some(format!("算法处理失败: {}", e)),
                        target_row_data: None,
                        tracker_state: None,
                        processing_stats: None,
                        recent_steps: None,
                        errors: Some(vec![e.to_string()]),
                        available_fund_pools: Some(vec![]),
                        fund_pool_records: Some(std::collections::HashMap::new()),
                    });
                }
            };
            
            let offsite_pool_records = audit_service.get_offsite_pool_records().clone();
            
            // 创建缓存数据
            let cache_data = FileCacheData {
                fingerprint: fingerprint.clone(),
                processed_transactions: processed_transactions.clone(),
                audit_summary: audit_summary.clone(),
                offsite_pool_records: offsite_pool_records.clone(),
                algorithm: request.algorithm.clone(),
                cached_at: std::time::SystemTime::now(),
            };
            
            // 存储到缓存
            self.file_cache.set_cache(fingerprint, cache_data.clone());
            cache_data
        };
        
        // 使用缓存数据进行时点查询
        self.query_from_cached_data(&request, &cache_data, start_time)
    }
    
    /// 从缓存数据执行时点查询
    fn query_from_cached_data(&self, request: &TimePointQueryRequest, cache_data: &FileCacheData, start_time: Instant) -> Result<TimePointQueryResult, crate::errors::AuditError> {
        let total_rows = cache_data.processed_transactions.len();
        debug!("使用缓存数据，共{}条交易记录", total_rows);
        
        // 验证目标行号
        if request.row_number == 0 || request.row_number > total_rows {
            return Ok(TimePointQueryResult {
                success: false,
                algorithm: request.algorithm.clone(),
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
                available_fund_pools: Some(vec![]),
                fund_pool_records: Some(std::collections::HashMap::new()),
            });
        }
        
        // 基于缓存数据进行时点分析（不生成文件）
        let (tracker_state, target_row_data, recent_steps, fund_pools, fund_records) = 
            self.process_with_cached_data(&cache_data.processed_transactions, request.row_number, &cache_data.audit_summary, &cache_data.offsite_pool_records)?;
        
        let total_time = start_time.elapsed().as_secs_f64();
        info!("缓存时点查询完成，总耗时{:.3}秒", total_time);
        
        // 返回查询结果
        Ok(TimePointQueryResult {
            success: true,
            algorithm: request.algorithm.clone(),
            target_row: request.row_number,
            total_rows,
            processing_time: total_time,
            query_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            message: Some(format!("时点查询成功（使用缓存），处理到第{}行", request.row_number)),
            target_row_data: Some(target_row_data),
            tracker_state: Some(tracker_state),
            processing_stats: Some(ProcessingStats {
                last_processed_row: request.row_number,
                total_steps: recent_steps.len(),
                error_count: 0,
                total_validation_time: 0.0, // 使用缓存，无验证时间
                algorithm_processing_time: total_time,
            }),
            recent_steps: Some(recent_steps),
            errors: None,
            available_fund_pools: Some(fund_pools),
            fund_pool_records: Some(fund_records),
        })
    }

    /// 完整的时点查询实现（保留原有方法作为备用）
    /// 使用审计服务的完整算法处理流程，确保获取准确的分析数据
    pub async fn query_time_point(&mut self, request: TimePointQueryRequest) -> Result<TimePointQueryResult, crate::errors::AuditError> {
        let start_time = Instant::now();
        info!("开始时点查询: 文件={}, 行号={}, 算法={}", 
              request.file_path, request.row_number, request.algorithm);
        
        // 使用审计服务完整算法处理流程
        use crate::services::AuditService;
        let audit_service = AuditService::new();
        
        // 执行完整的算法分析，获取经过算法处理的交易数据
        let (summary, processed_transactions, _log_messages) = match audit_service.analyze_financial_data(&request.algorithm, &request.file_path, None::<&String>).await {
            Ok((summary, transactions, log_messages)) => (summary, transactions, log_messages),
            Err(e) => {
                error!("算法处理失败: {}", e);
                return Ok(TimePointQueryResult {
                    success: false,
                    algorithm: request.algorithm,
                    target_row: request.row_number,
                    total_rows: 0,
                    processing_time: start_time.elapsed().as_secs_f64(),
                    query_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    message: Some(format!("算法处理失败: {}", e)),
                    target_row_data: None,
                    tracker_state: None,
                    processing_stats: None,
                    recent_steps: None,
                    errors: Some(vec![e.to_string()]),
                    available_fund_pools: Some(vec![]),
                    fund_pool_records: Some(std::collections::HashMap::new()),
                });
            }
        };
        
        // 获取场外资金池记录管理器
        let offsite_pool_records = audit_service.get_offsite_pool_records();
        
        let total_rows = processed_transactions.len();
        debug!("通过算法处理获得{}条交易记录", total_rows);
        
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
                available_fund_pools: Some(vec![]),
                fund_pool_records: Some(std::collections::HashMap::new()),
            });
        }
        
        // 数据已经通过审计服务完成了验证和算法处理，无需再次验证
        debug!("使用审计服务处理后的数据，包含完整的算法计算结果");
        
        // 第三步：基于算法处理后的数据进行时点查询分析
        let algorithm_start = Instant::now();
        let (tracker_state, target_row_data, recent_steps, fund_pools, fund_records) = match request.algorithm.to_uppercase().as_str() {
            "FIFO" => {
                self.process_with_processed_data(&processed_transactions, request.row_number, &summary, &offsite_pool_records)?
            },
            "BALANCE_METHOD" => {
                self.process_with_processed_data(&processed_transactions, request.row_number, &summary, &offsite_pool_records)?
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
                    available_fund_pools: Some(vec![]),
                    fund_pool_records: Some(std::collections::HashMap::new()),
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
                total_validation_time: 0.0, // 使用审计服务，无需单独验证时间
                algorithm_processing_time: algorithm_time,
            }),
            recent_steps: Some(recent_steps),
            errors: None,
            available_fund_pools: Some(fund_pools),
            fund_pool_records: Some(fund_records),
        })
    }
    
    /// 使用缓存数据进行时点查询分析（不生成临时文件）
    fn process_with_cached_data(&self, processed_transactions: &[Transaction], target_row: usize, summary: &crate::data_models::AuditSummary, offsite_pool_records: &crate::data_models::OffsitePoolRecordManager) -> AuditResult<(TrackerStateSnapshot, FrontendTransaction, Vec<TransactionStep>, Vec<FundPoolInfo>, std::collections::HashMap<String, Vec<serde_json::Value>>)> {
        // 复用现有逻辑，但标注为缓存处理模式
        debug!("使用缓存数据处理时点查询，不生成临时文件");
        self.process_with_processed_data(processed_transactions, target_row, summary, offsite_pool_records)
    }

    /// 使用算法处理后的数据进行时点查询分析
    fn process_with_processed_data(&self, processed_transactions: &[Transaction], target_row: usize, summary: &crate::data_models::AuditSummary, offsite_pool_records: &crate::data_models::OffsitePoolRecordManager) -> AuditResult<(TrackerStateSnapshot, FrontendTransaction, Vec<TransactionStep>, Vec<FundPoolInfo>, std::collections::HashMap<String, Vec<serde_json::Value>>)> {
        let mut recent_steps = Vec::new();
        let mut fund_pools = Vec::new();
        let mut fund_records = std::collections::HashMap::new();
        
        // 获取目标时点的交易时间，作为截止时间
        let target_transaction = processed_transactions.get(target_row - 1)
            .ok_or_else(|| AuditError::validation_error("目标行不存在"))?;
        let cutoff_date = target_transaction.transaction_date.date();
        
        // 从场外资金池记录管理器中提取截止到指定时点的资金池信息
        let pool_groups = offsite_pool_records.group_by_pool();
        
        // 构建资金池信息列表 - 只包含截止到指定时点的数据
        for (pool_name, pool_records_list) in pool_groups.iter() {
            // 只处理截止到目标时点的记录
            let cutoff_records: Vec<_> = pool_records_list.iter()
                .filter(|record| {
                    // 解析场外资金池记录的时间字符串并与目标时点比较
                    if let Ok(record_datetime) = chrono::NaiveDateTime::parse_from_str(&record.transaction_time, "%Y-%m-%d %H:%M:%S") {
                        record_datetime.date() <= cutoff_date
                    } else {
                        // 如果解析失败，包含该记录（保守处理）
                        true
                    }
                })
                .collect();
                
            if let Some(latest_record) = cutoff_records.last() {
                fund_pools.push(FundPoolInfo {
                    id: format!("pool_{}", pool_name.replace(" ", "_")),
                    name: pool_name.clone(),
                    total_balance: latest_record.total_balance,
                });
                
                // 将截止到时点的场外资金池记录转换为前端需要的格式
                let mut records = Vec::new();
                for record in cutoff_records {
                    // 计算个人和公司的百分比
                    let total_balance = record.personal_balance + record.company_balance;
                    let (personal_percent, company_percent) = if total_balance.abs() > rust_decimal::Decimal::new(1, 2) { // > 0.01
                        let personal_pct = if total_balance != rust_decimal::Decimal::ZERO {
                            (record.personal_balance / total_balance * rust_decimal::Decimal::from(100)).round_dp(1)
                        } else {
                            rust_decimal::Decimal::ZERO
                        };
                        let company_pct = if total_balance != rust_decimal::Decimal::ZERO {
                            (record.company_balance / total_balance * rust_decimal::Decimal::from(100)).round_dp(1)
                        } else {
                            rust_decimal::Decimal::ZERO
                        };
                        (personal_pct, company_pct)
                    } else {
                        (rust_decimal::Decimal::ZERO, rust_decimal::Decimal::ZERO)
                    };
                    
                    let record_json = serde_json::json!({
                        "交易时间": record.transaction_time,
                        "资金池名称": record.pool_name,
                        "入金": record.inflow,
                        "出金": record.outflow,
                        "总余额": record.total_balance,
                        "个人余额": format!("{:.2} ({}%)", record.personal_balance, personal_percent),
                        "公司余额": format!("{:.2} ({}%)", record.company_balance, company_percent),
                        "行为性质": record.behavior_nature,
                        "累计申购": record.cumulative_purchase,
                        "累计赎回": record.cumulative_redemption,
                        "净盈亏": record.net_profit_loss,
                    });
                    records.push(record_json);
                }
                fund_records.insert(pool_name.clone(), records);
            }
        }
        
        // 处理交易记录（用于生成最近步骤信息）
        for (index, transaction) in processed_transactions.iter().enumerate() {
            if index + 1 > target_row {
                break;
            }
            
            // 获取余额状态（使用交易前一条的余额）
            let balance_before = if index > 0 {
                processed_transactions[index - 1].balance
            } else {
                Decimal::ZERO // 第一条交易的初始余额
            };
            
            // 使用算法计算的真实行为性质而不是重新分析
            let behavior = transaction.behavior_nature.clone().unwrap_or_else(|| {
                if transaction.income_amount > Decimal::ZERO {
                    format!("资金流入：{:.2}", transaction.income_amount)
                } else if transaction.expense_amount > Decimal::ZERO {
                    format!("资金流出：{:.2}", transaction.expense_amount)
                } else {
                    "无变动".to_string()
                }
            });
            
            let balance_after = transaction.balance;
            
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
        
        // 获取目标行的交易数据（已经过算法处理，包含准确的比例和行为数据）
        let target_transaction_raw = processed_transactions.get(target_row - 1)
            .ok_or_else(|| AuditError::validation_error("目标行不存在"))?.clone();
        
        let target_transaction = self.convert_to_frontend_transaction(&target_transaction_raw);
        
        // 获取目标交易时的状态快照（使用算法计算的真实数据）
        let tracker_state = TrackerStateSnapshot {
            current_balance: target_transaction_raw.balance,
            personal_balance: target_transaction_raw.personal_balance.unwrap_or(Decimal::ZERO),
            company_balance: target_transaction_raw.company_balance.unwrap_or(Decimal::ZERO),
            total_personal_in: summary.total_personal_profit, // 使用审计摘要的真实数据
            total_company_in: summary.total_company_profit,
            total_personal_out: summary.total_misappropriation,
            total_company_out: summary.total_advance_payment,
            misappropriation_amount: target_transaction_raw.cumulative_misappropriation.unwrap_or(Decimal::ZERO),
            advance_amount: target_transaction_raw.cumulative_advance.unwrap_or(Decimal::ZERO),
        };
        
        Ok((tracker_state, target_transaction, recent_steps, fund_pools, fund_records))
    }
    
    /// 从行为描述中提取资金池名称
    fn extract_pool_name_from_behavior(&self, behavior: &str) -> Option<String> {
        // 简单正则提取 - 格式类似 "产品名称申购-XX："或"产品名称赎回-XX："
        if let Some(start) = behavior.find("申购-").or_else(|| behavior.find("赎回-")) {
            let pool_part = &behavior[..start];
            if !pool_part.is_empty() {
                return Some(pool_part.to_string());
            }
        }
        
        // 备用方案：提取冒号前的内容
        if let Some(colon_pos) = behavior.find('：') {
            let before_colon = &behavior[..colon_pos];
            if let Some(dash_pos) = before_colon.rfind('-') {
                let pool_name = &before_colon[..dash_pos];
                if !pool_name.is_empty() {
                    return Some(pool_name.to_string());
                }
            }
        }
        
        None
    }
    
    /// 从算法追踪器中提取资金池数据
    /// 这是正确的架构：复用算法追踪器中收集的资金池记录，而不是手动提取
    fn extract_fund_pools_from_tracker(&self, tracker_base: &crate::algorithms::shared::TrackerBase) -> (Vec<FundPoolInfo>, std::collections::HashMap<String, Vec<serde_json::Value>>) {
        let mut fund_pools = Vec::new();
        let mut fund_records = std::collections::HashMap::new();
        
        // 从OffsitePoolRecordManager中提取资金池数据
        let pool_records = &tracker_base.offsite_pool_records.records;
        
        // 按资金池名称分组
        let mut pools_map = std::collections::HashMap::new();
        for record in pool_records {
            pools_map.entry(record.pool_name.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        // 构建资金池信息和记录
        for (pool_name, records) in pools_map {
            // 计算资金池总余额（最后一条记录的余额）
            let total_balance = records.last()
                .map(|r| r.total_balance)
                .unwrap_or(rust_decimal::Decimal::ZERO);
                
            // 添加资金池信息
            fund_pools.push(FundPoolInfo {
                id: format!("pool_{}", pool_name.replace(" ", "_")),
                name: pool_name.clone(),
                total_balance,
            });
            
            // 转换记录为前端格式
            let json_records: Vec<serde_json::Value> = records.into_iter()
                .map(|record| serde_json::json!({
                    "交易时间": record.transaction_time,
                    "资金池名称": record.pool_name,
                    "入金": record.inflow,
                    "出金": record.outflow,
                    "总余额": record.total_balance,
                    "个人余额": record.personal_balance,
                    "公司余额": record.company_balance,
                    "资金占比": record.fund_ratio,
                    "行为性质": record.behavior_nature,
                    "累计申购": record.cumulative_purchase,
                    "累计赎回": record.cumulative_redemption,
                    "净盈亏": record.net_profit_loss,
                }))
                .collect();
                
            fund_records.insert(pool_name, json_records);
        }
        
        (fund_pools, fund_records)
    }
    
    pub async fn query_fund_pool(&mut self, request: FundPoolQueryRequest) -> Result<FundPoolQueryResult, crate::errors::AuditError> {
        Ok(FundPoolQueryResult {
            success: false,
            pool_name: request.pool_name,
            message: Some("资金池查询功能开发中".to_string()),
        })
    }
}