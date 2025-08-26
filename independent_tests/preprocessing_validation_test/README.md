# 预处理验证测试

本文件夹包含Python和Rust预处理完整功能的对比验证工具。

## 目的

验证Rust工具层实现是否100%精确匹配Python版本的预处理功能，包括：
- Excel数据读取
- 时间处理 
- 数据预处理
- 统一验证和修复
- Excel输出写入

## 文件说明

- `generate_python_preprocessed.py` - Python版本完整预处理生成器
- `src/main.rs` - Rust版本完整预处理生成器
- `compare_outputs.py` - Python vs Rust 输出对比工具
- `Cargo.toml` - Rust项目配置

## 运行步骤

### 1. 生成Python标准输出
```bash
python generate_python_preprocessed.py
```

输出文件：
- `python_preprocessed_output.xlsx` - Python修复后的完整Excel数据
- `python_preprocessing_validation.json` - Python处理统计信息

### 2. 生成Rust输出
```bash
cargo run --bin generate_rust_preprocessed
```

输出文件：
- `rust_preprocessed_output.xlsx` - Rust修复后的完整Excel数据  
- `rust_preprocessing_validation.json` - Rust处理统计信息

### 3. 对比两个输出
```bash
python compare_outputs.py
```

输出文件：
- `comparison_report.json` - 详细对比报告

## 验证标准

两个输出文件应该完全一致：

1. **数据结构一致**: 相同的列名和列数
2. **数据顺序一致**: 按交易日期时间排序后完全相同
3. **数据值一致**: 每个单元格的值完全匹配（数值允许0.01误差）
4. **统计信息一致**: 收入、支出、余额、交易数量等统计完全匹配

## 成功标准

当`compare_outputs.py`输出"✅ Python和Rust预处理输出完全一致！"时，表示：
- 🎉 Rust工具层实现100%精确匹配Python功能
- 🚀 可以进入算法层开发阶段

## 技术要点

### Python处理流程
1. ExcelProcessor.read_excel() - 读取原始Excel
2. TimeProcessor.process_datetime_columns() - 时间处理
3. DataProcessor.preprocess_data() - 数据预处理
4. UnifiedValidator.validate_and_repair_data() - 验证修复
5. ExcelProcessor.write_excel() - 写入Excel

### Rust处理流程  
1. read_excel_data() - 使用calamine读取Excel
2. 时间处理（在读取时完成）
3. 数据预处理（排序等）
4. validate_and_repair_same_time_transactions() - 验证修复
5. write_excel_data() - 使用rust_xlsxwriter写入Excel

## 预期结果示例

成功运行后应该看到类似输出：
```
=== Python vs Rust 预处理输出完整对比工具 ===
✅ Python Excel: 9,799 行
✅ Rust Excel: 9,799 行

🔍 逐列数据对比:
  交易日期: 0 个时间不匹配
  交易时间: 0 个不匹配
  交易收入金额: 0 个数值差异
  交易支出金额: 0 个数值差异
  余额: 0 个数值差异
  资金属性: 0 个不匹配

✅ Python和Rust预处理输出完全一致！
🎉 Rust工具层实现100%精确匹配Python功能！
```