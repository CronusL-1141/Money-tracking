import pandas as pd

# 检查原始流水数据
print('🔍 检查原始流水数据...')
df_original = pd.read_excel('data/input/流水.xlsx')
print(f'原始流水数据行数: {len(df_original):,}')
print(f'时间跨度: {df_original["交易日期"].min()} 到 {df_original["交易日期"].max()}')

# 检查完整算法结果
print('\n🔍 检查完整算法结果...')
df_fifo = pd.read_excel('FIFO_资金追踪结果.xlsx')
df_balance = pd.read_excel('BALANCE_METHOD_资金追踪结果.xlsx')

print(f'FIFO结果行数: {len(df_fifo):,}')
print(f'BALANCE结果行数: {len(df_balance):,}')

# 对比分析
print('\n🤔 数据对比分析:')
print(f'原始数据: {len(df_original):,}行')
print(f'处理结果: {len(df_fifo):,}行') 
ratio = len(df_fifo) / len(df_original) * 100
print(f'处理比例: {ratio:.1f}%')

if len(df_fifo) < len(df_original):
    print('⚠️  算法结果行数少于原始数据，可能原因:')
    print('   1. 数据预处理过程中过滤了部分记录')
    print('   2. 流水完整性验证失败的行被跳过')
    print('   3. 某些无效交易被排除')
