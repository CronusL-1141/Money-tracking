import pandas as pd
from utils.data_processor import DataProcessor

print("检查数据预处理逻辑...")

# 1. 原始数据
print("\n1. 原始数据第5609行:")
df_raw = pd.read_excel('流水.xlsx')
row_raw = df_raw.iloc[5608]  # 第5609行
print(f'收入: {row_raw["交易收入金额"]}')
print(f'支出: {row_raw["交易支出金额"]}')
print(f'余额: {row_raw["余额"]}')

# 2. 预处理后数据
print("\n2. 预处理后第5609行:")
processor = DataProcessor()
df_processed = processor.预处理财务数据('流水.xlsx')

if df_processed is not None:
    row_processed = df_processed.iloc[5608]  # 第5609行
    print(f'交易收入金额: {row_processed["交易收入金额"]}')
    print(f'交易支出金额: {row_processed["交易支出金额"]}')
    print(f'余额: {row_processed["余额"]}')
    print(f'完整时间戳: {row_processed["完整时间戳"]}')
    
    # 3. 处理单行交易结果
    print("\n3. 处理单行交易结果:")
    处理结果 = processor.处理单行交易(row_processed, 5608)
    print(f'方向: {处理结果["方向"]}')
    print(f'实际金额: {处理结果["实际金额"]}')
    print(f'资金属性: {处理结果["资金属性"]}')
    print(f'完整时间戳: {处理结果["完整时间戳"]}')
else:
    print("预处理失败") 