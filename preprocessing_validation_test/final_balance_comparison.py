#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
最终余额列对比工具 - 无emoji版本
Python[3]余额 vs Rust[4]余额
"""

import pandas as pd
import numpy as np

def final_balance_comparison():
    print("=== 最终余额列对比验证 ===")
    
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
            print(f"[错误] 行数不匹配! Python: {len(python_balances)}, Rust: {len(rust_balances)}")
            return
        else:
            print(f"[正确] 行数一致: {len(python_balances):,} 行")
        
        # 计算差异
        differences = np.abs(python_balances - rust_balances)
        tolerance = 0.01  # 允许0.01的误差
        
        significant_diffs = differences > tolerance
        diff_count = significant_diffs.sum()
        
        print(f"\n=== 余额数值对比结果 ===")
        print(f"超过容差({tolerance})的差异数量: {diff_count}")
        
        if diff_count == 0:
            print("[成功] 余额列数值完全一致!")
        else:
            print(f"[问题] 发现 {diff_count} 个显著差异")
            
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
        python_sum = python_balances.sum()
        rust_sum = rust_balances.sum()
        sum_diff = abs(python_sum - rust_sum)
        
        print(f"Python余额总和: {python_sum:,.2f}")
        print(f"Rust余额总和: {rust_sum:,.2f}")
        print(f"总和差异: {sum_diff:,.6f}")
        
        python_max = python_balances.max()
        rust_max = rust_balances.max()
        max_diff = abs(python_max - rust_max)
        
        print(f"Python最大余额: {python_max:,.2f}")
        print(f"Rust最大余额: {rust_max:,.2f}")
        print(f"最大值差异: {max_diff:,.6f}")
        
        python_min = python_balances.min()
        rust_min = rust_balances.min()
        min_diff = abs(python_min - rust_min)
        
        print(f"Python最小余额: {python_min:,.2f}")
        print(f"Rust最小余额: {rust_min:,.2f}")
        print(f"最小值差异: {min_diff:,.6f}")
        
        python_mean = python_balances.mean()
        rust_mean = rust_balances.mean()
        mean_diff = abs(python_mean - rust_mean)
        
        print(f"Python平均余额: {python_mean:,.2f}")
        print(f"Rust平均余额: {rust_mean:,.2f}")
        print(f"平均值差异: {mean_diff:,.6f}")
        
        # 检查顺序一致性 - 关键位置抽检
        print(f"\n=== 数据顺序一致性检查 ===")
        print("前5行余额对比:")
        for i in range(min(5, len(python_balances))):
            python_val = python_balances.iloc[i]
            rust_val = rust_balances.iloc[i]
            diff = abs(python_val - rust_val)
            status = "[匹配]" if diff <= tolerance else "[差异]"
            print(f"  行 {i+2}: Python={python_val:.2f}, Rust={rust_val:.2f}, 差异={diff:.6f} {status}")
        
        print(f"\n后5行余额对比:")
        start_idx = max(0, len(python_balances) - 5)
        for i in range(start_idx, len(python_balances)):
            python_val = python_balances.iloc[i]
            rust_val = rust_balances.iloc[i]
            diff = abs(python_val - rust_val)
            status = "[匹配]" if diff <= tolerance else "[差异]"
            print(f"  行 {i+2}: Python={python_val:.2f}, Rust={rust_val:.2f}, 差异={diff:.6f} {status}")
        
        # 随机抽检10行
        print(f"\n随机抽检10行:")
        np.random.seed(42)  # 固定随机种子确保可重现
        random_indices = np.random.choice(len(python_balances), size=10, replace=False)
        random_indices = sorted(random_indices)
        
        for idx in random_indices:
            python_val = python_balances.iloc[idx]
            rust_val = rust_balances.iloc[idx]
            diff = abs(python_val - rust_val)
            status = "[匹配]" if diff <= tolerance else "[差异]"
            print(f"  行 {idx+2}: Python={python_val:.2f}, Rust={rust_val:.2f}, 差异={diff:.6f} {status}")
        
        # 最终结论
        print(f"\n=== 最终验证结论 ===")
        
        # 检查所有关键指标
        all_checks = []
        all_checks.append(("行数一致", len(python_balances) == len(rust_balances)))
        all_checks.append(("数值差异", diff_count == 0))
        all_checks.append(("总和一致", sum_diff <= tolerance))
        all_checks.append(("最大值一致", max_diff <= tolerance))
        all_checks.append(("最小值一致", min_diff <= tolerance))
        all_checks.append(("平均值一致", mean_diff <= tolerance))
        
        all_passed = all([check[1] for check in all_checks])
        
        print("验证项目检查:")
        for check_name, passed in all_checks:
            status = "[通过]" if passed else "[失败]"
            print(f"  {check_name}: {status}")
        
        if all_passed:
            print("\n[重要结论]")
            print("Python和Rust的余额列数据完全一致!")
            print("数据顺序正确、数值精度一致、统计结果匹配")
            print("工具层验证完全通过!")
            print("可以进入算法层开发阶段!")
        else:
            print("\n[问题发现]")
            print("Python和Rust的余额列存在差异")
            print("需要进一步检查数据处理逻辑")
            
    except Exception as e:
        print(f"错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    final_balance_comparison()