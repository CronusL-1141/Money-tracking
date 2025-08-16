"""
数据验证模块
提供各种数据验证功能
"""

import pandas as pd
import numpy as np
from typing import List, Dict, Any, Tuple, Optional
from datetime import datetime
import re

from config import Config
from utils.logger import audit_logger


class DataValidator:
    """数据验证器"""
    
    @staticmethod
    def validate_dataframe(df: pd.DataFrame) -> Dict[str, Any]:
        """
        综合验证数据框
        
        Args:
            df: 数据框
            
        Returns:
            验证结果字典
        """
        results = {
            'is_valid': True,
            'errors': [],
            'warnings': [],
            'stats': {}
        }
        
        try:
            # 验证数据一致性
            consistency_result = DataValidator.validate_data_consistency(df)
            results['errors'].extend(consistency_result.get('errors', []))
            results['warnings'].extend(consistency_result.get('warnings', []))
            results['stats'].update(consistency_result.get('stats', {}))
            
            # 验证大额交易
            large_amounts_result = DataValidator.validate_large_amounts(df)
            if large_amounts_result.get('large_amounts'):
                results['stats']['large_amounts'] = large_amounts_result['large_amounts']
            
            # 验证日期范围
            date_ranges_result = DataValidator.validate_date_ranges(df)
            results['stats'].update(date_ranges_result)
            
            # 验证投资产品
            products_result = DataValidator.validate_investment_products(df)
            if products_result.get('product_errors'):
                results['errors'].extend(products_result['product_errors'])
            
            # 判断整体是否有效
            if results['errors']:
                results['is_valid'] = False
                
            audit_logger.info(f"数据验证完成：{len(results['errors'])}个错误，{len(results['warnings'])}个警告")
            
        except Exception as e:
            results['is_valid'] = False
            results['errors'].append(f"验证过程中出现异常: {str(e)}")
            audit_logger.error(f"数据验证异常: {e}")
        
        return results
    
    @staticmethod
    def validate_required_fields(row: pd.Series, required_fields: List[str]) -> List[str]:
        """验证必填字段"""
        missing_fields = []
        
        for field in required_fields:
            if field not in row.index:
                missing_fields.append(f"缺少字段: {field}")
                continue
            
            value = row[field]
            if pd.isna(value) or (isinstance(value, str) and value.strip() == ''):
                missing_fields.append(f"字段 {field} 为空")
        
        return missing_fields
    
    @staticmethod
    def validate_transaction_amounts(row: pd.Series) -> List[str]:
        """验证交易金额的有效性"""
        errors = []
        
        if '交易收入金额' in row.index and '交易支出金额' in row.index:
            收入金额 = row['交易收入金额']
            支出金额 = row['交易支出金额']
            
            # 检查是否有值（非空且非零）- 使用安全的方式
            收入有值 = False
            支出有值 = False
            
            # 检查收入金额
            if pd.notna(收入金额):
                try:
                    收入有值 = float(收入金额) != 0
                except (ValueError, TypeError):
                    收入有值 = False
            
            # 检查支出金额
            if pd.notna(支出金额):
                try:
                    支出有值 = float(支出金额) != 0
                except (ValueError, TypeError):
                    支出有值 = False
            
            # 验证互斥性
            if 收入有值 and 支出有值:
                errors.append("交易收入金额和交易支出金额不能同时有值")
        
        # 验证交易金额的存在性
        if '交易收入金额' not in row.index and '交易支出金额' not in row.index:
            errors.append("缺少交易金额字段")
        
        # 验证收入金额
        if '交易收入金额' in row.index:
            收入金额 = row['交易收入金额']
            if pd.notna(收入金额):
                try:
                    amount = float(收入金额)
                    if amount < 0:
                        errors.append("交易收入金额不能为负数")
                except (ValueError, TypeError):
                    errors.append("交易收入金额格式无效")
        
        # 验证支出金额
        if '交易支出金额' in row.index:
            支出金额 = row['交易支出金额']
            if pd.notna(支出金额):
                try:
                    amount = float(支出金额)
                    if amount < 0:
                        errors.append("交易支出金额不能为负数")
                except (ValueError, TypeError):
                    errors.append("交易支出金额格式无效")
        
        return errors
    
    @staticmethod
    def validate_date_format(row: pd.Series) -> List[str]:
        """验证日期格式"""
        errors = []
        
        if '交易日期' in row.index and pd.notna(row['交易日期']):
            date_value = row['交易日期']
            try:
                if isinstance(date_value, str):
                    # 尝试解析字符串日期
                    pd.to_datetime(date_value)
                elif not isinstance(date_value, (datetime, pd.Timestamp)):
                    errors.append("交易日期格式无效")
            except:
                errors.append("交易日期格式无效")
        
        return errors
    
    @staticmethod
    def validate_data_consistency(df: pd.DataFrame) -> Dict[str, Any]:
        """验证数据一致性"""
        results = {
            'errors': [],
            'warnings': [],
            'stats': {}
        }
        
        # 验证交易金额列的一致性
        if '交易收入金额' in df.columns and '交易支出金额' in df.columns:
            both_amounts_count = 0
            neither_amounts_count = 0
            收入交易数 = 0
            支出交易数 = 0
            
            for idx, row in df.iterrows():
                收入金额 = row['交易收入金额']
                支出金额 = row['交易支出金额']
                
                收入有值 = False
                支出有值 = False
                
                if pd.notna(收入金额):
                    try:
                        收入值 = float(收入金额)
                        if 收入值 != 0:
                            收入有值 = True
                            收入交易数 += 1
                    except (ValueError, TypeError):
                        pass
                
                if pd.notna(支出金额):
                    try:
                        支出值 = float(支出金额)
                        if 支出值 != 0:
                            支出有值 = True
                            支出交易数 += 1
                    except (ValueError, TypeError):
                        pass
                
                if 收入有值 and 支出有值:
                    both_amounts_count += 1
                
                if not 收入有值 and not 支出有值:
                    neither_amounts_count += 1
            
            results['stats']['收入交易数'] = 收入交易数
            results['stats']['支出交易数'] = 支出交易数
            results['stats']['总交易数'] = len(df)
            
            # 检查数据质量问题
            if both_amounts_count > 0:
                results['errors'].append(f"发现 {both_amounts_count} 行同时有收入和支出金额")
            
            if neither_amounts_count > 0:
                results['warnings'].append(f"发现 {neither_amounts_count} 行既无收入也无支出金额")
        
        # 基本字段验证
        basic_fields = ['交易日期', '资金属性', '余额']
        for field in basic_fields:
            if field in df.columns:
                null_count = df[field].isna().sum()
                if null_count > 0:
                    results['warnings'].append(f"字段 {field} 有 {null_count} 个缺失值")
        
        # 报告行级错误（只检查前100行避免过多错误）
        for idx, row in df.head(100).iterrows():
            row_errors = []
            row_errors.extend(DataValidator.validate_transaction_amounts(row))
            row_errors.extend(DataValidator.validate_date_format(row))
            
            if row_errors:
                results['errors'].append(f"第 {idx+1} 行: {'; '.join(row_errors)}")
        
        return results
    
    @staticmethod
    def validate_large_amounts(df: pd.DataFrame) -> Dict[str, Any]:
        """验证大额交易"""
        large_amounts = []
        
        for amount_col in ['交易收入金额', '交易支出金额']:
            if amount_col in df.columns:
                large_amounts_count = 0
                
                for idx, amount in enumerate(df[amount_col]):
                    if pd.notna(amount):
                        try:
                            if float(amount) > Config.LARGE_AMOUNT_THRESHOLD:
                                large_amounts.append({
                                    '行号': idx + 1,
                                    '类型': amount_col,
                                    '金额': float(amount)
                                })
                                large_amounts_count += 1
                        except (ValueError, TypeError):
                            pass
                
                if large_amounts_count > 0:
                    audit_logger.info(f"发现 {large_amounts_count} 笔大额{amount_col.replace('交易', '')}")
        
        return {'large_amounts': large_amounts}
    
    @staticmethod
    def validate_date_ranges(df: pd.DataFrame) -> Dict[str, Any]:
        """验证日期范围"""
        date_info = {}
        
        if '交易日期' in df.columns:
            有效日期 = []
            无效日期数 = 0
            
            for date_val in df['交易日期']:
                if pd.notna(date_val):
                    try:
                        parsed_date = pd.to_datetime(date_val)
                        有效日期.append(parsed_date)
                    except:
                        无效日期数 += 1
                else:
                    无效日期数 += 1
            
            if len(有效日期) > 0:
                date_info['最早日期'] = min(有效日期)
                date_info['最晚日期'] = max(有效日期)
                date_info['日期范围天数'] = (max(有效日期) - min(有效日期)).days
                date_info['有效日期数'] = len(有效日期)
                
            if 无效日期数 > 0:
                date_info['无效日期数'] = 无效日期数
                audit_logger.warning(f"发现 {无效日期数} 个无效日期")
        
        return date_info
    
    @staticmethod
    def validate_balance_consistency(df: pd.DataFrame, tracker_balances: List[float]) -> Dict[str, Any]:
        """验证余额一致性"""
        balance_errors = []
        
        if '余额' not in df.columns:
            return {'errors': ['缺少余额列']}
        
        # 检查追踪器余额数量是否匹配
        if len(tracker_balances) != len(df):
            return {'errors': [f'追踪器余额数量({len(tracker_balances)})与数据行数({len(df)})不匹配']}
        
        for idx, (expected_balance, tracker_balance) in enumerate(zip(df['余额'], tracker_balances)):
            if pd.isna(expected_balance):
                continue
            
            try:
                expected_balance = float(expected_balance)
                difference = abs(tracker_balance - expected_balance)
                if difference > Config.BALANCE_TOLERANCE:
                    balance_errors.append({
                        '行号': idx + 1,
                        '期望余额': expected_balance,
                        '追踪器余额': tracker_balance,
                        '差异': difference
                    })
            except (ValueError, TypeError):
                balance_errors.append({
                    '行号': idx + 1,
                    '错误': '余额格式无效'
                })
        
        return {
            'balance_errors': balance_errors,
            'total_errors': len(balance_errors)
        }
    
    @staticmethod
    def validate_investment_products(df: pd.DataFrame) -> Dict[str, Any]:
        """验证投资产品数据"""
        product_errors = []
        
        # 注意：总金额、个人金额、公司金额等字段是在投资产品处理时动态计算的
        # 在数据预处理阶段不需要验证这些字段的存在性
        # 这里只验证投资产品的资金属性格式是否正确
        
        for idx, row in df.iterrows():
            资金属性 = row.get('资金属性', '')
            if Config.is_investment_product(str(资金属性)):
                # 验证投资产品资金属性格式
                if not isinstance(资金属性, str) or '-' not in 资金属性:
                    product_errors.append(f"第{idx+1}行投资产品资金属性格式无效: {资金属性}")
                
                # 可以添加其他业务逻辑验证，但不验证动态计算字段的存在性
        
        return {
            'product_errors': product_errors,
            'total_errors': len(product_errors)
        } 