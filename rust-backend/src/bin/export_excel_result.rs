//! Excel结果导出程序
//! 
//! 用于导出审计分析结果到Excel文件

use audit_backend::{AuditService, AuditLogger};
use log::{info, error};
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    AuditLogger::init().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    info!("🚀 启动Excel结果导出程序");
    
    // 解析命令行参数
    let matches = Command::new("export_excel_result")
        .about("导出FIFO审计分析结果到Excel")
        .arg(Arg::new("algorithm")
            .short('a')
            .long("algorithm")
            .value_name("ALGORITHM")
            .help("算法类型 (FIFO 或 BALANCE_METHOD)")
            .required(true))
        .arg(Arg::new("input")
            .short('i')
            .long("input")
            .value_name("INPUT_FILE")
            .help("输入Excel文件路径")
            .required(true))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("OUTPUT_FILE")
            .help("输出Excel文件路径"))
        .get_matches();
    
    let algorithm = matches.get_one::<String>("algorithm").unwrap();
    let input_file = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output");
    
    info!("📂 输入文件: {}", input_file);
    info!("🔧 使用算法: {}", algorithm);
    if let Some(output) = output_file {
        info!("💾 输出文件: {}", output);
    }
    
    // 执行分析
    let mut service = AuditService::new();
    
    match service.analyze_financial_data(algorithm, input_file, output_file).await {
        Ok(summary) => {
            info!("✅ 分析完成");
            info!("📊 审计摘要:");
            info!("  个人余额: {:.2}", summary.personal_balance);
            info!("  公司余额: {:.2}", summary.company_balance);
            info!("  总余额: {:.2}", summary.total_balance);
            info!("  累计挪用: {:.2}", summary.total_misappropriation);
            info!("  累计垫付: {:.2}", summary.total_advance_payment);
            info!("  资金缺口: {:.2}", summary.funding_gap);
            info!("  投资产品数量: {}", summary.investment_product_count);
        }
        Err(e) => {
            error!("❌ 分析失败: {}", e);
            return Err(Box::new(e));
        }
    }
    
    info!("🎉 程序执行完成");
    Ok(())
}