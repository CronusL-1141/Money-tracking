#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
专门的余额列对比工具
检查Python和Rust输出的余额列是否完全一致
"""

import pandas as pd
import numpy as np

def compare_balance_columns():
    print("=== Python vs Rust 余额列详细对比 ===")
    
    # 读取两个Excel文件
    try:
        print("\n读取Excel文件...")
        df_python = pd.read_excel("python_preprocessed_output.xlsx")
        df_rust = pd.read_excel("rust_preprocessed_output.xlsx")
        
        print(f"Python Excel: {len(df_python):,} 行")
        print(f"Rust Excel: {len(df_rust):,} 行")
        
        # 显示列结构
        print(f"\nPython Excel列: {list(df_python.columns)}")
        print(f"Rust Excel列: {list(df_rust.columns)}")
        
        # 根据实际列结构确定余额列位置
        # Python: 19列，余额列应该是第5列(索引4)
        # Rust: 6列，余额列应该是第5列(索引4)
        
        python_columns = list(df_python.columns)
        rust_columns = list(df_rust.columns)
        
        # 找到余额列
        python_balance_col = python_columns[4] if len(python_columns) > 4 else None
        rust_balance_col = rust_columns[4] if len(rust_columns) > 4 else None
        
        print(f"\nPython余额列名: {python_balance_col}")
        print(f"Rust余额列名: {rust_balance_col}")
        
        if python_balance_col is None or rust_balance_col is None:
            print("错误: 无法找到余额列")
            return
        
        # 提取余额列数据
        python_balances = df_python[python_balance_col]
        rust_balances = df_rust[rust_balance_col]
        
        print(f"\nPython余额列数据类型: {python_balances.dtype}")
        print(f"Rust余额列数据类型: {rust_balances.dtype}")
        
        # 检查数据长度
        if len(python_balances) != len(rust_balances):
            print(f"错误: 行数不匹配! Python: {len(python_balances)}, Rust: {len(rust_balances)}")
            return
        
        # 转换为数值类型进行比较
        python_balances_numeric = pd.to_numeric(python_balances, errors='coerce')
        rust_balances_numeric = pd.to_numeric(rust_balances, errors='coerce')
        
        # 检查是否有无法转换的值
        python_nan_count = python_balances_numeric.isna().sum()
        rust_nan_count = rust_balances_numeric.isna().sum()
        
        print(f"\nPython余额列无法转换为数值的数量: {python_nan_count}")
        print(f"Rust余额列无法转换为数值的数量: {rust_nan_count}")
        
        if python_nan_count > 0:
            print("Python余额列非数值示例:")
            non_numeric_python = python_balances[python_balances_numeric.isna()]
            print(non_numeric_python.head())
        
        if rust_nan_count > 0:
            print("Rust余额列非数值示例:")
            non_numeric_rust = rust_balances[rust_balances_numeric.isna()]
            print(non_numeric_rust.head())
        
        # 如果都能转换为数值，进行详细对比
        if python_nan_count == 0 and rust_nan_count == 0:
            print("\n开始详细余额对比...")
            
            # 计算差异
            differences = np.abs(python_balances_numeric - rust_balances_numeric)
            tolerance = 0.01  # 允许0.01的误差
            
            significant_diffs = differences > tolerance
            diff_count = significant_diffs.sum()
            
            print(f"超过容差({tolerance})的差异数量: {diff_count}")
            
            if diff_count == 0:
                print("✅ 余额列完全一致!")
            else:
                print(f"❌ 发现 {diff_count} 个显著差异")
                
                # 显示前10个差异
                diff_indices = significant_diffs[significant_diffs].index[:10]
                print("\n前10个显著差异:")
                for idx in diff_indices:
                    python_val = python_balances_numeric.iloc[idx]
                    rust_val = rust_balances_numeric.iloc[idx]
                    diff = abs(python_val - rust_val)
                    print(f"  行 {idx+2}: Python={python_val:.6f}, Rust={rust_val:.6f}, 差异={diff:.6f}")
            
            # 统计信息对比
            print(f"\n统计信息对比:")
            print(f"Python余额总和: {python_balances_numeric.sum():,.2f}")
            print(f"Rust余额总和: {rust_balances_numeric.sum():,.2f}")
            print(f"总和差异: {abs(python_balances_numeric.sum() - rust_balances_numeric.sum()):,.2f}")
            
            print(f"Python最大余额: {python_balances_numeric.max():,.2f}")
            print(f"Rust最大余额: {rust_balances_numeric.max():,.2f}")
            
            print(f"Python最小余额: {python_balances_numeric.min():,.2f}")
            print(f"Rust最小余额: {rust_balances_numeric.min():,.2f}")
        
        # 检查顺序一致性 - 比较前后几行
        print(f"\n检查数据顺序一致性 (前10行余额):")
        for i in range(min(10, len(python_balances))):
            python_val = python_balances.iloc[i]
            rust_val = rust_balances.iloc[i]
            print(f"  行 {i+2}: Python={python_val}, Rust={rust_val}")
        
        print(f"\n检查数据顺序一致性 (后10行余额):")
        start_idx = max(0, len(python_balances) - 10)
        for i in range(start_idx, len(python_balances)):
            python_val = python_balances.iloc[i]
            rust_val = rust_balances.iloc[i]
            print(f"  行 {i+2}: Python={python_val}, Rust={rust_val}")
            
    except Exception as e:
        print(f"错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    compare_balance_columns()