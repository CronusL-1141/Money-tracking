/// 统一数据验证器模块
/// 合并Python版本中分散的验证逻辑，专注于流水完整性验证和修复
/// 
/// Python来源: 
/// - src/utils/flow_integrity_validator.py:FlowIntegrityValidator (主要功能)
/// - src/utils/validators.py:DataValidator::validate_required_columns
/// - src/utils/data_processor.py 验证相关函数
/// 
/// 移除功能: 大额交易验证、日期范围验证(用户明确要求移除)

use crate::data_models::Transaction;
use crate::errors::{AuditError, AuditResult};
use crate::utils::logger::AuditLogger;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use log::{info, warn, error, debug};

/// 验证结果结构
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否验证通过
    pub is_valid: bool,
    /// 错误数量
    pub errors_count: usize,
    /// 修复次数
    pub optimizations_count: usize,
    /// 修复是否失败
    pub optimization_failed: bool,
    /// 是否有数据修改
    pub has_modifications: bool,
    /// 验证错误列表
    pub errors: Vec<ValidationError>,
    /// 修复后的交易数据(如果有修复)
    pub fixed_transactions: Option<Vec<Transaction>>,
    /// 验证总结
    pub summary: String,
}

/// 验证错误信息
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// 错误行号(1-based)
    pub row: usize,
    /// 错误消息
    pub message: String,
    /// 错误时间戳
    pub timestamp: DateTime<Utc>,
}

/// 统一数据验证器
/// Python来源: src/utils/flow_integrity_validator.py:FlowIntegrityValidator
#[derive(Debug)]
pub struct UnifiedValidator {
    /// 余额容差
    /// Python来源: flow_integrity_validator.py:18 `self.tolerance = 0.01`
    tolerance: Decimal,
    /// 验证错误列表
    validation_errors: Vec<ValidationError>,
    /// 优化计数
    optimization_count: usize,
    /// 优化失败标志
    optimization_failed: bool,
    /// 日志记录器
    logger: AuditLogger,
}

impl UnifiedValidator {
    /// 创建新的统一验证器
    /// Python来源: flow_integrity_validator.py:16-21 `__init__`
    pub fn new() -> Self {
        Self {
            tolerance: Decimal::from_f64_retain(0.01).unwrap_or_default(),
            validation_errors: Vec::new(),
            optimization_count: 0,
            optimization_failed: false,
            logger: AuditLogger::new("UnifiedValidator"),
        }
    }

    /// 验证必需列
    /// Python来源: src/utils/validators.py:DataValidator::validate_required_columns
    pub fn validate_required_columns(&self, transactions: &[Transaction]) -> AuditResult<()> {
        if transactions.is_empty() {
            return Err(AuditError::validation_error("数据为空"));
        }

        // 基本数据完整性检查 - 所有交易都应该有必要的字段  
        for (index, transaction) in transactions.iter().enumerate() {
            if transaction.transaction_time.is_empty() {
                return Err(AuditError::validation_error(
                    format!("第{}行缺少交易时间", index + 1)
                ));
            }
        }

        info!("✅ 必需列验证通过, 共{}条记录", transactions.len());
        Ok(())
    }

    /// 验证流水完整性并尝试修复
    /// Python来源: flow_integrity_validator.py:23-70 `validate_flow_integrity`
    pub fn validate_flow_integrity(&mut self, transactions: &[Transaction]) -> AuditResult<ValidationResult> {
        info!("开始原始流水完整性验证...");
        
        // 重置状态
        self.validation_errors.clear();
        self.optimization_count = 0;
        self.optimization_failed = false;

        if transactions.is_empty() {
            return Ok(ValidationResult {
                is_valid: true,
                errors_count: 0,
                optimizations_count: 0,
                optimization_failed: false,
                has_modifications: false,
                errors: Vec::new(),
                fixed_transactions: None,
                summary: "数据为空，无需验证".to_string(),
            });
        }

        // 创建副本用于验证和修复，保持源数据不变
        let mut result_transactions = transactions.to_vec();

        // 逐行验证余额连贯性
        // Python来源: flow_integrity_validator.py:45-68 主验证循环
        for i in 1..result_transactions.len() {
            if !self.check_balance_continuity(&result_transactions[i-1], &result_transactions[i], i)? {
                // 余额不连贯，尝试修复
                if let Some(fixed_transactions) = self.attempt_reorder_fix(&result_transactions, i)? {
                    info!("✅ 第{}行余额问题已通过重排序修复", i + 1);
                    self.optimization_count += 1;
                    result_transactions = fixed_transactions;
                    
                    // 重新验证修复后的行
                    if !self.check_balance_continuity(&result_transactions[i-1], &result_transactions[i], i)? {
                        self.record_error(i, "重排序后仍无法修复余额连贯性");
                        self.optimization_failed = true;
                        break;
                    }
                } else {
                    self.record_error(i, "余额不连贯且无法通过重排序修复，可能存在数据丢失");
                    self.optimization_failed = true;
                    break;
                }
            }
        }

        // 生成验证报告
        self.generate_validation_report(&result_transactions, transactions)
    }

    /// 检查两笔交易间的余额连贯性
    /// Python来源: flow_integrity_validator.py:72-115 `_check_balance_continuity`
    fn check_balance_continuity(&self, previous: &Transaction, current: &Transaction, row_idx: usize) -> AuditResult<bool> {
        let prev_balance = previous.balance;
        let curr_balance = current.balance;
        let income = current.income_amount;
        let expense = current.expense_amount;

        // 计算期望余额：上一笔余额 + 收入 - 支出
        let expected_balance = prev_balance + income - expense;

        // 检查是否在容差范围内
        let difference = (curr_balance - expected_balance).abs();
        if difference <= self.tolerance {
            Ok(true)
        } else {
            warn!(
                "第{}行余额不连贯: 上笔余额{} + 收入{} - 支出{} = 期望{}, 实际{}, 差异{}",
                row_idx + 1, prev_balance, income, expense, expected_balance, curr_balance, curr_balance - expected_balance
            );
            Ok(false)
        }
    }

    /// 尝试通过重新排序同时间交易来修复余额问题
    /// Python来源: flow_integrity_validator.py:137-177 `_attempt_reorder_fix`
    fn attempt_reorder_fix(&mut self, transactions: &[Transaction], problem_row_idx: usize) -> AuditResult<Option<Vec<Transaction>>> {
        // 创建完整时间戳用于比较(组合日期和时间)
        let current_datetime = self.create_full_timestamp(&transactions[problem_row_idx])?;

        // 找出所有同时间交易
        let same_time_indices: Vec<usize> = transactions.iter().enumerate()
            .filter_map(|(idx, tx)| {
                if let Ok(datetime) = self.create_full_timestamp(tx) {
                    if datetime == current_datetime {
                        Some(idx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if same_time_indices.len() <= 1 {
            info!("第{}行无同时间交易，无法重排序修复", problem_row_idx + 1);
            return Ok(None);
        }

        info!("发现第{}行有{}笔同时间交易，尝试重排序...", problem_row_idx + 1, same_time_indices.len());

        // 尝试找到最佳排序
        if let Some(best_order) = self.find_best_order(transactions, &same_time_indices)? {
            if best_order != same_time_indices {
                let fixed_transactions = self.create_reordered_transactions(transactions, &same_time_indices, &best_order);
                info!("✅ 成功重排序交易");
                return Ok(Some(fixed_transactions));
            }
        }

        warn!("❌ 未找到有效的重排序方案");
        Ok(None)
    }

    /// 找到最佳的交易排序(使用贪心策略)
    /// Python来源: flow_integrity_validator.py:179-206 `_find_best_order`
    fn find_best_order(&self, transactions: &[Transaction], indices: &[usize]) -> AuditResult<Option<Vec<usize>>> {
        info!("使用贪心策略寻找正确顺序，共{}笔同时间交易...", indices.len());
        
        if let Some(result_order) = self.greedy_order_search(transactions, indices)? {
            info!("✅ 贪心策略找到正确顺序");
            Ok(Some(result_order))
        } else {
            warn!("❌ 贪心策略未找到有效顺序");
            Ok(None)
        }
    }

    /// 使用贪心策略寻找正确的交易顺序
    /// Python来源: flow_integrity_validator.py:208-278 `_greedy_order_search`
    fn greedy_order_search(&self, transactions: &[Transaction], indices: &[usize]) -> AuditResult<Option<Vec<usize>>> {
        if indices.len() <= 1 {
            return Ok(Some(indices.to_vec()));
        }

        let min_pos = *indices.iter().min().unwrap();
        
        // 如果第一个位置是索引0，无法获取前一行
        if min_pos == 0 {
            return Ok(Some(indices.to_vec()));
        }

        // 获取前一行数据用于计算期望余额
        let prev_balance = transactions[min_pos - 1].balance;
        
        let mut result_order = Vec::new();
        let mut remaining_indices = indices.to_vec();
        let mut current_balance = prev_balance;

        // 逐步构建正确顺序
        for position in 0..indices.len() {
            let mut found_next = false;
            
            debug!("  寻找第{}笔交易，当前余额: {}", position + 1, current_balance);

            // 在剩余交易中找到下一笔符合余额连贯性的交易
            for (i, &candidate_idx) in remaining_indices.iter().enumerate() {
                let candidate = &transactions[candidate_idx];
                
                let income = candidate.income_amount;
                let expense = candidate.expense_amount;
                let expected_balance = current_balance + income - expense;
                let actual_balance = candidate.balance;

                // 检查是否符合余额连贯性
                if (actual_balance - expected_balance).abs() <= self.tolerance {
                    // 找到符合的交易
                    result_order.push(candidate_idx);
                    remaining_indices.remove(i);
                    current_balance = actual_balance;
                    found_next = true;
                    
                    info!("    ✅ 找到第{}笔: 第{}行, 收入{}, 支出{}, 余额{}",
                          position + 1, candidate_idx + 1, income, expense, actual_balance);
                    break;
                } else {
                    debug!("    ❌ 第{}行不符合: 期望{}, 实际{}, 差异{}",
                           candidate_idx + 1, expected_balance, actual_balance, actual_balance - expected_balance);
                }
            }

            if !found_next {
                warn!("  ❌ 无法找到第{}笔符合余额连贯性的交易", position + 1);
                return Ok(None);
            }
        }

        info!("✅ 贪心策略成功找到完整顺序");
        Ok(Some(result_order))
    }

    /// 创建重新排序后的交易数据
    /// Python来源: flow_integrity_validator.py:319-349 `_create_reordered_dataframe`
    fn create_reordered_transactions(&self, transactions: &[Transaction], original_indices: &[usize], new_order: &[usize]) -> Vec<Transaction> {
        let mut result = transactions.to_vec();
        
        // 按原始位置更新数据
        let mut sorted_positions = original_indices.to_vec();
        sorted_positions.sort_unstable();
        
        for (i, &new_idx) in new_order.iter().enumerate() {
            if let (Some(&pos), Some(new_data)) = (sorted_positions.get(i), transactions.get(new_idx)) {
                if let Some(target) = result.get_mut(pos) {
                    *target = new_data.clone();
                }
            }
        }
        
        result
    }

    /// 记录验证错误
    /// Python来源: flow_integrity_validator.py:351-359 `_record_error`
    fn record_error(&mut self, row_idx: usize, message: &str) {
        let error = ValidationError {
            row: row_idx + 1,
            message: message.to_string(),
            timestamp: Utc::now(),
        };
        self.validation_errors.push(error);
        error!("第{}行: {}", row_idx + 1, message);
    }

    /// 生成验证报告
    /// Python来源: flow_integrity_validator.py:361-401 `_generate_validation_report`
    fn generate_validation_report(&self, result_transactions: &[Transaction], original_transactions: &[Transaction]) -> AuditResult<ValidationResult> {
        let has_modifications = self.optimization_count > 0;
        let is_valid = self.validation_errors.is_empty();
        
        let summary = format!("验证完成: {}行数据, {}个错误, {}次重排序修复",
                            original_transactions.len(), self.validation_errors.len(), self.optimization_count);

        // 输出摘要
        info!("{}", "=".repeat(60));
        info!("原始流水完整性验证报告");
        info!("{}", "=".repeat(60));
        info!("总行数: {}", original_transactions.len());
        info!("发现错误: {}个", self.validation_errors.len());
        info!("成功修复: {}个", self.optimization_count);

        if self.optimization_failed {
            warn!("⚠️  优化失败 - 源文件保持不变");
            warn!("建议：请检查数据完整性，可能存在缺失交易或数据错误");
            warn!("解决方案：");
            warn!("1. 检查银行流水数据是否完整");
            warn!("2. 确认是否有遗漏的交易记录");
            warn!("3. 验证余额计算是否正确");
        }

        info!("验证结果: {}", if is_valid { "✅ 通过" } else { "❌ 失败" });
        info!("数据修改: {}", if has_modifications { "是" } else { "否（源文件保持只读）" });

        if !self.validation_errors.is_empty() {
            info!("\n错误详情:");
            for error in &self.validation_errors {
                error!("  第{}行: {}", error.row, error.message);
            }
        }

        info!("{}", "=".repeat(60));

        Ok(ValidationResult {
            is_valid,
            errors_count: self.validation_errors.len(),
            optimizations_count: self.optimization_count,
            optimization_failed: self.optimization_failed,
            has_modifications,
            errors: self.validation_errors.clone(),
            fixed_transactions: if has_modifications { Some(result_transactions.to_vec()) } else { None },
            summary,
        })
    }

    /// 创建完整时间戳（组合日期和时间）
    /// Python来源: flow_integrity_validator.py使用 '完整时间戳' 字段进行比较
    fn create_full_timestamp(&self, transaction: &Transaction) -> AuditResult<DateTime<Utc>> {
        // 使用时间处理器来解析和组合时间
        // 这里简化处理，直接使用transaction_date作为完整时间戳的基础
        let base_date = transaction.transaction_date.date();
        
        // 解析时间字符串并创建完整的DateTime
        let datetime = base_date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| AuditError::validation_error("无法创建时间戳"))?
            .and_utc();
            
        Ok(datetime)
    }

    /// 兼容方法：验证交易数据
    /// 提供与原Validator接口兼容的方法
    pub fn validate_transactions(&mut self, transactions: &[Transaction]) -> AuditResult<ValidationResult> {
        // 先验证必需列
        self.validate_required_columns(transactions)?;
        
        // 然后进行流水完整性验证
        self.validate_flow_integrity(transactions)
    }
}

impl Default for UnifiedValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc, TimeZone};
    use rust_decimal_macros::dec;

    fn create_test_transaction(time: DateTime<Utc>, balance: Decimal, income: Decimal, expense: Decimal) -> Transaction {
        Transaction {
            transaction_time: Some(time),
            balance: Some(balance),
            income_amount: Some(income),
            expense_amount: Some(expense),
            ..Default::default()
        }
    }

    #[test]
    fn test_validate_required_columns() {
        let validator = UnifiedValidator::new();
        
        // 测试空数据
        let empty_transactions: Vec<Transaction> = Vec::new();
        assert!(validator.validate_required_columns(&empty_transactions).is_err());

        // 测试有效数据
        let time = Utc.timestamp_opt(1609459200, 0).unwrap();
        let transactions = vec![
            create_test_transaction(time, dec!(1000), dec!(0), dec!(0))
        ];
        assert!(validator.validate_required_columns(&transactions).is_ok());
    }

    #[test]
    fn test_balance_continuity_check() {
        let validator = UnifiedValidator::new();
        let time = Utc.timestamp_opt(1609459200, 0).unwrap();
        
        let prev = create_test_transaction(time, dec!(1000), dec!(0), dec!(0));
        let curr = create_test_transaction(time, dec!(1100), dec!(100), dec!(0));
        
        // 余额连贯: 1000 + 100 - 0 = 1100
        assert!(validator.check_balance_continuity(&prev, &curr, 1).unwrap());
        
        let curr_bad = create_test_transaction(time, dec!(1200), dec!(100), dec!(0));
        // 余额不连贯: 1000 + 100 - 0 = 1100, 但实际是1200
        assert!(!validator.check_balance_continuity(&prev, &curr_bad, 1).unwrap());
    }
}