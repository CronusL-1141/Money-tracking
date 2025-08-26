#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Python版本完整预处理输出生成器
直接调用现有的Python模块，获取预处理后的数据
"""

import sys
import os
import pandas as pd
import json
from datetime import datetime

# 添加上级目录的src到Python路径
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from utils.data_processor import DataProcessor
from utils.flow_integrity_validator import FlowIntegrityValidator

def main():
    print("=== Python版本完整预处理输出生成器 ===")
    
    input_file = "../流水.xlsx"
    output_file = "python_preprocessed_output.xlsx"
    validation_output = "python_preprocessing_validation.json"
    
    try:
        # 1. 数据预处理 - 直接调用现有模块
        print("\n第一步: 数据预处理")
        data_processor = DataProcessor()
        df = data_processor.预处理财务数据(input_file)
        
        if df is None:
            print("错误 数据预处理失败")
            return
        
        print(f"成功 数据预处理完成，共加载 {len(df):,} 条记录")
        original_rows = len(df)
        
        # 2. 流水完整性验证 - 直接调用现有模块  
        print("\n第二步: 流水完整性验证")
        flow_validator = FlowIntegrityValidator()
        validation_result = flow_validator.validate_flow_integrity(df)
        
        print(f"验证结果:")
        print(f"  - 验证状态: {'通过' if validation_result['is_valid'] else '发现问题'}")
        print(f"  - 发现错误数: {validation_result['errors_count']}")
        
        # 获取处理后的数据 (如果有修复的话)
        processed_df = validation_result.get('processed_data', df)
        
        # 打印列名以便调试
        print(f"DataFrame列名: {list(processed_df.columns)}")
        
        # 3. 保存处理后的Excel文件
        print(f"\n第三步: 保存处理后的Excel文件到 {output_file}")
        processed_df.to_excel(output_file, index=False)
        print("成功 Excel文件保存完成")
        
        # 4. 生成数据统计信息
        print("\n第四步: 生成数据统计信息")
        
        # 使用实际的列名（处理编码问题）
        columns = processed_df.columns.tolist()
        date_col = columns[0]  # 第一列应该是日期
        income_col = columns[2]  # 第三列是收入
        expense_col = columns[3]  # 第四列是支出
        balance_col = columns[4]  # 第五列是余额
        fund_attr_col = columns[5]  # 第六列是资金属性
        
        stats = {
            "preprocessing_timestamp": datetime.now().isoformat(),
            "input_file": input_file,
            "output_file": output_file,
            "original_rows": original_rows,
            "processed_rows": len(processed_df),
            "statistics": {
                "收入交易数": len(processed_df[processed_df[income_col] > 0]),
                "支出交易数": len(processed_df[processed_df[expense_col] > 0]),
                "总收入": float(processed_df[income_col].sum()),
                "总支出": float(processed_df[expense_col].sum()),
                "最终余额": float(processed_df[balance_col].iloc[-1]),
                "资金属性类型数": len(processed_df[fund_attr_col].unique()),
                "资金属性列表": sorted(processed_df[fund_attr_col].unique().tolist()),
                "时间范围开始": processed_df[date_col].min().isoformat(),
                "时间范围结束": processed_df[date_col].max().isoformat(),
                "columns": columns,  # 保存实际列名
            },
            "validation_result": validation_result,
        }
        
        # 保存验证信息
        with open(validation_output, 'w', encoding='utf-8') as f:
            json.dump(stats, f, ensure_ascii=False, indent=2)
        
        print(f"成功 验证信息保存到 {validation_output}")
        
        print(f"\nPython版本预处理完成！")
        print(f"处理后的Excel文件: {output_file}")
        print(f"验证信息: {validation_output}")
        
        # 显示关键统计信息
        print(f"\n关键统计信息:")
        print(f"  - 处理数据行数: {stats['processed_rows']:,}")
        print(f"  - 收入交易: {stats['statistics']['收入交易数']:,} 笔")
        print(f"  - 支出交易: {stats['statistics']['支出交易数']:,} 笔")
        print(f"  - 总收入: {stats['statistics']['总收入']:,.2f}")
        print(f"  - 总支出: {stats['statistics']['总支出']:,.2f}")
        print(f"  - 最终余额: {stats['statistics']['最终余额']:,.2f}")
        print(f"  - 资金属性类型: {stats['statistics']['资金属性类型数']} 种")
        
    except Exception as e:
        print(f"错误 处理过程中发生错误: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()