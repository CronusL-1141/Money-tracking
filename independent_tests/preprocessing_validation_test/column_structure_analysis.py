#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
列结构分析工具 - 找到正确的余额列
"""

import pandas as pd
import numpy as np

def analyze_column_structure():
    print("=== 列结构详细分析 ===")
    
    try:
        # 读取两个Excel文件
        df_python = pd.read_excel("python_preprocessed_output.xlsx")
        df_rust = pd.read_excel("rust_preprocessed_output.xlsx")
        
        print(f"\n=== Python Excel结构分析 ===")
        print(f"总行数: {len(df_python):,}")
        print(f"总列数: {len(df_python.columns)}")
        print("列结构:")
        for i, col in enumerate(df_python.columns):
            sample_values = df_python[col].head(3).tolist()
            data_type = df_python[col].dtype
            print(f"  [{i}] {col} ({data_type}) - 示例: {sample_values}")
        
        print(f"\n=== Rust Excel结构分析 ===")
        print(f"总行数: {len(df_rust):,}")
        print(f"总列数: {len(df_rust.columns)}")
        print("列结构:")
        for i, col in enumerate(df_rust.columns):
            sample_values = df_rust[col].head(3).tolist()
            data_type = df_rust[col].dtype
            print(f"  [{i}] {col} ({data_type}) - 示例: {sample_values}")
        
        # 寻找数值型列（可能的余额列）
        print(f"\n=== Python数值型列分析 ===")
        numeric_cols_python = df_python.select_dtypes(include=[np.number]).columns
        for col in numeric_cols_python:
            print(f"  {col}: 最小值={df_python[col].min():.2f}, 最大值={df_python[col].max():.2f}, 平均值={df_python[col].mean():.2f}")
        
        print(f"\n=== Rust数值型列分析 ===")
        numeric_cols_rust = df_rust.select_dtypes(include=[np.number]).columns
        for col in numeric_cols_rust:
            print(f"  {col}: 最小值={df_rust[col].min():.2f}, 最大值={df_rust[col].max():.2f}, 平均值={df_rust[col].mean():.2f}")
        
        # 根据列名和位置猜测余额列
        print(f"\n=== 余额列推断 ===")
        
        # 在Python中寻找包含"余额"的列
        python_balance_candidates = [col for col in df_python.columns if "余额" in col or "余" in col]
        print(f"Python可能的余额列: {python_balance_candidates}")
        
        # 在Rust中寻找包含"余额"的列  
        rust_balance_candidates = [col for col in df_rust.columns if "余额" in col or "余" in col]
        print(f"Rust可能的余额列: {rust_balance_candidates}")
        
        # 手动检查第5列（通常是余额列）
        if len(df_python.columns) >= 5:
            python_col4 = df_python.columns[4]
            print(f"Python第5列: {python_col4}")
            print(f"  示例值: {df_python[python_col4].head(5).tolist()}")
            print(f"  数据类型: {df_python[python_col4].dtype}")
        
        if len(df_rust.columns) >= 5:
            rust_col4 = df_rust.columns[4]
            print(f"Rust第5列: {rust_col4}")
            print(f"  示例值: {df_rust[rust_col4].head(5).tolist()}")
            print(f"  数据类型: {df_rust[rust_col4].dtype}")
            
    except Exception as e:
        print(f"错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    analyze_column_structure()