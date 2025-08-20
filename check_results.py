import pandas as pd

print('🔍 检查根目录的完整算法结果文件...')

files = {
    'FIFO_资金追踪结果.xlsx': 'FIFO算法完整结果',
    'BALANCE_METHOD_资金追踪结果.xlsx': '差额计算法完整结果', 
    '场外资金池记录_FIFO.xlsx': 'FIFO场外资金池',
    '场外资金池记录_BALANCE_METHOD.xlsx': '差额计算法场外资金池'
}

for file, desc in files.items():
    print(f'\n📊 {desc}: {file}')
    try:
        df = pd.read_excel(file)
        print(f'  - 行数: {len(df):,}')
        print(f'  - 列数: {len(df.columns)}')
        print(f'  - 前8列: {list(df.columns)[:8]}')
        if len(df.columns) > 8:
            print(f'  - 其余: {list(df.columns)[8:]}')
        # 检查时间跨度
        if '交易日期' in df.columns:
            print(f'  - 时间跨度: {df["交易日期"].min()} 到 {df["交易日期"].max()}')
    except Exception as e:
        print(f'  ❌ 读取失败: {e}')
