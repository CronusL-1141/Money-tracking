//! 时点查询相关的Tauri命令
//! 
//! 使用Rust后端直接实现，不再依赖Python

use tauri::{command, State};
use audit_backend::{TimePointService, TimePointQueryRequest, TimePointQueryResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{AppState, generate_id, QueryHistory, TimePointQuery, QueryResult};

use chrono::Utc;
use log::{info, error, warn};

/// Tauri命令：清除缓存状态（当用户选择新文件时调用）
#[command]
pub async fn clear_query_cache(state: State<'_, AppState>) -> Result<(), String> {
    let mut last_query = state.last_full_query.lock().await;
    if last_query.is_some() {
        info!("清除查询缓存状态");
        *last_query = None;
    }
    Ok(())
}

/// Tauri命令：时点查询（新的Rust原生实现）
#[command]
pub async fn time_point_query_rust(
    query: TimePointQuery,
    state: State<'_, AppState>
) -> Result<QueryResult, String> {
    info!("Time point query: file={}, row={}, algorithm={}", query.file_path, query.row_number, query.algorithm);
    
    // 构建Rust后端请求
    let request = TimePointQueryRequest {
        file_path: query.file_path.clone(),
        row_number: query.row_number as usize,
        algorithm: query.algorithm.clone(),
    };
    
    // 更新进程状态日志
    {
        let mut process_status = state.current_process.lock().await;
        process_status.output_log.push(format!("[{}] ===== 开始时点查询 (Rust原生) =====", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));
        process_status.output_log.push(format!("[{}] 🔍 执行时点查询: 第{}行", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), query.row_number
        ));
        process_status.output_log.push(format!("[{}] 📁 文件: {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), 
            query.file_path.split(&['/', '\\'][..]).last().unwrap_or(&query.file_path)
        ));
        process_status.output_log.push(format!("[{}] 🔧 算法: {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), 
            match query.algorithm.as_str() {
                "FIFO" => "FIFO先进先出算法",
                "BALANCE_METHOD" => "差额计算法",
                _ => &query.algorithm
            }
        ));
    }
    
    // 获取或创建时点查询服务（支持缓存）
    {
        let mut service_guard = state.time_point_service.lock().await;
        
        // 检查是否需要创建新服务实例（算法变更时）
        let service_needs_update = match service_guard.as_ref() {
            Some(existing_service) => existing_service.algorithm != query.algorithm,
            None => true, // 首次创建
        };
        
        if service_needs_update {
            match TimePointService::new(query.algorithm.clone()) {
                Ok(new_service) => {
                    *service_guard = Some(new_service);
                    info!("时点查询服务已更新，算法: {}", query.algorithm);
                },
                Err(e) => {
                    error!("Failed to create TimePointService: {}", e);
                    let mut process_status = state.current_process.lock().await;
                    process_status.output_log.push(format!("[{}] ❌ 服务初始化失败: {}", 
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), e
                    ));
                    return Ok(QueryResult {
                        success: false,
                        data: None,
                        message: format!("服务初始化失败: {}", e),
                    });
                }
            }
        }
    } // service_guard 在这里被释放
    
    // 统一使用缓存机制：所有查询都走缓存路径，让后端的文件指纹机制决定是否命中缓存
    println!("🔍 缓存策略: 统一使用缓存路径，由后端文件指纹机制决定缓存命中");
    
    println!("🚀 尝试使用智能缓存优化");
    let mut process_status = state.current_process.lock().await;
    process_status.output_log.push(format!("[{}] 🚀 智能缓存检测，优化查询速度", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    ));
    drop(process_status);
    
    // 获取服务实例进行缓存查询
    let cached_result = {
        let mut service_guard = state.time_point_service.lock().await;
        let service = service_guard.as_mut().unwrap();
        service.query_time_point_cached(request.clone()).await
    };
    
    let rust_result = match cached_result {
        Ok(result) => result,
        Err(_) => {
            // 缓存失败，回退到完整处理
            info!("缓存查询失败，回退到完整处理");
            let mut process_status = state.current_process.lock().await;
            process_status.output_log.push(format!("[{}] ⚡ 缓存未命中，执行完整算法处理", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            ));
            drop(process_status);
            
            // 获取服务实例进行完整查询
            let full_result = {
                let mut service_guard = state.time_point_service.lock().await;
                let service = service_guard.as_mut().unwrap();
                service.query_time_point(request).await
            };
            
            match full_result {
                Ok(result) => {
                    // 缓存回退查询成功（后端文件指纹缓存已自动管理）
                    if result.success {
                        println!("💾 缓存回退查询成功，文件指纹缓存已自动更新");
                    }
                    result
                },
                Err(e) => {
                    error!("完整查询也失败: {}", e);
                    return Ok(QueryResult {
                        success: false,
                        data: None,
                        message: format!("查询失败: {}", e),
                    });
                }
            }
        }
    };
    
    // 转换结果格式
    if rust_result.success {
        let mut process_status = state.current_process.lock().await;
        process_status.output_log.push(format!("[{}] ✅ 查询完成: 处理时间 {:.3}s", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), rust_result.processing_time
        ));
        process_status.output_log.push(format!("[{}] 📊 数据: 第{}/{}行", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), rust_result.target_row, rust_result.total_rows
        ));
        
        // 添加到查询历史
        let history_entry = QueryHistory {
            id: generate_id(),
            timestamp: Utc::now(),
            file_path: query.file_path.clone(),
            row_number: query.row_number,
            algorithm: query.algorithm.clone(),
            result: Some("查询成功 (Rust原生)".to_string()),
        };
        
        let mut history = state.query_history.lock().await;
        history.push(history_entry);
        
        // 保持历史记录数量限制
        let config = state.app_config.lock().await;
        let max_history = config.max_history;
        drop(config);
        
        if history.len() > max_history {
            let len = history.len();
            history.drain(0..len - max_history);
        }
        
        info!("Time point query completed successfully");
        println!("Rust后端查询结果: {:?}", rust_result); // 调试信息
        
        // 转换为前端期望的格式
        Ok(QueryResult {
            success: true,
            data: Some(serde_json::to_value(&rust_result).map_err(|e| e.to_string())?),
            message: "查询完成 (Rust原生)".to_string(),
        })
    } else {
        warn!("Time point query failed: {}", rust_result.message.as_deref().unwrap_or("未知错误"));
        let mut process_status = state.current_process.lock().await;
        process_status.output_log.push(format!("[{}] ❌ 查询失败: {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), 
            rust_result.message.as_deref().unwrap_or("未知错误")
        ));
        
        Ok(QueryResult {
            success: false,
            data: Some(serde_json::to_value(&rust_result).map_err(|e| e.to_string())?),
            message: rust_result.message.unwrap_or("查询失败".to_string()),
        })
    }
}

/// Excel导出请求结构
#[derive(Deserialize)]
pub struct ExportFundPoolsRequest {
    pub query_info: Value,
    pub fund_pools: Value,
    pub fund_pool_records: Value,
    pub export_type: String,
    pub output_path: Option<String>, // 用户选择的输出路径
}

/// Excel导出结果结构
#[derive(Serialize)]
pub struct ExportResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub message: Option<String>,
}

/// Tauri命令：导出当前时点资金池信息到Excel
#[command]
pub async fn export_fund_pools_excel(
    request: ExportFundPoolsRequest,
    state: State<'_, AppState>
) -> Result<ExportResult, String> {
    info!("Starting fund pools Excel export for type: {}", request.export_type);
    
    // 更新进程状态日志
    {
        let mut process_status = state.current_process.lock().await;
        process_status.output_log.push(format!("[{}] 📊 开始导出当前时点资金池信息到Excel", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));
    }
    
    // 使用现有的Excel处理器导出资金池记录
    use audit_backend::{ExcelProcessor, Config, FundPoolRecord};
    use std::path::Path;
    
    let config = Config::new();
    let excel_processor = ExcelProcessor::new(config);
    
    // 确定输出路径
    let output_path = if let Some(user_path) = request.output_path {
        // 用户指定了路径，直接使用
        Path::new(&user_path).to_path_buf()
    } else {
        // 用户没有指定路径，使用输入文件目录作为默认
        let input_file_path = request.query_info.get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let input_dir = if !input_file_path.is_empty() {
            Path::new(input_file_path).parent().unwrap_or(Path::new("."))
        } else {
            Path::new("temp_analysis_results") // 兜底目录
        };
        
        // 生成输出文件名
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let output_filename = format!("当前时点资金池信息_{}.xlsx", timestamp);
        input_dir.join(&output_filename)
    };
    
    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    // 获取查询时点信息，用于过滤
    let query_time = request.query_info.get("query_time")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let target_row = request.query_info.get("target_row")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    
    info!("导出当前时点资金池信息: query_time={}, target_row={}", query_time, target_row);
    
    // 将前端数据转换为FundPoolRecord格式，导出截止到当前时点的所有相关记录
    let mut fund_pool_records: Vec<FundPoolRecord> = Vec::new();
    
    if let Some(records_obj) = request.fund_pool_records.as_object() {
        for (_pool_name, records) in records_obj {
            if let Some(record_array) = records.as_array() {
                // 导出所有截止到当前时点的记录，不仅仅是最后一条
                for record_data in record_array {
                    if let Some(record_obj) = record_data.as_object() {
                        // 解析每个字段 - 使用简单的字符串数值转换，避免复杂的Decimal依赖
                        let transaction_time = record_obj.get("交易时间")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                            
                        let pool_name = record_obj.get("资金池名称")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                            
                        let behavior_nature = record_obj.get("行为性质")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        // 解析数值字段
                        let parse_decimal = |s: &str| -> f64 {
                            // 提取数值部分，去除括号和百分号
                            let binding = s.replace("¥", "").replace(",", "");
                            let cleaned = binding.split('(').next().unwrap_or("0").trim();
                            cleaned.parse::<f64>().unwrap_or(0.0)
                        };
                        
                        let inflow = record_obj.get("入金")
                            .and_then(|v| v.as_str())
                            .map(parse_decimal)
                            .unwrap_or(0.0);
                            
                        let outflow = record_obj.get("出金")
                            .and_then(|v| v.as_str())
                            .map(parse_decimal)
                            .unwrap_or(0.0);
                            
                        let total_balance = record_obj.get("总余额")
                            .and_then(|v| v.as_str())
                            .map(parse_decimal)
                            .unwrap_or(0.0);
                            
                        // 解析个人余额和公司余额数值
                        let personal_balance = record_obj.get("个人余额")
                            .and_then(|v| v.as_str())
                            .map(parse_decimal)
                            .unwrap_or(0.0);
                            
                        let company_balance = record_obj.get("公司余额")
                            .and_then(|v| v.as_str())
                            .map(parse_decimal)
                            .unwrap_or(0.0);
                            
                        let cumulative_purchase = record_obj.get("累计申购")
                            .and_then(|v| v.as_str())
                            .map(parse_decimal)
                            .unwrap_or(0.0);
                            
                        let cumulative_redemption = record_obj.get("累计赎回")
                            .and_then(|v| v.as_str())
                            .map(parse_decimal)
                            .unwrap_or(0.0);
                            
                        let net_profit_loss = record_obj.get("净盈亏")
                            .and_then(|v| v.as_str())
                            .map(parse_decimal)
                            .unwrap_or(0.0);
                        
                        // 从个人余额和公司余额中提取占比信息（用于显示）
                        let personal_ratio = record_obj.get("个人余额")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.00 (0%)")
                            .to_string();
                            
                        let company_ratio = record_obj.get("公司余额")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.00 (0%)")
                            .to_string();
                        
                        // 使用audit_backend重新导出的Decimal类型
                        use audit_backend::rust_decimal::Decimal;
                        
                        // 创建记录（包含完整的12个字段）- 使用new方法
                        let record = FundPoolRecord::new(
                            transaction_time,
                            pool_name,
                            Decimal::from_f64_retain(inflow).unwrap_or_default(),
                            Decimal::from_f64_retain(outflow).unwrap_or_default(),
                            Decimal::from_f64_retain(total_balance).unwrap_or_default(),
                            Decimal::from_f64_retain(personal_balance).unwrap_or_default(),
                            Decimal::from_f64_retain(company_balance).unwrap_or_default(),
                            format!("个人:{}, 公司:{}", personal_ratio, company_ratio),
                            format!("个人:{}, 公司:{}", personal_ratio, company_ratio),
                            behavior_nature,
                            Decimal::from_f64_retain(cumulative_purchase).unwrap_or_default(),
                            Decimal::from_f64_retain(cumulative_redemption).unwrap_or_default(),
                            Decimal::from_f64_retain(net_profit_loss).unwrap_or_default(),
                        );
                        
                        fund_pool_records.push(record);
                    }
                }
            }
        }
    }
    
    // 使用现有的Excel导出功能
    match excel_processor.export_fund_pool_records(&fund_pool_records, &output_path) {
        Ok(_) => {
            let output_excel = output_path.to_string_lossy().to_string();
            
            // 更新进程状态日志
            let mut process_status = state.current_process.lock().await;
            process_status.output_log.push(format!("[{}] ✅ 资金池信息已导出到Excel: {}", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), output_excel
            ));
            
            info!("Fund pools Excel export completed: {}", output_excel);
            
            Ok(ExportResult {
                success: true,
                output_path: Some(output_excel),
                message: Some("资金池信息已成功导出到Excel".to_string()),
            })
        }
        Err(e) => {
            error!("Failed to export fund pools to Excel: {}", e);
            
            let mut process_status = state.current_process.lock().await;
            process_status.output_log.push(format!("[{}] ❌ Excel导出失败: {}", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), e
            ));
            
            Ok(ExportResult {
                success: false,
                output_path: None,
                message: Some(format!("Excel导出失败: {}", e)),
            })
        }
    }
}


