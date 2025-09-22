import pandas as pd
import numpy as np

def analyze_data():
    file_path = "流水.xlsx"
    
    print("=" * 60)
    print("原始流水数据分析")
    print("=" * 60)
    
    try:
        # 读取数据
        df = pd.read_excel(file_path, sheet_name="需计算")
        
        print(f"数据形状: {df.shape}")
        print(f"列名: {list(df.columns)}")
        
        print("\n数据类型:")
        for col in df.columns:
            print(f"  {col}: {df[col].dtype}")
        
        print("\n前5行数据:")
        print(df.head())
        
        print("\n后5行数据:")
        print(df.tail())
        
        print("\n时间范围:")
        print(f"  最早: {df['交易日期'].min()}")
        print(f"  最晚: {df['交易日期'].max()}")
        
        print("\n金额统计:")
        for col in ['交易收入金额', '交易支出金额', '余额']:
            non_null = df[col].dropna()
            if len(non_null) > 0:
                print(f"  {col}:")
                print(f"    最小: {non_null.min()}")
                print(f"    最大: {non_null.max()}")
                print(f"    平均: {non_null.mean():.2f}")
        
        print("\n资金属性分布:")
        attr_counts = df['资金属性'].value_counts()
        print(f"  总类型数: {len(attr_counts)}")
        for attr, count in attr_counts.head(15).items():
            percent = count/len(df)*100
            print(f"  {attr}: {count}次 ({percent:.1f}%)")
        
        # 查找资金池相关
        print("\n资金池相关交易:")
        keywords = ['理财', '投资', '保险', '申购', '赎回', '资金池']
        pool_mask = df['资金属性'].str.contains('|'.join(keywords), na=False)
        pool_data = df[pool_mask]
        print(f"  资金池交易数: {len(pool_data)}")
        
        if len(pool_data) > 0:
            pool_attrs = pool_data['资金属性'].value_counts().head(10)
            for attr, count in pool_attrs.items():
                print(f"    {attr}: {count}次")
        
        # 个人公司分布
        print("\n个人/公司分布:")
        personal = df[df['资金属性'].str.contains('个人', na=False)]
        company = df[df['资金属性'].str.contains('公司', na=False)]
        print(f"  个人交易: {len(personal)}次")
        print(f"  公司交易: {len(company)}次")
        print(f"  其他交易: {len(df)-len(personal)-len(company)}次")
        
        # 应收应付
        print("\n应收应付分布:")
        receivable = df[df['资金属性'].str.contains('应收', na=False)]
        payable = df[df['资金属性'].str.contains('应付', na=False)]
        print(f"  应收: {len(receivable)}次")
        print(f"  应付: {len(payable)}次")
        
        # 余额连续性检查
        print("\n余额连续性检查:")
        df_sorted = df.sort_values(['交易日期', '交易时间']).reset_index(drop=True)
        
        balance_errors = []
        for i in range(1, len(df_sorted)):
            prev_row = df_sorted.iloc[i-1]
            curr_row = df_sorted.iloc[i]
            
            income = curr_row['交易收入金额'] if pd.notna(curr_row['交易收入金额']) else 0
            expense = curr_row['交易支出金额'] if pd.notna(curr_row['交易支出金额']) else 0
            
            expected_balance = prev_row['余额'] + income - expense
            actual_balance = curr_row['余额']
            
            if abs(expected_balance - actual_balance) > 0.01:
                balance_errors.append({
                    'index': i,
                    'date': curr_row['交易日期'],
                    'time': curr_row['交易时间'],
                    'expected': expected_balance,
                    'actual': actual_balance,
                    'diff': abs(expected_balance - actual_balance),
                    'attr': curr_row['资金属性']
                })
        
        print(f"  检查记录数: {len(df_sorted)-1}")
        print(f"  余额不一致: {len(balance_errors)}次")
        
        if len(balance_errors) > 0:
            print("  前5个不一致:")
            for i, error in enumerate(balance_errors[:5]):
                print(f"    {i+1}. {error['date']} {error['time']} {error['attr']}")
                print(f"       期望:{error['expected']:.2f} 实际:{error['actual']:.2f} 差异:{error['diff']:.2f}")
        
        print("\n分析完成!")
        
    except Exception as e:
        print(f"分析出错: {e}")

if __name__ == "__main__":
    analyze_data()