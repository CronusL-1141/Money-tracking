import pandas as pd
from utils.flow_integrity_validator import FlowIntegrityValidator
from utils.data_processor import DataProcessor
import time

print("测试新的贪心策略...")

# 1. 先进行数据预处理
print("正在预处理数据...")
processor = DataProcessor()
df = processor.预处理财务数据('流水.xlsx')

if df is None:
    print("❌ 数据预处理失败")
    exit(1)

# 2. 测试验证器
validator = FlowIntegrityValidator()

print(f"预处理后数据共 {len(df)} 行")

# 记录开始时间
start_time = time.time()

# 执行验证
result = validator.validate_flow_integrity(df)

# 记录结束时间
end_time = time.time()
processing_time = end_time - start_time

print("\n" + "="*80)
print("新贪心策略测试结果")
print("="*80)
print(f"处理时间: {processing_time:.2f} 秒")
print(f"验证结果: {'✅ 通过' if result['is_valid'] else '❌ 失败'}")
print(f"发现错误: {result['errors_count']} 个")
print(f"成功修复: {result['optimizations_count']} 个")
print(f"优化状态: {'成功' if not result['optimization_failed'] else '失败'}")

if result['optimizations_count'] > 0:
    print(f"数据已修复: 使用修复后的数据框进行后续分析")
else:
    print(f"数据未修改: 保持原始数据框")

# 显示详细错误信息
if result['errors_count'] > 0:
    print(f"\n错误详情:")
    for i, error in enumerate(result['errors'][:3]):  # 只显示前3个错误
        print(f"{i+1}. 第{error['row']}行: {error['message']}")

print(f"\n策略优势分析:")
print(f"- 贪心策略：O(n²) 复杂度，逐步验证")
print(f"- 原排列策略：O(n!) 复杂度，盲目尝试")
print(f"- 对于6个同时间交易：贪心=36次比较 vs 排列=720种组合")
print(f"- 效率提升约 {720/36:.1f} 倍") 