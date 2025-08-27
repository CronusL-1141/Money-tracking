//! 服务层集成测试
//! 
//! 验证AuditService是否能正确调用底层算法并产生正确结果

use audit_backend::{AuditService};
use std::path::Path;

#[tokio::test]
async fn test_audit_service_creation() {
    // 测试服务创建
    let service = AuditService::new();
    let algorithms = service.get_supported_algorithms();
    
    assert_eq!(algorithms.len(), 2);
    assert!(algorithms.contains(&"FIFO"));
    assert!(algorithms.contains(&"BALANCE_METHOD"));
    
    // 测试算法信息
    let info = service.get_algorithms_info();
    assert!(info.contains_key("FIFO"));
    assert!(info.contains_key("BALANCE_METHOD"));
    
    println!("✅ 服务层基础功能测试通过");
}

#[tokio::test]
async fn test_audit_service_fifo_integration() {
    // 集成测试：使用真实数据文件测试FIFO算法
    let service = AuditService::new();
    let input_file = "C:\\Users\\TUF\\Desktop\\资金追踪\\流水.xlsx";
    
    // 检查文件是否存在
    if !Path::new(input_file).exists() {
        println!("⚠️ 测试文件不存在，跳过集成测试: {}", input_file);
        return;
    }
    
    println!("🚀 开始FIFO算法集成测试");
    
    // 执行分析（不导出文件）
    let result = service.analyze_financial_data("FIFO", input_file, None::<&str>).await;
    
    match result {
        Ok((summary, transactions)) => {
            println!("✅ FIFO算法集成测试成功");
            println!("   📊 处理交易记录: {} 条", transactions.len());
            println!("   💰 个人余额: {}", summary.personal_balance);
            println!("   🏢 公司余额: {}", summary.company_balance);
            println!("   📈 总余额: {}", summary.total_balance);
            
            // 基本验证
            assert!(transactions.len() > 0);
            assert_eq!(summary.total_balance, summary.personal_balance + summary.company_balance);
        }
        Err(e) => {
            panic!("❌ FIFO算法集成测试失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_audit_service_balance_method_integration() {
    // 集成测试：使用真实数据文件测试差额计算法
    let service = AuditService::new();
    let input_file = "C:\\Users\\TUF\\Desktop\\资金追踪\\流水.xlsx";
    
    // 检查文件是否存在
    if !Path::new(input_file).exists() {
        println!("⚠️ 测试文件不存在，跳过集成测试: {}", input_file);
        return;
    }
    
    println!("🚀 开始Balance Method算法集成测试");
    
    // 执行分析（不导出文件）
    let result = service.analyze_financial_data("BALANCE_METHOD", input_file, None::<&str>).await;
    
    match result {
        Ok((summary, transactions)) => {
            println!("✅ Balance Method算法集成测试成功");
            println!("   📊 处理交易记录: {} 条", transactions.len());
            println!("   💰 个人余额: {}", summary.personal_balance);
            println!("   🏢 公司余额: {}", summary.company_balance);
            println!("   📈 总余额: {}", summary.total_balance);
            
            // 基本验证
            assert!(transactions.len() > 0);
            assert_eq!(summary.total_balance, summary.personal_balance + summary.company_balance);
        }
        Err(e) => {
            panic!("❌ Balance Method算法集成测试失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_audit_service_algorithm_consistency() {
    // 一致性测试：验证两种算法的数据处理一致性
    let service = AuditService::new();
    let input_file = "C:\\Users\\TUF\\Desktop\\资金追踪\\流水.xlsx";
    
    // 检查文件是否存在
    if !Path::new(input_file).exists() {
        println!("⚠️ 测试文件不存在，跳过一致性测试: {}", input_file);
        return;
    }
    
    println!("🔍 开始算法一致性测试");
    
    // 分别运行两种算法
    let fifo_result = service.analyze_financial_data("FIFO", input_file, None::<&str>).await;
    let balance_result = service.analyze_financial_data("BALANCE_METHOD", input_file, None::<&str>).await;
    
    match (fifo_result, balance_result) {
        (Ok((fifo_summary, fifo_transactions)), Ok((balance_summary, balance_transactions))) => {
            println!("✅ 两种算法都成功执行");
            
            // 验证数据处理一致性
            assert_eq!(fifo_transactions.len(), balance_transactions.len(), "交易记录数量应该一致");
            
            // 验证总余额一致（应该都匹配最后一笔交易的余额）
            assert_eq!(fifo_summary.total_balance, balance_summary.total_balance, "两种算法的总余额应该一致");
            
            println!("   📊 FIFO处理记录: {} 条, 总余额: {}", fifo_transactions.len(), fifo_summary.total_balance);
            println!("   📊 Balance Method处理记录: {} 条, 总余额: {}", balance_transactions.len(), balance_summary.total_balance);
            
            // 验证个人/公司余额分配不同（这是预期的，因为算法不同）
            println!("   💡 FIFO个人余额: {}, 公司余额: {}", fifo_summary.personal_balance, fifo_summary.company_balance);
            println!("   💡 Balance Method个人余额: {}, 公司余额: {}", balance_summary.personal_balance, balance_summary.company_balance);
            
            println!("✅ 算法一致性测试通过");
        }
        (Err(e), _) => panic!("❌ FIFO算法失败: {}", e),
        (_, Err(e)) => panic!("❌ Balance Method算法失败: {}", e),
    }
}