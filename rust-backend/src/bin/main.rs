//! FIFO资金追踪审计系统 Rust CLI
//! 
//! 支持双算法切换的命令行工具，提供完整的审计分析功能

use std::process;
use std::io::{self, Write};
use std::collections::HashMap;

use clap::{Args, Parser, Subcommand};
use tokio;

use audit_backend::AuditService;

/// FIFO资金追踪审计系统 v3.2 - 支持双算法
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// 选择算法类型：FIFO（先进先出）或 BALANCE_METHOD（差额计算法）
    #[arg(short, long, value_enum, default_value_t = Algorithm::Fifo)]
    algorithm: Algorithm,
    
    /// 输入Excel文件路径
    #[arg(short, long, default_value = "流水.xlsx")]
    input: String,
    
    /// 输出Excel文件路径（默认根据算法自动生成）
    #[arg(short, long)]
    output: Option<String>,
    
    /// 安静模式，减少输出信息
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出所有可用算法
    ListAlgorithms,
    /// 比较两种算法的结果  
    Compare(CompareArgs),
    /// 交互模式
    Interactive,
    /// 运行单算法分析（默认命令）
    Analyze(AnalyzeArgs),
}

#[derive(Args)]
struct CompareArgs {
    /// 输入Excel文件路径
    #[arg(short, long, default_value = "流水.xlsx")]
    input: String,
}

#[derive(Args)]
struct AnalyzeArgs {
    /// 选择算法类型
    #[arg(short, long, value_enum, default_value_t = Algorithm::Fifo)]
    algorithm: Algorithm,
    
    /// 输入Excel文件路径
    #[arg(short, long, default_value = "流水.xlsx")]
    input: String,
    
    /// 输出Excel文件路径
    #[arg(short, long)]
    output: Option<String>,
    
    /// 安静模式，减少输出信息
    #[arg(short, long)]
    quiet: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Algorithm {
    #[value(name = "FIFO")]
    Fifo,
    #[value(name = "BALANCE_METHOD")]
    BalanceMethod,
}

impl Algorithm {
    fn to_string(&self) -> &'static str {
        match self {
            Algorithm::Fifo => "FIFO",
            Algorithm::BalanceMethod => "BALANCE_METHOD",
        }
    }
}

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();
    
    let cli = Cli::parse();
    
    // 如果没有子命令，检查是否为交互模式
    let result = match &cli.command {
        Some(Commands::ListAlgorithms) => {
            list_algorithms().await
        }
        Some(Commands::Compare(args)) => {
            compare_algorithms(&args.input).await
        }
        Some(Commands::Interactive) => {
            interactive_mode().await
        }
        Some(Commands::Analyze(args)) => {
            run_single_analysis(
                args.algorithm.to_string(),
                &args.input,
                args.output.as_deref(),
                args.quiet,
            ).await
        }
        None => {
            // 默认行为：如果有输入参数就分析，否则进入交互模式
            if std::env::args().len() > 1 {
                run_single_analysis(
                    cli.algorithm.to_string(),
                    &cli.input,
                    cli.output.as_deref(),
                    cli.quiet,
                ).await
            } else {
                interactive_mode().await
            }
        }
    };
    
    if let Err(e) = result {
        eprintln!("❌ 错误: {}", e);
        process::exit(1);
    }
}

/// 列出所有可用算法
async fn list_algorithms() -> Result<(), Box<dyn std::error::Error>> {
    println!("可用算法:");
    
    let service = AuditService::new();
    let algorithms = service.get_algorithms_info();
    
    for (algo, desc) in algorithms.iter() {
        println!("  {}: {}", algo, desc);
    }
    
    Ok(())
}

/// 运行单算法分析
async fn run_single_analysis(
    algorithm: &str,
    input_file: &str,
    output_file: Option<&str>,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    
    if !quiet {
        let service = AuditService::new();
        let algorithms = service.get_algorithms_info();
        let algo_desc = algorithms.get(algorithm).unwrap_or(&"未知算法");
        
        println!("🚀 启动算法: {}", algorithm);
        println!("📝 算法描述: {}", algo_desc);
        println!("📂 输入文件: {}", input_file);
        io::stdout().flush()?;
    }
    
    // 创建审计服务
    let service = AuditService::new().with_suppress_output(quiet);
    
    // 分析数据
    let result = service.analyze_financial_data(algorithm, input_file, output_file).await;
    
    match result {
        Ok((summary, transactions, _output_files)) => {
            if !quiet {
                println!("✅ {}算法分析完成！", algorithm);
                println!("📊 处理行数: {}", transactions.len());
                
                let output_name = if let Some(output) = output_file {
                    output.to_string()
                } else {
                    format!("{}_资金追踪结果.xlsx", algorithm)
                };
                println!("💾 结果已保存至: {}", output_name);
                
                // 显示关键指标
                println!("\n📈 分析摘要:");
                println!("   💰 个人余额: {:.2}", summary.personal_balance);
                println!("   🏢 公司余额: {:.2}", summary.company_balance);
                println!("   📊 总余额: {:.2}", summary.total_balance);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ {}算法分析失败: {}", algorithm, e);
            Err(Box::new(e))
        }
    }
}

/// 比较两种算法的结果
async fn compare_algorithms(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 开始比较FIFO与差额计算法...");
    println!("📂 输入文件: {}", input_file);
    
    let mut results = HashMap::new();
    let algorithms = ["FIFO", "BALANCE_METHOD"];
    
    // 运行两种算法
    for &algorithm in &algorithms {
        println!("\n正在运行 {} 算法...", algorithm);
        
        let service = AuditService::new().with_suppress_output(true);
        
        match service.analyze_financial_data(algorithm, input_file, None::<&str>).await {
            Ok((summary, transactions, _output_files)) => {
                results.insert(algorithm, (summary, transactions.len()));
                println!("✅ {} 算法完成", algorithm);
            }
            Err(e) => {
                eprintln!("❌ {} 算法失败: {}", algorithm, e);
                return Err(Box::new(e));
            }
        }
    }
    
    // 显示比较结果
    println!("\n{}", "=".repeat(80));
    println!("📊 算法对比结果");
    println!("{}", "=".repeat(80));
    
    println!("{:<20} {:<20} {:<20} {:<15}", "指标", "FIFO算法", "差额计算法", "差异");
    println!("{}", "-".repeat(80));
    
    if let (Some((fifo_summary, fifo_count)), Some((balance_summary, balance_count))) = 
        (results.get("FIFO"), results.get("BALANCE_METHOD")) {
        
        let metrics = vec![
            ("个人余额", fifo_summary.personal_balance, balance_summary.personal_balance),
            ("公司余额", fifo_summary.company_balance, balance_summary.company_balance),
            ("总余额", fifo_summary.total_balance, balance_summary.total_balance),
        ];
        
        for (metric, fifo_val, balance_val) in metrics {
            let diff = balance_val - fifo_val;
            println!("{:<20} {:<20.2} {:<20.2} {:<15.2}", metric, fifo_val, balance_val, diff);
        }
        
        println!("\n📋 对比说明:");
        println!("1. FIFO算法：按先进先出原则分配资金来源");  
        println!("2. 差额计算法：个人余额优先扣除，简化计算逻辑");
        println!("3. 差异：正数表示差额计算法数值更大，负数表示更小");
        println!("4. 处理记录数：FIFO {} 条，差额计算法 {} 条", fifo_count, balance_count);
    }
    
    Ok(())
}

/// 交互模式
async fn interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(60));
    println!("🏦 FIFO资金追踪审计系统 v3.2 - Rust版本");
    println!("{}", "=".repeat(60));
    
    // 显示算法选项
    let service = AuditService::new();
    let algorithms = service.get_algorithms_info();
    let algo_list: Vec<_> = algorithms.keys().collect();
    
    println!("\n可选算法:");
    for (i, algo) in algo_list.iter().enumerate() {
        let desc = algorithms.get(*algo).unwrap();
        println!("  {}. {}: {}", i + 1, algo, desc);
    }
    
    // 用户选择算法
    let algorithm = loop {
        print!("\n请选择算法 (1-{}) 或输入 'q' 退出: ", algo_list.len());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.to_lowercase() == "q" {
            println!("👋 退出系统");
            return Ok(());
        }
        
        match input.parse::<usize>() {
            Ok(choice) if choice >= 1 && choice <= algo_list.len() => {
                break algo_list[choice - 1];
            }
            _ => {
                println!("❌ 无效选择，请重试");
            }
        }
    };
    
    // 文件选择
    print!("\n请输入Excel文件路径 (默认: 流水.xlsx): ");
    io::stdout().flush()?;
    
    let mut input_file = String::new();
    io::stdin().read_line(&mut input_file)?;
    let input_file = input_file.trim();
    let input_file = if input_file.is_empty() {
        "流水.xlsx"
    } else {
        input_file
    };
    
    // 运行分析
    run_single_analysis(algorithm, input_file, None, false).await?;
    
    Ok(())
}