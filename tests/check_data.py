import pandas as pd

# 读取数据
df = pd.read_excel('流水.xlsx')

# 查看第5609行附近的数据
print('第5605-5615行详细数据:')
print('=' * 150)

for i in range(5604, 5614):  # 第5605-5614行 (索引从0开始)
    if i < len(df):
        row = df.iloc[i]
        print(f'第{i+1:4d}行: 日期={row["交易日期"]} 时间={row["交易时间"]} ' +
              f'收入={row["交易收入金额"]:>10.2f} 支出={row["交易支出金额"]:>10.2f} ' +
              f'余额={row["余额"]:>12.2f} 属性={row["资金属性"]}')

print('\n手动验证余额连贯性:')
print('-' * 100)

# 手动验证第5608和5609行
prev_row = df.iloc[5607]  # 第5608行
curr_row = df.iloc[5608]  # 第5609行

print(f'第5608行余额: {prev_row["余额"]:>12.2f}')
print(f'第5609行收入: {curr_row["交易收入金额"]:>12.2f}')
print(f'第5609行支出: {curr_row["交易支出金额"]:>12.2f}')
print(f'期望余额: {prev_row["余额"] + curr_row["交易收入金额"] - curr_row["交易支出金额"]:>12.2f}')
print(f'实际余额: {curr_row["余额"]:>12.2f}')
print(f'差异: {curr_row["余额"] - (prev_row["余额"] + curr_row["交易收入金额"] - curr_row["交易支出金额"]):>12.2f}')

# 检查时间戳
prev_timestamp = f'{prev_row["交易日期"]} {prev_row["交易时间"]}'
curr_timestamp = f'{curr_row["交易日期"]} {curr_row["交易时间"]}'
print(f'\n时间戳检查:')
print(f'第5608行时间戳: {prev_timestamp}')
print(f'第5609行时间戳: {curr_timestamp}') 