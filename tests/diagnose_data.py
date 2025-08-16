import pandas as pd
from models.flow_analyzer import FlowAnalyzer

print("详细诊断数据预处理过程...")

# 1. 读取原始数据
df_original = pd.read_excel('流水.xlsx')
print(f"\n1. 原始数据第5605-5615行:")
print("=" * 100)
for i in range(5604, 5614):
    if i < len(df_original):
        row = df_original.iloc[i]
        print(f'第{i+1:4d}行: 收入={row["交易收入金额"]:>8} 支出={row["交易支出金额"]:>8} '
              f'余额={row["余额"]:>10.2f} 日期={row["交易日期"]} 时间={row["交易时间"]}')

# 2. 手动执行预处理步骤
flow_analyzer = FlowAnalyzer()

# 复制数据
df = df_original.copy()

# 确保交易日期是datetime类型
df['交易日期'] = pd.to_datetime(df['交易日期'])

# 处理交易时间
df['交易时间_格式化'] = df['交易时间'].apply(flow_analyzer.解析交易时间)

# 创建完整时间戳
df['完整时间戳'] = df.apply(
    lambda row: flow_analyzer.创建完整时间戳(row['交易日期'], row['交易时间_格式化']), 
    axis=1
)

print(f"\n2. 添加时间戳后第5605-5615行:")
print("=" * 120)
for i in range(5604, 5614):
    if i < len(df):
        row = df.iloc[i]
        print(f'第{i+1:4d}行: 收入={row["交易收入金额"]:>8} 支出={row["交易支出金额"]:>8} '
              f'余额={row["余额"]:>10.2f} 完整时间戳={row["完整时间戳"]}')

# 3. 添加原始索引并排序
df['原始索引'] = df.index

print(f"\n3. 排序前第5605-5615行的原始索引:")
print("=" * 80)
for i in range(5604, 5614):
    if i < len(df):
        row = df.iloc[i]
        print(f'第{i+1:4d}行: 原始索引={row["原始索引"]} 完整时间戳={row["完整时间戳"]}')

# 4. 执行排序
df_sorted = df.sort_values(['完整时间戳', '原始索引'], kind='stable').reset_index(drop=True)

print(f"\n4. 排序后第5605-5615行:")
print("=" * 120)
for i in range(5604, 5614):
    if i < len(df_sorted):
        row = df_sorted.iloc[i]
        print(f'第{i+1:4d}行: 收入={row["交易收入金额"]:>8} 支出={row["交易支出金额"]:>8} '
              f'余额={row["余额"]:>10.2f} 原始索引={row["原始索引"]} 完整时间戳={row["完整时间戳"]}')

# 5. 删除原始索引列并重排列
df_final = df_sorted.drop('原始索引', axis=1)

print(f"\n5. 最终结果第5605-5615行:")
print("=" * 120)
for i in range(5604, 5614):
    if i < len(df_final):
        row = df_final.iloc[i]
        print(f'第{i+1:4d}行: 收入={row["交易收入金额"]:>8} 支出={row["交易支出金额"]:>8} '
              f'余额={row["余额"]:>10.2f} 完整时间戳={row["完整时间戳"]}') 