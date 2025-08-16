import pandas as pd

df = pd.read_excel('流水.xlsx')
print('原始第9790-9800行数据:')
print('=' * 120)

for i in range(9789, 9799):
    if i < len(df):
        row = df.iloc[i]
        print(f'第{i+1:4d}行: 收入={row["交易收入金额"]:>8} 支出={row["交易支出金额"]:>8} '
              f'余额={row["余额"]:>10.2f} 日期={row["交易日期"]} 时间={row["交易时间"]}')

# 特别检查第9794行
print('\n特别检查第9794行:')
if 9793 < len(df):
    row = df.iloc[9793]
    print(f'第9794行详情:')
    print(f'  收入: {row["交易收入金额"]}')
    print(f'  支出: {row["交易支出金额"]}')
    print(f'  余额: {row["余额"]}')
    print(f'  日期: {row["交易日期"]}')
    print(f'  时间: {row["交易时间"]}')
    print(f'  资金属性: {row["资金属性"]}') 