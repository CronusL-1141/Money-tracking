"""
原始流水完整性验证模块
专门用于验证银行流水数据的基础连贯性，不涉及FIFO业务逻辑
"""

import pandas as pd
import itertools
from typing import List, Tuple, Optional, Dict
from utils.logger import audit_logger
from config import Config


class FlowIntegrityValidator:
    """原始流水完整性验证器"""
    
    def __init__(self):
        """初始化验证器"""
        self.tolerance = 0.01  # 余额容差
        self.validation_errors = []
        self.optimization_count = 0
        self.optimization_failed = False
        
    def validate_flow_integrity(self, df: pd.DataFrame) -> Dict:
        """
        验证整个流水的完整性（不修改源数据）
        
        Args:
            df: 原始流水数据框（只读）
            
        Returns:
            验证结果字典，包含修复后的数据框（如果有修复的话）
        """
        audit_logger.info("开始原始流水完整性验证...")
        
        self.validation_errors = []
        self.optimization_count = 0
        self.optimization_failed = False
        
        # 创建副本用于验证和修复，保持源文件不变
        result_df = df.copy()
        
        # 逐行验证余额连贯性
        for i in range(1, len(result_df)):  # 从第二行开始
            current_row = result_df.iloc[i]
            previous_row = result_df.iloc[i-1]
            
            if not self._check_balance_continuity(previous_row, current_row, i):
                # 余额不连贯，尝试修复
                fixed_df = self._attempt_reorder_fix(result_df, i)
                if fixed_df is not None:
                    audit_logger.info(f"✅ 第{i+1}行余额问题已通过重排序修复")
                    self.optimization_count += 1
                    result_df = fixed_df  # 使用修复后的数据框
                    # 重新验证修复后的行
                    current_row = result_df.iloc[i]
                    previous_row = result_df.iloc[i-1]
                    if not self._check_balance_continuity(previous_row, current_row, i):
                        self._record_error(i, "重排序后仍无法修复余额连贯性")
                        self.optimization_failed = True
                        break  # 停止优化
                else:
                    self._record_error(i, "余额不连贯且无法通过重排序修复，可能存在数据丢失")
                    self.optimization_failed = True
                    break  # 停止优化并提示用户
        
        # 生成验证报告
        return self._generate_validation_report(result_df, df)
    
    def _check_balance_continuity(self, previous_row: pd.Series, current_row: pd.Series, row_idx: int) -> bool:
        """
        检查两行之间的余额连贯性（使用完整时间戳验证）
        
        Args:
            previous_row: 上一行数据
            current_row: 当前行数据
            row_idx: 当前行索引
            
        Returns:
            是否连贯
        """
        try:
            # 获取完整时间戳进行比较
            prev_timestamp = previous_row['完整时间戳']
            curr_timestamp = current_row['完整时间戳']
            
            # 获取余额
            prev_balance = self._safe_get_balance(previous_row)
            curr_balance = self._safe_get_balance(current_row)
            
            # 获取交易金额
            income = self._safe_get_amount(current_row, '交易收入金额')
            expense = self._safe_get_amount(current_row, '交易支出金额')
            
            # 计算期望余额：上一笔余额 + 收入 - 支出
            expected_balance = prev_balance + income - expense
            
            # 检查是否在容差范围内
            if abs(curr_balance - expected_balance) <= self.tolerance:
                return True
            else:
                audit_logger.warning(
                    f"第{row_idx+1}行余额不连贯: "
                    f"时间戳: {curr_timestamp}, "
                    f"上笔余额{prev_balance:,.2f} + 收入{income:,.2f} - 支出{expense:,.2f} "
                    f"= 期望{expected_balance:,.2f}, 实际{curr_balance:,.2f}, "
                    f"差异{curr_balance - expected_balance:,.2f}"
                )
                return False
                
        except Exception as e:
            audit_logger.error(f"检查第{row_idx+1}行余额连贯性时出错: {e}")
            return False
    
    def _safe_get_balance(self, row: pd.Series) -> float:
        """安全获取余额值"""
        try:
            balance_str = str(row['余额'])
            if balance_str in ['nan', 'None', '', 'NaN']:
                return 0.0
            return float(balance_str)
        except (ValueError, TypeError, KeyError):
            return 0.0
    
    def _safe_get_amount(self, row: pd.Series, column: str) -> float:
        """安全获取金额值"""
        try:
            amount_str = str(row[column])
            if amount_str in ['nan', 'None', '', 'NaN']:
                return 0.0
            return float(amount_str)
        except (ValueError, TypeError, KeyError):
            return 0.0
    
    def _attempt_reorder_fix(self, df: pd.DataFrame, problem_row_idx: int) -> Optional[pd.DataFrame]:
        """
        尝试通过重新排序同时间交易来修复余额问题（返回新的数据框，不修改原数据）
        
        Args:
            df: 数据框（不会被修改）
            problem_row_idx: 出现问题的行索引
            
        Returns:
            修复后的新数据框，如果修复失败则返回None
        """
        try:
            # 使用完整时间戳查找同时间交易
            current_timestamp = df.iloc[problem_row_idx]['完整时间戳']
            
            # 找出所有同完整时间戳的交易
            same_time_mask = df['完整时间戳'] == current_timestamp
            same_time_indices = df[same_time_mask].index.tolist()
            
            if len(same_time_indices) <= 1:
                audit_logger.info(f"第{problem_row_idx+1}行无同时间交易，无法重排序修复")
                return None
            
            audit_logger.info(f"发现第{problem_row_idx+1}行有{len(same_time_indices)}笔同完整时间戳交易，尝试重排序...")
            audit_logger.info(f"完整时间戳: {current_timestamp}")
            
            # 尝试不同的排列
            best_order = self._find_best_order(df, same_time_indices, problem_row_idx)
            
            if best_order and best_order != same_time_indices:
                # 创建修复后的新数据框
                fixed_df = self._create_reordered_dataframe(df, same_time_indices, best_order)
                audit_logger.info(f"✅ 成功重排序: {[i+1 for i in same_time_indices]} → {[i+1 for i in best_order]}")
                return fixed_df
            else:
                audit_logger.warning(f"❌ 未找到有效的重排序方案")
                return None
                
        except Exception as e:
            audit_logger.error(f"重排序修复时出错: {e}")
            return None
    
    def _find_best_order(self, df: pd.DataFrame, indices: List[int], problem_idx: int) -> Optional[List[int]]:
        """
        找到最佳的交易排序（使用贪心策略）
        
        Args:
            df: 数据框
            indices: 同时间交易索引列表
            problem_idx: 问题行索引
            
        Returns:
            最佳排序，如果找不到则返回None
        """
        try:
            audit_logger.info(f"使用贪心策略寻找正确顺序，共{len(indices)}笔同时间交易...")
            
            # 使用贪心策略逐步构建正确顺序
            result_order = self._greedy_order_search(df, indices)
            
            if result_order:
                audit_logger.info(f"✅ 贪心策略找到正确顺序")
                return result_order
            else:
                audit_logger.warning(f"❌ 贪心策略未找到有效顺序")
                return None
            
        except Exception as e:
            audit_logger.error(f"查找最佳排序时出错: {e}")
            return None
    
    def _greedy_order_search(self, df: pd.DataFrame, indices: List[int]) -> Optional[List[int]]:
        """
        使用贪心策略寻找正确的交易顺序
        
        Args:
            df: 数据框
            indices: 同时间交易索引列表
            
        Returns:
            正确的顺序，如果找不到则返回None
        """
        if len(indices) <= 1:
            return indices
        
        # 获取排序后的位置信息
        sorted_positions = sorted(indices)
        min_pos = min(sorted_positions)
        
        # 如果第一个位置是索引0，无法获取前一行，直接尝试原顺序
        if min_pos == 0:
            if self._test_order_validity(df, indices, min_pos):
                return indices
            else:
                return None
        
        # 获取前一行数据（用于计算期望余额）
        prev_row = df.iloc[min_pos - 1]
        prev_balance = self._safe_get_balance(prev_row)
        
        result_order = []
        remaining_indices = indices.copy()
        current_balance = prev_balance
        
        # 逐步构建正确顺序
        for position in range(len(indices)):
            found_next = False
            
            audit_logger.info(f"  寻找第{position+1}笔交易，当前余额: {current_balance:.2f}")
            
            # 在剩余交易中找到下一笔符合余额连贯性的交易
            for i, candidate_idx in enumerate(remaining_indices):
                candidate_row = df.iloc[candidate_idx]
                
                # 计算使用这笔交易后的期望余额
                income = self._safe_get_amount(candidate_row, '交易收入金额')
                expense = self._safe_get_amount(candidate_row, '交易支出金额')
                expected_balance = current_balance + income - expense
                actual_balance = self._safe_get_balance(candidate_row)
                
                # 检查是否符合余额连贯性
                if abs(actual_balance - expected_balance) <= self.tolerance:
                    # 找到符合的交易
                    result_order.append(candidate_idx)
                    remaining_indices.pop(i)
                    current_balance = actual_balance
                    found_next = True
                    
                    audit_logger.info(f"    ✅ 找到第{position+1}笔: 第{candidate_idx+1}行, "
                                    f"收入{income:.2f}, 支出{expense:.2f}, 余额{actual_balance:.2f}")
                    break
                else:
                    audit_logger.debug(f"    ❌ 第{candidate_idx+1}行不符合: "
                                     f"期望{expected_balance:.2f}, 实际{actual_balance:.2f}, "
                                     f"差异{actual_balance - expected_balance:.2f}")
            
            if not found_next:
                audit_logger.warning(f"  ❌ 无法找到第{position+1}笔符合余额连贯性的交易")
                return None
        
        audit_logger.info(f"✅ 贪心策略成功找到完整顺序: {[idx+1 for idx in result_order]}")
        return result_order
    
    def _test_order_validity(self, df: pd.DataFrame, order: List[int], problem_idx: int) -> bool:
        """
        测试特定排序是否能解决余额连贯性问题
        
        Args:
            df: 数据框
            order: 要测试的排序
            problem_idx: 问题行索引
            
        Returns:
            是否有效
        """
        try:
            # 创建临时数据框来测试
            temp_df = self._create_reordered_dataframe(df, sorted(order), order)
            
            # 重新计算这些行的余额连贯性
            min_idx = min(order)
            max_idx = max(order)
            
            # 检查从第一个同时间交易到最后一个的余额连贯性
            for i in range(min_idx, max_idx + 1):
                if i == 0:  # 第一行跳过
                    continue
                    
                if not self._check_balance_continuity(temp_df.iloc[i-1], temp_df.iloc[i], i):
                    return False
            
            # 如果所有检查都通过，还要检查最后一笔交易之后的连贯性
            if max_idx + 1 < len(temp_df):
                if not self._check_balance_continuity(temp_df.iloc[max_idx], temp_df.iloc[max_idx + 1], max_idx + 1):
                    return False
            
            return True
            
        except Exception as e:
            audit_logger.error(f"测试排序有效性时出错: {e}")
            return False
    
    def _create_reordered_dataframe(self, df: pd.DataFrame, original_indices: List[int], new_order: List[int]) -> pd.DataFrame:
        """
        创建重新排序后的新数据框（不修改原数据框）
        
        Args:
            df: 原始数据框
            original_indices: 原始索引位置
            new_order: 新的排序
            
        Returns:
            重新排序后的新数据框
        """
        try:
            # 创建原数据框的副本
            result_df = df.copy()
            
            # 收集新排序的数据
            reordered_data = []
            for idx in new_order:
                reordered_data.append(df.iloc[idx].copy())
            
            # 按原始位置更新数据
            sorted_positions = sorted(original_indices)
            for i, new_data in enumerate(reordered_data):
                result_df.iloc[sorted_positions[i]] = new_data
                
            return result_df
                
        except Exception as e:
            audit_logger.error(f"创建重排序数据框时出错: {e}")
            return df.copy()
    
    def _record_error(self, row_idx: int, message: str) -> None:
        """记录验证错误"""
        error_info = {
            'row': row_idx + 1,
            'message': message,
            'timestamp': pd.Timestamp.now()
        }
        self.validation_errors.append(error_info)
        audit_logger.error(f"第{row_idx+1}行: {message}")
    
    def _generate_validation_report(self, result_df: pd.DataFrame, original_df: pd.DataFrame) -> Dict:
        """生成验证报告"""
        report = {
            'total_rows': len(original_df),
            'errors_count': len(self.validation_errors),
            'optimizations_count': self.optimization_count,
            'optimization_failed': self.optimization_failed,
            'is_valid': len(self.validation_errors) == 0,
            'errors': self.validation_errors,
            'result_dataframe': result_df if self.optimization_count > 0 else original_df,
            'has_modifications': self.optimization_count > 0,
            'summary': f"验证完成: {len(original_df)}行数据, {len(self.validation_errors)}个错误, {self.optimization_count}次重排序修复"
        }
        
        # 输出摘要
        audit_logger.info("=" * 60)
        audit_logger.info("原始流水完整性验证报告")
        audit_logger.info("=" * 60)
        audit_logger.info(f"总行数: {report['total_rows']}")
        audit_logger.info(f"发现错误: {report['errors_count']}个")
        audit_logger.info(f"成功修复: {report['optimizations_count']}个")
        
        if self.optimization_failed:
            audit_logger.warning("⚠️  优化失败 - 源文件保持不变")
            audit_logger.warning("建议：请检查数据完整性，可能存在缺失交易或数据错误")
            audit_logger.warning("解决方案：")
            audit_logger.warning("1. 检查银行流水数据是否完整")
            audit_logger.warning("2. 确认是否有遗漏的交易记录")
            audit_logger.warning("3. 验证余额计算是否正确")
        
        audit_logger.info(f"验证结果: {'✅ 通过' if report['is_valid'] else '❌ 失败'}")
        audit_logger.info(f"数据修改: {'是' if report['has_modifications'] else '否（源文件保持只读）'}")
        
        if self.validation_errors:
            audit_logger.info("\n错误详情:")
            for error in self.validation_errors:
                audit_logger.error(f"  第{error['row']}行: {error['message']}")
        
        audit_logger.info("=" * 60)
        
        return report 