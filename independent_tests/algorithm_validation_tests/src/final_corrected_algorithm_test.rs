use audit_backend::{
    Transaction, AuditError, Config, AuditSummary,
    FifoTracker, BalanceMethodTracker,
    UnifiedValidator, ExcelProcessor
};
use rust_decimal::Decimal;
use colored::Colorize;

/// 最终修复版算法测试 - 修复时间戳和初始余额分配问题
/// 
/// 修复内容：
/// 1. 时间戳正确合并日期和时间
/// 2. 智能分析第一笔交易，正确分配初始个人/公司余额
#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("{}", "🚀 开始最终修复版算法测试".bright_green().bold());
    
    // 阶段1：数据准备
    let test_data = prepare_test_data().await?;
    println!("{}", format!("✅ 数据准备完成：{} 条交易记录", test_data.len()).green());
    
    // 阶段2：FIFO算法测试 - 智能初始化
    test_fifo_algorithm_with_smart_initialization(&test_data).await?;
    
    // 阶段3：Balance Method算法测试 - 智能初始化
    test_balance_method_algorithm_with_smart_initialization(&test_data).await?;
    
    println!("{}", "🎉 最终修复版算法测试完成！".bright_green().bold());
    Ok(())
}

/// 准备测试数据
async fn prepare_test_data() -> Result<Vec<Transaction>, AuditError> {
    println!("{}", "📋 步骤1：准备测试数据...".yellow());
    
    let config = Config::new();
    let excel_processor = ExcelProcessor::new(config);
    let raw_transactions = excel_processor.read_transactions("C:\\Users\\TUF\\Desktop\\资金追踪\\流水.xlsx")?;
    
    println!("{}", format!("   📊 读取原始交易记录: {} 条", raw_transactions.len()).blue());
    
    let mut validator = UnifiedValidator::new();
    let validation_result = validator.validate_transactions(&raw_transactions)?;
    
    println!("{}", format!("   🔧 数据验证结果: {} ({} 个错误)", 
        if validation_result.is_valid { "通过" } else { "发现问题" },
        validation_result.errors_count).blue());
    
    Ok(raw_transactions)
}

/// 智能分析第一笔交易，确定初始余额分配
/// 
/// 逻辑：根据第一笔交易的收支情况和资金属性，推算交易前的余额构成
fn analyze_initial_balance_allocation(first_transaction: &Transaction) -> Result<(Decimal, Decimal, String), AuditError> {
    println!("{}", "🧠 智能分析第一笔交易，确定初始余额分配...".blue());
    
    let income = first_transaction.income_amount;
    let expense = first_transaction.expense_amount;
    let final_balance = first_transaction.balance;
    let fund_attr = &first_transaction.fund_attribute;
    
    println!("{}", format!("   第一笔交易分析:").cyan());
    println!("{}", format!("   - 时间: {}", first_transaction.transaction_date.format("%Y/%m/%d %H:%M:%S")).cyan());
    println!("{}", format!("   - 收入金额: {}", income).cyan());
    println!("{}", format!("   - 支出金额: {}", expense).cyan());
    println!("{}", format!("   - 余额: {}", final_balance).cyan());
    println!("{}", format!("   - 资金属性: {}", fund_attr).cyan());
    
    // 计算交易前余额
    let pre_transaction_balance = final_balance - income + expense;
    println!("{}", format!("   - 交易前余额: {} - {} + {} = {}", final_balance, income, expense, pre_transaction_balance).cyan());
    
    // 基于资金属性判断资金来源性质
    let (initial_personal, initial_company, reasoning) = if fund_attr.contains("个人") || fund_attr.contains("私人") {
        // 个人相关交易，交易前余额推测为个人资金
        (pre_transaction_balance, Decimal::ZERO, "基于个人相关资金属性，初始余额分配给个人")
    } else if fund_attr.contains("公司") || fund_attr.contains("应收") || fund_attr.contains("应付") {
        // 公司相关交易，交易前余额推测为公司资金
        (Decimal::ZERO, pre_transaction_balance, "基于公司相关资金属性，初始余额分配给公司")
    } else {
        // 其他情况，默认为公司资金（保守处理）
        (Decimal::ZERO, pre_transaction_balance, "未明确资金性质，默认分配给公司")
    };
    
    println!("{}", format!("   💡 初始化决策: {}", reasoning).green());
    println!("{}", format!("   - 初始个人余额: {}", initial_personal).green());
    println!("{}", format!("   - 初始公司余额: {}", initial_company).green());
    println!("{}", format!("   - 总初始余额: {}", initial_personal + initial_company).green());
    
    Ok((initial_personal, initial_company, reasoning.to_string()))
}

/// FIFO算法测试 - 智能初始化
async fn test_fifo_algorithm_with_smart_initialization(transactions: &[Transaction]) -> Result<(), AuditError> {
    println!("{}", "🔍 步骤2：FIFO算法测试（智能初始化）...".yellow());
    
    if transactions.is_empty() {
        return Err(AuditError::validation_error("没有交易数据可供测试"));
    }
    
    // 创建FIFO追踪器
    let config = Config::new();
    let mut fifo_tracker = FifoTracker::new(config.clone());
    
    // 智能分析第一笔交易，确定初始余额分配
    let first_transaction = &transactions[0];
    let (initial_personal, initial_company, reasoning) = analyze_initial_balance_allocation(first_transaction)?;
    
    // 正确初始化两个余额类型
    // 1. 先初始化个人余额（可能为0）
    fifo_tracker.initialize_balance(initial_personal, "个人")?;
    
    // 2. 如果有公司初始余额，添加为公司资金
    if initial_company > Decimal::ZERO {
        // 使用process_inflow添加公司初始余额，这样会正确处理FIFO队列
        fifo_tracker.process_inflow(initial_company, "公司初始余额", Some(first_transaction.transaction_date))?;
    }
    
    println!("{}", format!("   ⚙️  FIFO追踪器智能初始化完成").blue());
    println!("{}", format!("   - {}", reasoning).blue());
    
    // 处理所有交易并正确更新Transaction结构
    let mut updated_transactions = Vec::new();
    let mut processed_count = 0;
    
    for (index, transaction) in transactions.iter().enumerate() {
        let mut tx = transaction.clone();
        
        // 根据交易类型处理
        let result = if transaction.income_amount > Decimal::ZERO {
            // 资金流入处理 - 需要区分投资收入和普通收入
            // 判断是否为投资产品：包含"-"分隔符的都是投资产品（格式：分类-名称）
            if transaction.fund_attribute.contains('-') {
                // 投资产品收入，调用投资赎回处理（理财-xxx，保险-xxx，投资-xxx，关联银行卡-xxx等）
                fifo_tracker.process_investment_redemption(
                    transaction.income_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            } else {
                // 普通资金流入处理
                fifo_tracker.process_inflow(
                    transaction.income_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            }
        } else if transaction.expense_amount > Decimal::ZERO {
            // 资金流出处理
            // 判断是否为投资产品申购：包含"-"分隔符的都是投资产品
            if transaction.fund_attribute.contains('-') {
                fifo_tracker.process_investment_purchase(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            } else {
                fifo_tracker.process_outflow(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            }
        } else {
            // 其他类型交易
            Ok((Decimal::ZERO, Decimal::ZERO, "无变化".to_string()))
        };
        
        match result {
            Ok((personal_ratio, company_ratio, behavior)) => {
                // 使用正确的API更新Transaction的所有计算字段
                fifo_tracker.update_transaction_fields(&mut tx, personal_ratio, company_ratio, &behavior)?;
                
                updated_transactions.push(tx);
                processed_count += 1;
                
                // 每处理1000条交易输出进度
                if processed_count % 1000 == 0 {
                    println!("{}", format!("     处理进度: {}/{}", processed_count, transactions.len()).cyan());
                }
            }
            Err(e) => {
                println!("{}", format!("   ❌ 第{}条交易处理失败: {}", index + 1, e).red());
                // 即使处理失败，也添加原始记录（但会显示为0值）
                updated_transactions.push(tx);
            }
        }
    }
    
    println!("{}", format!("   ✅ FIFO算法处理完成: {}/{} 条交易", processed_count, transactions.len()).green());
    
    // 输出最终状态验证
    let fifo_summary = fifo_tracker.get_summary()?;
    println!("{}", format!("   📊 最终状态验证:").cyan());
    println!("{}", format!("     - 个人余额: {}", fifo_summary.personal_balance).cyan());
    println!("{}", format!("     - 公司余额: {}", fifo_summary.company_balance).cyan());
    println!("{}", format!("     - 总余额: {}", fifo_summary.total_balance).cyan());
    if let Some(last_tx) = updated_transactions.last() {
        println!("{}", format!("     - 最后一笔交易余额: {}", last_tx.balance).cyan());
        let calculated_total = fifo_summary.personal_balance + fifo_summary.company_balance;
        let balance_match = (calculated_total - last_tx.balance).abs() < Decimal::new(1, 2); // 容差0.01
        println!("{}", format!("     - 余额匹配: {} (计算总额: {} vs 交易余额: {})", 
            if balance_match { "✅ 匹配" } else { "❌ 不匹配" },
            calculated_total,
            last_tx.balance
        ).cyan());
    }
    
    // 使用标准ExcelProcessor API导出结果
    let config = Config::new();
    let excel_processor = ExcelProcessor::new(config);
    
    excel_processor.export_analysis_results(
        &updated_transactions,
        &fifo_summary,
        "output/FIFO_最终修复版算法分析结果.xlsx"
    )?;
    
    println!("{}", "   ✅ FIFO最终修复版算法分析结果已导出".green());
    
    Ok(())
}

/// Balance Method算法测试 - 智能初始化
async fn test_balance_method_algorithm_with_smart_initialization(transactions: &[Transaction]) -> Result<(), AuditError> {
    println!("{}", "🔍 步骤3：Balance Method算法测试（智能初始化）...".yellow());
    
    if transactions.is_empty() {
        return Err(AuditError::validation_error("没有交易数据可供测试"));
    }
    
    // 创建Balance Method追踪器
    let config = Config::new();
    let mut balance_tracker = BalanceMethodTracker::new(config.clone());
    
    // 智能分析第一笔交易，确定初始余额分配
    let first_transaction = &transactions[0];
    let (initial_personal, initial_company, reasoning) = analyze_initial_balance_allocation(first_transaction)?;
    
    // 正确初始化两个余额类型
    // 1. 先初始化个人余额（可能为0）
    balance_tracker.initialize_balance(initial_personal, "个人")?;
    
    // 2. 如果有公司初始余额，添加为公司资金
    if initial_company > Decimal::ZERO {
        // 使用process_inflow添加公司初始余额
        balance_tracker.process_inflow(initial_company, "公司初始余额", Some(first_transaction.transaction_date))?;
    }
    
    println!("{}", format!("   ⚙️  Balance Method追踪器智能初始化完成").blue());
    println!("{}", format!("   - {}", reasoning).blue());
    
    // 处理所有交易并正确更新Transaction结构
    let mut updated_transactions = Vec::new();
    let mut processed_count = 0;
    
    for (index, transaction) in transactions.iter().enumerate() {
        let mut tx = transaction.clone();
        
        // 根据交易类型处理
        let result = if transaction.income_amount > Decimal::ZERO {
            // 资金流入处理 - 需要区分投资收入和普通收入
            // 判断是否为投资产品：包含"-"分隔符的都是投资产品（格式：分类-名称）
            if transaction.fund_attribute.contains('-') {
                // 投资产品收入，调用投资赎回处理（理财-xxx，保险-xxx，投资-xxx，关联银行卡-xxx等）
                balance_tracker.process_investment_redemption(
                    transaction.income_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            } else {
                // 普通资金流入处理
                balance_tracker.process_inflow(
                    transaction.income_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            }
        } else if transaction.expense_amount > Decimal::ZERO {
            // 资金流出处理
            // 判断是否为投资产品申购：包含"-"分隔符的都是投资产品
            if transaction.fund_attribute.contains('-') {
                balance_tracker.process_investment_purchase(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            } else {
                balance_tracker.process_outflow(
                    transaction.expense_amount,
                    &transaction.fund_attribute,
                    Some(transaction.transaction_date),
                )
            }
        } else {
            // 其他类型交易
            Ok((Decimal::ZERO, Decimal::ZERO, "无变化".to_string()))
        };
        
        match result {
            Ok((personal_ratio, company_ratio, behavior)) => {
                // 使用正确的API更新Transaction的所有计算字段
                balance_tracker.update_transaction_fields(&mut tx, personal_ratio, company_ratio, &behavior)?;
                
                updated_transactions.push(tx);
                processed_count += 1;
                
                // 每处理1000条交易输出进度
                if processed_count % 1000 == 0 {
                    println!("{}", format!("     处理进度: {}/{}", processed_count, transactions.len()).cyan());
                }
            }
            Err(e) => {
                println!("{}", format!("   ❌ 第{}条交易处理失败: {}", index + 1, e).red());
                // 即使处理失败，也添加原始记录（但会显示为0值）
                updated_transactions.push(tx);
            }
        }
    }
    
    println!("{}", format!("   ✅ Balance Method算法处理完成: {}/{} 条交易", processed_count, transactions.len()).green());
    
    // 输出最终状态验证
    let balance_summary = balance_tracker.get_summary()?;
    println!("{}", format!("   📊 最终状态验证:").cyan());
    println!("{}", format!("     - 个人余额: {}", balance_summary.personal_balance).cyan());
    println!("{}", format!("     - 公司余额: {}", balance_summary.company_balance).cyan());
    println!("{}", format!("     - 总余额: {}", balance_summary.total_balance).cyan());
    if let Some(last_tx) = updated_transactions.last() {
        println!("{}", format!("     - 最后一笔交易余额: {}", last_tx.balance).cyan());
        let calculated_total = balance_summary.personal_balance + balance_summary.company_balance;
        let balance_match = (calculated_total - last_tx.balance).abs() < Decimal::new(1, 2); // 容差0.01
        println!("{}", format!("     - 余额匹配: {} (计算总额: {} vs 交易余额: {})", 
            if balance_match { "✅ 匹配" } else { "❌ 不匹配" },
            calculated_total,
            last_tx.balance
        ).cyan());
    }
    
    // 使用标准ExcelProcessor API导出结果
    let config = Config::new();
    let excel_processor = ExcelProcessor::new(config);
    
    excel_processor.export_analysis_results(
        &updated_transactions,
        &balance_summary,
        "output/BALANCE_METHOD_最终修复版算法分析结果.xlsx"
    )?;
    
    println!("{}", "   ✅ Balance Method最终修复版算法分析结果已导出".green());
    
    Ok(())
}