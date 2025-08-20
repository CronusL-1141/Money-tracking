"""
分析现有数据格式并创建标准化测试数据集
"""
import pandas as pd
import numpy as np
import os
from datetime import datetime, timedelta
import random

def analyze_existing_data():
    """分析现有数据格式"""
    try:
        # 读取现有数据
        df = pd.read_excel('data/input/流水.xlsx')
        
        print('🔍 现有数据分析:')
        print(f'总行数: {len(df):,}')
        print(f'列数: {len(df.columns)}')
        
        print('\n📋 列名和数据类型:')
        for i, col in enumerate(df.columns):
            non_null = df[col].notna().sum()
            print(f'{i+1:2d}. {col:<20} ({str(df[col].dtype):<10}) - {non_null:,}/{len(df):,} 非空')
        
        print('\n📊 前5行数据样例:')
        display_cols = df.columns[:6] if len(df.columns) > 6 else df.columns
        print(df[display_cols].head().to_string())
        
        if '资金属性' in df.columns:
            print('\n💰 资金属性类型统计:')
            fund_attrs = df['资金属性'].value_counts()
            for attr, count in fund_attrs.head(15).items():
                print(f'  {attr}: {count:,}')
        
        if '交易日期' in df.columns:
            print('\n📅 日期范围分析:')
            df['交易日期'] = pd.to_datetime(df['交易日期'])
            print(f'最早日期: {df["交易日期"].min()}')
            print(f'最晚日期: {df["交易日期"].max()}')
            print(f'日期跨度: {(df["交易日期"].max() - df["交易日期"].min()).days} 天')
        
        # 分析数值字段
        numeric_cols = ['交易收入金额', '交易支出金额', '余额']
        for col in numeric_cols:
            if col in df.columns:
                print(f'\n📈 {col} 统计:')
                print(f'  最小值: {df[col].min():,.2f}')
                print(f'  最大值: {df[col].max():,.2f}')
                print(f'  平均值: {df[col].mean():,.2f}')
                print(f'  非零个数: {(df[col] > 0).sum():,}')
        
        return df
        
    except Exception as e:
        print(f'❌ 读取原始数据失败: {e}')
        return None

def create_test_datasets(original_df):
    """基于原始数据创建5套测试数据集"""
    if original_df is None:
        print('❌ 无法创建测试数据集，原始数据读取失败')
        return
    
    # 确保测试数据目录存在
    test_dir = 'rust-backend/test_data'
    os.makedirs(test_dir, exist_ok=True)
    
    print(f'\n🏗️ 开始创建测试数据集到目录: {test_dir}')
    
    # 1. Minimal Dataset (50行) - 基础功能测试
    print('📊 创建 minimal 测试数据集 (50行)...')
    minimal_df = original_df.head(50).copy()
    minimal_df.to_excel(f'{test_dir}/test_data_minimal.xlsx', index=False)
    
    # 2. Standard Dataset (1000行) - 常规场景测试  
    print('📊 创建 standard 测试数据集 (1000行)...')
    if len(original_df) >= 1000:
        standard_df = original_df.head(1000).copy()
    else:
        # 如果原始数据不够1000行，重复采样
        standard_df = original_df.sample(n=min(1000, len(original_df)), replace=True).copy()
        standard_df = standard_df.reset_index(drop=True)
    standard_df.to_excel(f'{test_dir}/test_data_standard.xlsx', index=False)
    
    # 3. Investment Dataset - 专门包含投资产品的测试数据
    print('📊 创建 investment 测试数据集...')
    investment_keywords = ['理财', '投资', '保险', '关联银行卡', '资金池']
    if '资金属性' in original_df.columns:
        investment_mask = original_df['资金属性'].str.contains('|'.join(investment_keywords), na=False)
        investment_rows = original_df[investment_mask]
        if len(investment_rows) > 0:
            # 包含投资产品的记录 + 一些普通交易记录
            normal_rows = original_df[~investment_mask].head(200)
            investment_df = pd.concat([investment_rows.head(300), normal_rows]).reset_index(drop=True)
        else:
            # 如果没有投资产品记录，创建一些模拟的
            investment_df = create_mock_investment_data(original_df)
    else:
        investment_df = original_df.head(500).copy()
    
    investment_df.to_excel(f'{test_dir}/test_data_investment.xlsx', index=False)
    
    # 4. Complex Dataset (10000行) - 大数据量测试
    print('📊 创建 complex 测试数据集 (10000行)...')
    if len(original_df) >= 10000:
        complex_df = original_df.head(10000).copy()
    else:
        # 重复采样创建大数据集
        n_repeats = (10000 // len(original_df)) + 1
        complex_df = pd.concat([original_df] * n_repeats).head(10000).reset_index(drop=True)
    complex_df.to_excel(f'{test_dir}/test_data_complex.xlsx', index=False)
    
    # 5. Integrity Issues Dataset - 包含完整性问题的测试数据
    print('📊 创建 integrity_issues 测试数据集...')
    integrity_df = create_integrity_issues_data(original_df)
    integrity_df.to_excel(f'{test_dir}/test_data_integrity_issues.xlsx', index=False)
    
    print(f'\n✅ 测试数据集创建完成！')
    print(f'📁 保存位置: {test_dir}/')
    
    # 验证创建的文件
    for filename in ['test_data_minimal.xlsx', 'test_data_standard.xlsx', 
                     'test_data_investment.xlsx', 'test_data_complex.xlsx', 
                     'test_data_integrity_issues.xlsx']:
        filepath = f'{test_dir}/{filename}'
        if os.path.exists(filepath):
            size = os.path.getsize(filepath)
            print(f'  ✅ {filename} - {size:,} bytes')
        else:
            print(f'  ❌ {filename} - 创建失败')

def create_mock_investment_data(base_df):
    """创建模拟的投资产品数据"""
    investment_df = base_df.head(500).copy()
    
    # 添加一些投资产品相关的资金属性
    investment_products = [
        '理财-SL100613100620',
        '投资-基金产品A',
        '保险-人寿保险001',
        '关联银行卡-工商银行',
        '资金池-固定收益001'
    ]
    
    # 随机替换一些资金属性
    if '资金属性' in investment_df.columns:
        n_replacements = min(100, len(investment_df))
        replacement_indices = random.sample(range(len(investment_df)), n_replacements)
        for idx in replacement_indices:
            investment_df.at[idx, '资金属性'] = random.choice(investment_products)
    
    return investment_df

def create_integrity_issues_data(base_df):
    """创建包含完整性问题的测试数据"""
    issues_df = base_df.head(500).copy()
    
    # 引入一些完整性问题
    if '余额' in issues_df.columns:
        # 1. 余额计算错误
        for i in range(10):
            if i < len(issues_df):
                issues_df.at[i, '余额'] += random.uniform(-1000, 1000)
        
        # 2. 创建一些余额跳跃
        for i in range(50, 60):
            if i < len(issues_df):
                issues_df.at[i, '余额'] = issues_df.at[i-1, '余额'] + random.uniform(10000, 50000)
    
    # 3. 时间顺序问题
    if '交易日期' in issues_df.columns:
        # 打乱部分日期顺序
        for i in range(20, 40):
            if i < len(issues_df) and i > 0:
                # 交换相邻两行的日期
                temp_date = issues_df.at[i, '交易日期']
                issues_df.at[i, '交易日期'] = issues_df.at[i-1, '交易日期']
                issues_df.at[i-1, '交易日期'] = temp_date
    
    return issues_df

if __name__ == '__main__':
    print('🚀 开始分析数据格式并创建测试数据集...')
    
    # 分析现有数据
    original_df = analyze_existing_data()
    
    # 创建测试数据集
    create_test_datasets(original_df)
    
    print('\n🎉 任务完成！')
