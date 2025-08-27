//! 时点查询相关的Tauri命令
//! 
//! 使用Rust后端直接实现，不再依赖Python

use tauri::{command, State};
use audit_backend::{TimePointService, TimePointQueryRequest, TimePointQueryResult, FundPoolQueryRequest, FundPoolQueryResult};
use crate::{AppState, generate_id, QueryHistory, TimePointQuery, QueryResult};
use chrono::Utc;
use log::{info, error, warn};
use tokio::sync::Mutex;
use std::sync::Arc;

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
    
    // 创建时点查询服务
    let mut service = match TimePointService::new(query.algorithm.clone()) {
        Ok(service) => service,
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
    };
    
    // 执行查询
    let rust_result = match service.query_time_point(request).await {
        Ok(rust_result) => rust_result,
        Err(e) => {
            error!("Time point query failed: {}", e);
            let mut process_status = state.current_process.lock().await;
            process_status.output_log.push(format!("[{}] ❌ 查询异常: {}", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), e
            ));
            
            return Ok(QueryResult {
                success: false,
                data: None,
                message: e.to_string(),
            });
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

/// Tauri命令：资金池查询
#[command]
pub async fn query_fund_pool(
    pool_name: String,
    file_path: String,
    row_number: u32,
    algorithm: String,
    _state: State<'_, AppState>
) -> Result<FundPoolQueryResult, String> {
    info!("Fund pool query: pool={}, file={}, row={}, algorithm={}", pool_name, file_path, row_number, algorithm);
    
    // 构建请求
    let request = FundPoolQueryRequest {
        pool_name: pool_name.clone(),
        file_path,
        row_number: row_number as usize,
        algorithm,
    };
    
    // 创建时点查询服务
    let mut service = match TimePointService::new(request.algorithm.clone()) {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to create TimePointService for fund pool query: {}", e);
            return Ok(FundPoolQueryResult {
                success: false,
                pool_name,
                message: Some(format!("服务初始化失败: {}", e)),
            });
        }
    };
    
    // 执行查询
    let result = match service.query_fund_pool(request).await {
        Ok(result) => result,
        Err(e) => {
            error!("Fund pool query failed: {}", e);
            FundPoolQueryResult {
                success: false,
                pool_name,
                message: Some(e.to_string()),
            }
        }
    };
    
    Ok(result)
}