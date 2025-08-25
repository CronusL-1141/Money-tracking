//! 集成测试程序
//! 
//! 测试Rust后端的完整功能

use audit_backend::{IntegrationProcessor, AuditAnalysisRequest, TimePointQueryRequest, AuditLogger};
use log::{info, error, warn};
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    AuditLogger::init()?;
    
    info!("🚀 启动集成测试程序");
    
    // 解析命令行参数
    let matches = Command::new("integration_test")
        .about("测试Rust后端集成功能")
        .arg(Arg::new("input")
            .short('i')
            .long("input")
            .value_name("INPUT_FILE")
            .help("测试用的Excel文件路径")
            .default_value("../流水.xlsx"))
        .get_matches();
    
    let input_file = matches.get_one::<String>("input").unwrap();
    
    // 检查文件是否存在
    if !std::path::Path::new(input_file).exists() {
        warn!("⚠️ 测试文件不存在: {}", input_file);
        warn!("将创建一个基础的功能测试");
        test_basic_functionality().await?;
        return Ok(());
    }
    
    info!("📂 使用测试文件: {}", input_file);
    
    // 创建集成处理器
    let mut processor = IntegrationProcessor::new();
    
    // 测试1: 获取算法列表
    info!("🧪 测试1: 获取算法列表");
    let algorithms = processor.get_algorithms();
    info!("✅ 支持的算法数量: {}", algorithms.len());
    for algo in &algorithms {
        info!("  - {}: {}", algo.name, algo.description);
    }
    
    // 测试2: 文件路径验证
    info!("🧪 测试2: 文件路径验证");
    let is_valid = processor.validate_file_path(input_file);
    info!("✅ 文件路径有效性: {}", is_valid);
    
    if !is_valid {
        error!("❌ 文件路径无效，跳过后续测试");
        return Ok(());
    }
    
    // 测试3: FIFO算法分析
    info!("🧪 测试3: FIFO算法分析");
    let fifo_request = AuditAnalysisRequest {
        algorithm: "FIFO".to_string(),
        input_file: input_file.to_string(),
        output_file: Some("test_fifo_result.xlsx".to_string()),
    };
    
    match processor.run_audit_analysis(fifo_request).await {
        Ok(response) => {
            info!("✅ FIFO分析成功");
            info!("  处理时间: {:.2}秒", response.processing_time.unwrap_or(0.0));
            info!("  输出文件: {:?}", response.output_files);
            info!("  个人余额: {:.2}", response.summary.personal_balance);
            info!("  公司余额: {:.2}", response.summary.company_balance);
        }
        Err(e) => {
            error!("❌ FIFO分析失败: {}", e);
        }
    }
    
    // 测试4: 差额计算法分析
    info!("🧪 测试4: 差额计算法分析");
    let balance_request = AuditAnalysisRequest {
        algorithm: "BALANCE_METHOD".to_string(),
        input_file: input_file.to_string(),
        output_file: Some("test_balance_method_result.xlsx".to_string()),
    };
    
    match processor.run_audit_analysis(balance_request).await {
        Ok(response) => {
            info!("✅ 差额计算法分析成功");
            info!("  处理时间: {:.2}秒", response.processing_time.unwrap_or(0.0));
            info!("  个人余额: {:.2}", response.summary.personal_balance);
            info!("  公司余额: {:.2}", response.summary.company_balance);
        }
        Err(e) => {
            error!("❌ 差额计算法分析失败: {}", e);
        }
    }
    
    // 测试5: 时点查询
    info!("🧪 测试5: 时点查询");
    let query_request = TimePointQueryRequest {
        file_path: input_file.to_string(),
        row_number: 10,
        algorithm: "FIFO".to_string(),
    };
    
    match processor.query_time_point(query_request).await {
        Ok(response) => {
            info!("✅ 时点查询成功");
            if let Some(summary) = response.summary {
                info!("  第10行时点状态:");
                info!("    个人余额: {:.2}", summary.personal_balance);
                info!("    公司余额: {:.2}", summary.company_balance);
            }
        }
        Err(e) => {
            error!("❌ 时点查询失败: {}", e);
        }
    }
    
    info!("🎉 集成测试完成");
    Ok(())
}

/// 基础功能测试（不需要真实Excel文件）
async fn test_basic_functionality() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 执行基础功能测试");
    
    let processor = IntegrationProcessor::new();
    
    // 测试算法列表
    let algorithms = processor.get_algorithms();
    assert!(!algorithms.is_empty());
    assert!(algorithms.iter().any(|a| a.name == "FIFO"));
    assert!(algorithms.iter().any(|a| a.name == "BALANCE_METHOD"));
    info!("✅ 算法列表测试通过");
    
    // 测试文件验证
    assert!(!processor.validate_file_path("nonexistent_file.xlsx"));
    info!("✅ 文件验证测试通过");
    
    info!("🎉 基础功能测试完成");
    Ok(())
}