# 独立测试套件

这个目录包含了资金追踪系统的所有独立测试，用于验证不同阶段的功能实现。

## 目录结构

```
independent_tests/
├── README.md                           # 本文件 - 测试套件总览
├── preprocessing_validation_test/       # 数据预处理验证测试（已完成）
│   ├── Cargo.toml                      # Rust项目配置
│   ├── src/main.rs                     # 预处理验证程序
│   ├── python_preprocessed_output.xlsx # Python基准预处理结果
│   ├── rust_preprocessed_output.xlsx   # Rust预处理结果
│   └── rust_preprocessing_validation.json # 验证结果报告
├── algorithm_validation_tests/          # 算法验证测试（当前实施中）
│   ├── Cargo.toml                      # 算法测试项目配置
│   ├── src/
│   │   ├── common.rs                   # 共享验证逻辑
│   │   ├── fifo_test.rs               # FIFO算法验证测试
│   │   └── balance_method_test.rs     # 差额计算法验证测试
│   └── output/                        # 测试输出目录
│       ├── fifo_test_results.json     # FIFO测试结果
│       ├── fifo_test_report.txt       # FIFO测试报告
│       ├── balance_method_test_results.json
│       └── balance_method_test_report.txt
└── performance_benchmarks/             # 性能基准测试（计划中）
    └── (待实现)
```

## 测试类型

### 1. 数据预处理验证测试 ✅ 已完成
- **位置**: `preprocessing_validation_test/`
- **目的**: 验证Rust后端的数据预处理功能与Python版本的100%一致性
- **方法**: 使用相同输入数据，比较Python和Rust的预处理输出
- **状态**: ✅ 已通过验证 - Rust预处理与Python完全一致
- **特色**: 保持独立的Transaction数据结构，专为测试目的设计

### 2. 算法验证测试 ✅ 基础框架完成
- **位置**: `algorithm_validation_tests/`
- **目的**: 验证FIFO和差额计算法算法的正确性
- **当前状态**: 
  - ✅ 基础框架建立完成
  - ✅ 简化测试运行成功 (`simplified_fifo_test.rs`)
  - 🔄 完整验证测试待启用
- **测试方法**: 使用现有的算法结果Excel文件作为验证基准
  - 输入: `../../流水.xlsx`
  - FIFO基准: `../../FIFO_资金追踪结果.xlsx`
  - 差额计算法基准: `../../BALANCE_METHOD_资金追踪结果.xlsx`
- **测试内容**:
  - 算法基础构造功能验证 ✅
  - 交易记录增强字段计算
  - 挪用金额和垫付金额计算
  - 个人余额和公司余额追踪
  - 投资产品处理
  - 审计摘要生成

### 3. 性能基准测试 ⏳ 计划中
- **位置**: `performance_benchmarks/`
- **目的**: 对比Python和Rust实现的性能差异
- **内容**: 处理速度、内存使用、大数据量处理能力

## 运行测试

### 算法验证测试

```bash
# 进入测试目录
cd independent_tests/algorithm_validation_tests

# 运行FIFO算法验证
cargo run --bin fifo_algorithm_test

# 运行差额计算法验证
cargo run --bin balance_method_test

# 编译所有测试
cargo build --release
```

### 预处理验证测试（已完成）

```bash
# 进入预处理测试目录
cd independent_tests/preprocessing_validation_test

# 运行预处理验证（如需重新验证）
cargo run --release
```

## 验证数据来源

所有测试使用根目录下的验证数据：
- **输入数据**: `流水.xlsx` - 原始交易流水数据
- **FIFO基准**: `FIFO_资金追踪结果.xlsx` - Python版本FIFO算法输出
- **差额计算法基准**: `BALANCE_METHOD_资金追踪结果.xlsx` - Python版本差额计算法输出

## 测试报告

每个测试完成后会在对应的output目录生成：
1. **JSON结果文件**: 包含详细的测试数据和比较结果
2. **文本报告文件**: 人类可读的测试报告，包含通过/失败状态和差异详情

## 注意事项

1. **路径一致性**: 所有测试都使用绝对路径引用验证数据
2. **依赖关系**: 算法测试依赖`../../rust-backend`库
3. **输出隔离**: 每个测试的输出都保存在独立目录中，避免相互影响
4. **版本对应**: 确保测试使用的基准数据与当前算法版本一致