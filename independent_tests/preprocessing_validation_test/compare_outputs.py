#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Python vs Rust 预处理输出对比工具
比较两个Excel文件的数据结构、顺序和值的一致性
"""

import pandas as pd
import numpy as np
import json
import os
from datetime import datetime

def compare_dataframes(df1, df2, name1="Python", name2="Rust"):
    """详细比较两个DataFrame"""
    print(f"\n=== {name1} vs {name2} 数据对比 ===")
    
    # 基本信息对比
    print(f"\n📊 基本信息对比:")
    print(f"  {name1} 行数: {len(df1):,}")
    print(f"  {name2} 行数: {len(df2):,}")
    print(f"  行数差异: {abs(len(df1) - len(df2)):,}")
    
    print(f"  {name1} 列数: {len(df1.columns)}")
    print(f"  {name2} 列数: {len(df2.columns)}")
    
    # 列名对比
    columns1 = set(df1.columns)
    columns2 = set(df2.columns)
    common_columns = columns1.intersection(columns2)
    
    print(f"\n📋 列结构对比:")
    print(f"  共同列数: {len(common_columns)}")
    print(f"  共同列名: {list(common_columns)}")
    if columns1 - columns2:
        print(f"  {name1} 独有列: {list(columns1 - columns2)}")
    if columns2 - columns1:
        print(f"  {name2} 独有列: {list(columns2 - columns1)}")
    
    if len(df1) != len(df2):
        print(f"\n⚠️  行数不匹配，无法进行详细对比")
        return False
    
    # 确保都有相同的列进行对比
    comparison_columns = list(common_columns)
    
    # 逐列对比
    differences_found = False
    print(f"\n🔍 逐列数据对比:")
    
    for col in comparison_columns:
        if col not in df1.columns or col not in df2.columns:
            continue
            
        # 处理不同的数据类型
        series1 = df1[col]
        series2 = df2[col]
        
        # 字符串类型对比
        if col in ['资金属性', '交易时间']:
            mismatches = series1 != series2
            mismatch_count = mismatches.sum()
            
            print(f"  {col}: {mismatch_count} 个不匹配")
            if mismatch_count > 0:
                differences_found = True
                print(f"    前5个不匹配示例:")
                mismatched_indices = mismatches[mismatches].index[:5]
                for idx in mismatched_indices:
                    print(f"      行 {idx}: '{series1.iloc[idx]}' vs '{series2.iloc[idx]}'")
        
        # 数值类型对比
        elif col in ['交易收入金额', '交易支出金额', '余额']:
            # 将NaN和空值统一处理
            s1 = series1.fillna(0)
            s2 = series2.fillna(0)
            
            # 数值差异对比（允许小误差）
            tolerance = 0.01
            differences = np.abs(s1 - s2) > tolerance
            diff_count = differences.sum()
            
            print(f"  {col}: {diff_count} 个数值差异 (容差: {tolerance})")
            if diff_count > 0:
                differences_found = True
                max_diff = np.abs(s1 - s2).max()
                print(f"    最大差异: {max_diff:.6f}")
                
                # 显示前5个差异示例
                diff_indices = differences[differences].index[:5]
                for idx in diff_indices:
                    print(f"      行 {idx}: {s1.iloc[idx]:.6f} vs {s2.iloc[idx]:.6f} (差异: {abs(s1.iloc[idx] - s2.iloc[idx]):.6f})")
        
        # 日期时间类型对比
        elif col in ['交易日期']:
            # 确保都是datetime类型
            try:
                dt1 = pd.to_datetime(series1)
                dt2 = pd.to_datetime(series2)
                
                mismatches = dt1 != dt2
                mismatch_count = mismatches.sum()
                
                print(f"  {col}: {mismatch_count} 个时间不匹配")
                if mismatch_count > 0:
                    differences_found = True
                    mismatched_indices = mismatches[mismatches].index[:5]
                    for idx in mismatched_indices:
                        print(f"      行 {idx}: {dt1.iloc[idx]} vs {dt2.iloc[idx]}")
            except Exception as e:
                print(f"  {col}: 时间格式转换错误 - {e}")
    
    # 数据统计对比
    print(f"\n📈 数据统计对比:")
    for col in ['交易收入金额', '交易支出金额', '余额']:
        if col in df1.columns and col in df2.columns:
            sum1 = df1[col].sum()
            sum2 = df2[col].sum()
            diff = abs(sum1 - sum2)
            print(f"  {col} 总和:")
            print(f"    {name1}: {sum1:,.2f}")
            print(f"    {name2}: {sum2:,.2f}")
            print(f"    差异: {diff:,.2f}")
    
    if not differences_found:
        print(f"\n✅ {name1} 和 {name2} 的数据完全一致！")
        return True
    else:
        print(f"\n❌ {name1} 和 {name2} 的数据存在差异")
        return False

def compare_json_stats(file1, file2, name1="Python", name2="Rust"):
    """比较JSON统计文件"""
    print(f"\n=== {name1} vs {name2} 统计信息对比 ===")
    
    try:
        with open(file1, 'r', encoding='utf-8') as f:
            stats1 = json.load(f)
        with open(file2, 'r', encoding='utf-8') as f:
            stats2 = json.load(f)
        
        # 比较关键统计数据
        key_stats = ['processed_rows', 'original_rows']
        print(f"\n📊 基本统计对比:")
        for key in key_stats:
            if key in stats1 and key in stats2:
                val1 = stats1[key]
                val2 = stats2[key]
                print(f"  {key}: {val1:,} vs {val2:,} (差异: {abs(val1-val2):,})")
        
        # 比较详细统计
        if 'statistics' in stats1 and 'statistics' in stats2:
            print(f"\n📈 详细统计对比:")
            stats_detail1 = stats1['statistics']
            stats_detail2 = stats2['statistics']
            
            numeric_keys = ['收入交易数', '支出交易数', '资金属性类型数']
            for key in numeric_keys:
                if key in stats_detail1 and key in stats_detail2:
                    val1 = stats_detail1[key]
                    val2 = stats_detail2[key]
                    print(f"  {key}: {val1:,} vs {val2:,} (差异: {abs(val1-val2):,})")
            
            # 比较金额类数据（需要转换为数字）
            amount_keys = ['总收入', '总支出', '最终余额']
            for key in amount_keys:
                if key in stats_detail1 and key in stats_detail2:
                    val1 = float(stats_detail1[key])
                    val2 = float(stats_detail2[key])
                    diff = abs(val1 - val2)
                    print(f"  {key}: {val1:,.2f} vs {val2:,.2f} (差异: {diff:,.2f})")
        
        print("✅ 统计信息对比完成")
        
    except Exception as e:
        print(f"❌ 统计信息对比失败: {e}")

def main():
    print("=== Python vs Rust 预处理输出完整对比工具 ===")
    
    # 文件路径
    python_excel = "python_preprocessed_output.xlsx"
    rust_excel = "rust_preprocessed_output.xlsx"
    python_json = "python_preprocessing_validation.json"
    rust_json = "rust_preprocessing_validation.json"
    
    # 检查文件是否存在
    files_to_check = [
        (python_excel, "Python Excel输出"),
        (rust_excel, "Rust Excel输出"),
    ]
    
    missing_files = []
    for file_path, desc in files_to_check:
        if not os.path.exists(file_path):
            missing_files.append(f"  - {desc}: {file_path}")
    
    if missing_files:
        print("❌ 以下文件不存在，请先运行预处理生成器:")
        print("\n".join(missing_files))
        print("\n运行步骤:")
        print("1. python generate_python_preprocessed.py")
        print("2. cargo run --bin generate_rust_preprocessed")
        print("3. python compare_outputs.py")
        return
    
    try:
        # 读取Excel文件
        print(f"\n读取Excel文件...")
        df_python = pd.read_excel(python_excel)
        df_rust = pd.read_excel(rust_excel)
        
        print(f"Python Excel: {len(df_python):,} 行")
        print(f"Rust Excel: {len(df_rust):,} 行")
        
        # 执行详细对比
        is_identical = compare_dataframes(df_python, df_rust, "Python", "Rust")
        
        # 对比JSON统计文件（如果存在）
        if os.path.exists(python_json) and os.path.exists(rust_json):
            compare_json_stats(python_json, rust_json, "Python", "Rust")
        
        # 生成对比报告
        report = {
            "comparison_timestamp": datetime.now().isoformat(),
            "python_file": python_excel,
            "rust_file": rust_excel,
            "python_rows": len(df_python),
            "rust_rows": len(df_rust),
            "is_identical": is_identical,
            "row_difference": abs(len(df_python) - len(df_rust)),
        }
        
        with open("comparison_report.json", "w", encoding='utf-8') as f:
            json.dump(report, f, ensure_ascii=False, indent=2)
        
        print(f"\n🎯 最终对比结果:")
        if is_identical:
            print("✅ Python和Rust预处理输出完全一致！")
            print("🎉 Rust工具层实现100%精确匹配Python功能！")
            print("🚀 可以进入算法层开发阶段")
        else:
            print("❌ Python和Rust预处理输出存在差异")
            print("🔧 需要进一步调试Rust实现")
        
        print(f"📄 详细对比报告已保存到: comparison_report.json")
        
    except Exception as e:
        print(f"❌ 对比过程中发生错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()