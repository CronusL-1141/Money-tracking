#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
正确的余额列对比工具
Python[3]余额 vs Rust[4]余额
"""

import pandas as pd
import numpy as np

def correct_balance_comparison():
    print("=== 正确的余额列对比 ===")
    
    try:
        # 读取两个Excel文件
        df_python = pd.read_excel("python_preprocessed_output.xlsx")
        df_rust = pd.read_excel("rust_preprocessed_output.xlsx")
        
        print(f"Python Excel: {len(df_python):,} 行")
        print(f"Rust Excel: {len(df_rust):,} 行")
        
        # 正确的余额列
        python_balance_col = df_python.columns[3]  # 第4列：余额
        rust_balance_col = df_rust.columns[4]      # 第5列：余额
        
        print(f"\nPython余额列: [{3}] {python_balance_col}")
        print(f"Rust余额列: [{4}] {rust_balance_col}")
        
        # 提取余额列数据
        python_balances = df_python[python_balance_col]
        rust_balances = df_rust[rust_balance_col]
        
        print(f"\nPython余额列数据类型: {python_balances.dtype}")
        print(f"Rust余额列数据类型: {rust_balances.dtype}")
        
        # 检查数据长度
        if len(python_balances) != len(rust_balances):
            print(f"❌ 行数不匹配! Python: {len(python_balances)}, Rust: {len(rust_balances)}")
            return
        else:
            print(f"行数一致: {len(python_balances):,} 行")
        
        # 计算差异
        differences = np.abs(python_balances - rust_balances)
        tolerance = 0.01  # 允许0.01的误差
        
        significant_diffs = differences > tolerance
        diff_count = significant_diffs.sum()
        
        print(f"\n=== 余额数值对比结果 ===")
        print(f"超过容差({tolerance})的差异数量: {diff_count}")
        
        if diff_count == 0:
            print("余额列数值完全一致!")
        else:
            print(f"发现 {diff_count} 个显著差异")
            
            # 显示前10个差异
            diff_indices = significant_diffs[significant_diffs].index[:10]
            print("\n前10个显著差异:")
            for idx in diff_indices:
                python_val = python_balances.iloc[idx]
                rust_val = rust_balances.iloc[idx]
                diff = abs(python_val - rust_val)
                print(f"  行 {idx+2}: Python={python_val:.6f}, Rust={rust_val:.6f}, 差异={diff:.6f}")
        
        # 统计信息对比
        print(f"\n=== 统计信息对比 ===")
        print(f"Python余额总和: {python_balances.sum():,.2f}")
        print(f"Rust余额总和: {rust_balances.sum():,.2f}")
        print(f"总和差异: {abs(python_balances.sum() - rust_balances.sum()):,.6f}")
        
        print(f"Python最大余额: {python_balances.max():,.2f}")
        print(f"Rust最大余额: {rust_balances.max():,.2f}")
        print(f"最大值差异: {abs(python_balances.max() - rust_balances.max()):,.6f}")
        
        print(f"Python最小余额: {python_balances.min():,.2f}")
        print(f"Rust最小余额: {rust_balances.min():,.2f}")
        print(f"最小值差异: {abs(python_balances.min() - rust_balances.min()):,.6f}")
        
        print(f"Python平均余额: {python_balances.mean():,.2f}")
        print(f"Rust平均余额: {rust_balances.mean():,.2f}")
        print(f"平均值差异: {abs(python_balances.mean() - rust_balances.mean()):,.6f}")
        
        # 检查顺序一致性 - 比较关键位置
        print(f"\n=== 数据顺序一致性检查 ===")
        print(f"前10行余额对比:")
        for i in range(min(10, len(python_balances))):
            python_val = python_balances.iloc[i]
            rust_val = rust_balances.iloc[i]
            diff = abs(python_val - rust_val)
            status = "✅" if diff <= tolerance else "❌"
            print(f"  行 {i+2}: Python={python_val:.2f}, Rust={rust_val:.2f}, 差异={diff:.6f} {status}")
        
        print(f"\n中间10行余额对比 (第{len(python_balances)//2-4}到{len(python_balances)//2+5}行):")
        mid_start = len(python_balances) // 2 - 5
        mid_end = len(python_balances) // 2 + 5
        for i in range(mid_start, mid_end):
            python_val = python_balances.iloc[i]
            rust_val = rust_balances.iloc[i]
            diff = abs(python_val - rust_val)
            status = "✅" if diff <= tolerance else "❌"
            print(f"  行 {i+2}: Python={python_val:.2f}, Rust={rust_val:.2f}, 差异={diff:.6f} {status}")
        
        print(f"\n后10行余额对比:")
        start_idx = max(0, len(python_balances) - 10)
        for i in range(start_idx, len(python_balances)):
            python_val = python_balances.iloc[i]
            rust_val = rust_balances.iloc[i]
            diff = abs(python_val - rust_val)
            status = "✅" if diff <= tolerance else "❌"
            print(f"  行 {i+2}: Python={python_val:.2f}, Rust={rust_val:.2f}, 差异={diff:.6f} {status}")
        
        # 最终结论
        print(f"\n=== 最终验证结论 ===")
        if diff_count == 0:
            print("🎉 Python和Rust的余额列数据完全一致!")
            print("✅ 数据顺序正确")
            print("✅ 数值精度一致") 
            print("✅ 统计结果一致")
            print("🚀 工具层验证通过，可以进入算法层开发!")
        else:
            print("⚠️ Python和Rust的余额列存在差异")
            print("❌ 需要进一步检查数据处理逻辑")
            
    except Exception as e:
        print(f"错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    correct_balance_comparison()