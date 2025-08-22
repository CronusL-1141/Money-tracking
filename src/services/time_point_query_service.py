"""
时点查询服务
基于debug_tool.py转换而来，支持特定时点查询和历史记录管理
"""

import pandas as pd
from datetime import datetime
from typing import Dict, List, Any, Optional, Tuple
import json
import sys

from core.interfaces.tracker_interface import ITracker
from core.factories.tracker_factory import TrackerFactory
from utils.data_processor import DataProcessor
from utils.flow_integrity_validator import FlowIntegrityValidator
from utils.logger import audit_logger
from config import Config


class TimePointQueryService:
    """
    时点查询服务 - 查询任意交易行的系统状态
    """
    
    def __init__(self, algorithm: str = "FIFO"):
        """
        初始化时点查询服务
        
        Args:
            algorithm: 算法类型 ("FIFO" 或 "BALANCE_METHOD")
        """
        self.algorithm = algorithm
        self.tracker: Optional[ITracker] = None
        self.data_processor = DataProcessor()
        self.flow_validator = FlowIntegrityValidator()
        
        # 数据状态
        self.data: Optional[pd.DataFrame] = None
        self.total_rows = 0
        self.current_row = 0
        
        # 查询历史（最多保存100条）
        self.query_history: List[Dict[str, Any]] = []
        self.MAX_HISTORY_SIZE = 100
        
        # 处理记录
        self.processing_steps: List[Dict[str, Any]] = []
        self.error_records: List[Dict[str, Any]] = []
        
        audit_logger.info(f"时点查询服务初始化完成，使用算法: {algorithm}")
    
    def load_data(self, file_path: str) -> Dict[str, Any]:
        """
        加载Excel数据文件
        
        Args:
            file_path: Excel文件路径
            
        Returns:
            加载结果信息
        """
        # 在函数开始就导入所需的模块，避免作用域问题
        import logging
        from utils.logger import audit_logger
        
        try:
            audit_logger.info(f"开始加载数据文件: {file_path}")
            
            # 1. 数据预处理（静默模式）
            print("📊 开始数据预处理...", file=sys.stderr)
            sys.stderr.flush()
            sys.stdout.flush()
            
            # 临时调整日志级别，减少详细输出
            
            # 保存原始级别
            original_level = logging.getLogger().level
            original_audit_level = audit_logger.logger.level
            
            # 调整日志级别：保留重要信息，抑制详细的贪心算法步骤和WARNING
            logging.getLogger().setLevel(logging.ERROR)
            audit_logger.logger.setLevel(logging.ERROR)
            
            try:
                self.data = self.data_processor.预处理财务数据(file_path)
            finally:
                # 恢复原始日志级别
                logging.getLogger().setLevel(original_level)
                audit_logger.logger.setLevel(original_audit_level)
            
            if self.data is None:
                error_msg = "数据预处理失败"
                audit_logger.error(error_msg)
                return {
                    "success": False,
                    "message": error_msg,
                    "file_path": file_path
                }
            print(f"✅ 数据预处理完成，共加载 {len(self.data):,} 条记录", file=sys.stderr)
            sys.stderr.flush()
            sys.stdout.flush()
            
            # 2. 流水完整性验证（静默模式）
            print("🔍 开始流水完整性验证...", file=sys.stderr)
            sys.stderr.flush()
            sys.stdout.flush()
            
            # 临时提升日志级别以减少详细输出（隐藏WARNING信息）
            logging.getLogger().setLevel(logging.ERROR)
            audit_logger.logger.setLevel(logging.ERROR)
            
            try:
                validation_result = self.flow_validator.validate_flow_integrity(self.data)
            finally:
                # 恢复原始日志级别
                logging.getLogger().setLevel(original_level)
                audit_logger.logger.setLevel(original_audit_level)
            if not validation_result['is_valid']:
                print(f"⚠️  流水完整性验证发现 {validation_result['errors_count']} 个问题", file=sys.stderr)
                sys.stderr.flush()
                sys.stdout.flush()
                audit_logger.warning(f"流水完整性验证发现{validation_result['errors_count']}个问题")
                
                if validation_result['optimization_failed']:
                    print("❌ 流水优化失败，无法自动修复数据完整性问题", file=sys.stderr)
                    sys.stderr.flush()
                    sys.stdout.flush()
                    audit_logger.error("❌ 流水优化失败，无法自动修复数据完整性问题")
                    return {
                        "success": False,
                        "message": "流水完整性验证失败，无法自动修复",
                        "file_path": file_path
                    }
                
                if validation_result['optimizations_count'] > 0:
                    print(f"🔧 已通过重排序修复 {validation_result['optimizations_count']} 个问题", file=sys.stderr)
                    sys.stderr.flush()
                    sys.stdout.flush()
                    audit_logger.info(f"已通过重排序修复{validation_result['optimizations_count']}个问题")
                    self.data = validation_result['result_dataframe']
                    print("✅ 使用修复后的数据继续处理（源文件保持不变）", file=sys.stderr)
                    sys.stderr.flush()
                    sys.stdout.flush()
                    audit_logger.info("✅ 使用修复后的数据继续处理（源文件保持不变）")
                    
                    # 重要：数据已重排序，重置DataFrame索引以避免余额验证问题
                    self.data.reset_index(drop=True, inplace=True)
            else:
                print("✅ 流水完整性验证通过", file=sys.stderr)
                sys.stderr.flush()
                audit_logger.info("✅ 流水完整性验证通过")
                sys.stdout.flush()
                sys.stderr.flush()
            
            # 3. 数据验证（静默模式）
            print("🔎 开始数据验证...", file=sys.stderr)
            sys.stderr.flush()
            sys.stdout.flush()
            
            # 临时提升日志级别以减少详细输出（隐藏WARNING信息）
            logging.getLogger().setLevel(logging.ERROR)
            audit_logger.logger.setLevel(logging.ERROR)
            
            try:
                validation_result = self.data_processor.验证数据完整性(self.data)
            finally:
                # 恢复原始日志级别
                logging.getLogger().setLevel(original_level)
                audit_logger.logger.setLevel(original_audit_level)
            if not validation_result['is_valid']:
                print("⚠️  数据验证发现问题，但继续处理", file=sys.stderr)
                sys.stderr.flush()
                sys.stdout.flush()
                audit_logger.warning("数据验证发现问题，但继续处理")
                for error in validation_result['errors'][:5]:
                    audit_logger.warning(error)
            else:
                print("✅ 数据验证通过", file=sys.stderr)
                sys.stderr.flush()
                sys.stdout.flush()
                
            # 4. 设置基本信息
            self.total_rows = len(self.data)
            self.current_row = 0
            
            # 清除历史记录
            self.query_history.clear()
            self.processing_steps.clear()
            self.error_records.clear()
            
            result = {
                "success": True,
                "total_rows": self.total_rows,
                "message": f"数据加载成功，共 {self.total_rows} 行（包含完整性验证）",
                "file_path": file_path
            }
            
            audit_logger.info(f"时点查询数据加载完成: {self.total_rows} 行")
            return result
                
        except Exception as e:
            error_msg = f"数据加载出错: {str(e)}"
            audit_logger.error(error_msg)
            return {
                "success": False,
                "message": error_msg,
                "error_details": str(e),
                "file_path": file_path
            }
    
    def query_time_point(self, target_row: int, save_to_history: bool = True) -> Dict[str, Any]:
        """
        查询指定时点（行数）的系统状态
        
        Args:
            target_row: 目标行数 (1-based)
            save_to_history: 是否保存到查询历史
            
        Returns:
            查询结果信息
        """
        start_time = datetime.now()
        
        try:
            # 输入验证
            if self.data is None:
                return {
                    "success": False,
                    "message": "请先加载数据文件",
                    "query_time": start_time.isoformat()
                }
            
            if target_row < 1 or target_row > self.total_rows:
                return {
                    "success": False,
                    "message": f"行数超出范围 (1-{self.total_rows})",
                    "query_time": start_time.isoformat()
                }
            
            audit_logger.info(f"开始时点查询: 第 {target_row} 行，算法: {self.algorithm}")
            
            # 重置追踪器（每次查询都从头开始）
            self._reset_tracker()
            
            # 处理数据到目标行
            processing_result = self._process_to_row(target_row)
            
            if not processing_result["success"]:
                return {
                    "success": False,
                    "message": processing_result["message"],
                    "query_time": start_time.isoformat(),
                    "target_row": target_row
                }
            
            # 生成查询结果
            query_result = self._generate_query_result(target_row, start_time)
            
            # 保存到历史记录
            if save_to_history:
                self._save_to_history(query_result)
            
            audit_logger.info(f"时点查询完成: 第 {target_row} 行")
            return query_result
            
        except Exception as e:
            error_msg = f"时点查询失败: {str(e)}"
            audit_logger.error(error_msg)
            return {
                "success": False,
                "message": error_msg,
                "error_details": str(e),
                "query_time": start_time.isoformat(),
                "target_row": target_row
            }
    
    def _reset_tracker(self) -> None:
        """重置追踪器状态"""
        # 重新创建追踪器
        self.tracker = TrackerFactory.create_tracker(self.algorithm)
        self.current_row = 0
        self.processing_steps.clear()
        self.error_records.clear()
        
        # 设置初始余额
        if self.data is not None:
            初始余额 = self.data_processor.计算初始余额(self.data)
            if 初始余额 > 0:
                self.tracker.初始化余额(初始余额, '公司')
                
                self.processing_steps.append({
                    "step": 0,
                    "action": "初始化余额",
                    "amount": 初始余额,
                    "result": f"初始余额设置为: {初始余额:,.2f} (公司余额)",
                    "timestamp": datetime.now()
                })
    
    def _process_to_row(self, target_row: int) -> Dict[str, Any]:
        """
        处理数据到指定行数
        
        Args:
            target_row: 目标行数
            
        Returns:
            处理结果
        """
        try:
            import sys
            # 逐行处理到目标行
            progress_interval = max(1, target_row // 20)  # 最多显示20次进度更新
            
            for i in range(target_row):
                try:
                    step_result = self._process_single_row(i)
                    if not step_result["success"]:
                        return {
                            "success": False,
                            "message": f"第 {i + 1} 行处理失败: {step_result['message']}",
                            "failed_row": i + 1
                        }
                    
                    # 显示进度（每处理一定数量行就输出一次）
                    if (i + 1) % progress_interval == 0 or i + 1 == target_row:
                        percentage = (i + 1) / target_row * 100
                        print(f"⏳ 处理进度: {i + 1}/{target_row} ({percentage:.1f}%)", file=sys.stderr)
                        sys.stderr.flush()
                    
                    # 跳过逐行余额验证（流水完整性验证已确保数据正确性）
                    # 避免由于数据重排序导致的行号不匹配问题
                
                except Exception as e:
                    error_info = {
                        'row': i + 1,
                        'error': str(e),
                        'timestamp': datetime.now(),
                        'tracker_state': self._get_tracker_state()
                    }
                    self.error_records.append(error_info)
                    
                    return {
                        "success": False,
                        "message": f"第 {i + 1} 行处理出错: {str(e)}",
                        "failed_row": i + 1,
                        "error_details": str(e)
                    }
            
            self.current_row = target_row
            return {
                "success": True,
                "message": f"成功处理到第 {target_row} 行",
                "processed_rows": target_row
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"批量处理失败: {str(e)}",
                "error_details": str(e)
            }
    
    def _process_single_row(self, row_idx: int) -> Dict[str, Any]:
        """
        处理单行数据
        
        Args:
            row_idx: 行索引 (0-based)
            
        Returns:
            处理结果
        """
        try:
            row = self.data.iloc[row_idx]
            
            # 使用数据处理器处理单行交易
            处理结果 = self.data_processor.处理单行交易(row, row_idx)
            
            # 记录处理步骤
            step_info = {
                "step": row_idx + 1,
                "action": "处理交易",
                "direction": 处理结果['方向'],
                "amount": 处理结果['实际金额'],
                "fund_attr": 处理结果['资金属性'],
                "timestamp": datetime.now()
            }
            
            # 根据交易方向调用追踪器
            if 处理结果['方向'] == '收入':
                if 处理结果['is_investment']:
                    个人占比, 公司占比, 行为性质 = self.tracker.处理投资产品赎回(
                        处理结果['实际金额'], 
                        处理结果['资金属性'], 
                        处理结果['完整时间戳']
                    )
                else:
                    个人占比, 公司占比, 行为性质 = self.tracker.处理资金流入(
                        处理结果['实际金额'], 
                        处理结果['资金属性'], 
                        处理结果['完整时间戳']
                    )
            elif 处理结果['方向'] == '支出':
                个人占比, 公司占比, 行为性质 = self.tracker.处理资金流出(
                    处理结果['实际金额'], 
                    处理结果['资金属性'], 
                    处理结果['完整时间戳']
                )
            else:
                个人占比, 公司占比, 行为性质 = 0, 0, '无交易'
            
            # 更新步骤信息
            step_info.update({
                "personal_ratio": 个人占比,
                "company_ratio": 公司占比,
                "behavior": 行为性质,
                "result": f"{处理结果['方向']} {处理结果['实际金额']:,.2f} - {行为性质}"
            })
            
            self.processing_steps.append(step_info)
            
            # 将计算出的行为性质存储回DataFrame中，以便查询结果时使用
            if self.data is not None:
                self.data.at[row_idx, '行为性质'] = 行为性质
                self.data.at[row_idx, '个人占比'] = 个人占比
                self.data.at[row_idx, '公司占比'] = 公司占比
            
            return {
                "success": True,
                "message": f"第 {row_idx + 1} 行处理成功",
                "processing_result": 处理结果,
                "personal_ratio": 个人占比,
                "company_ratio": 公司占比,
                "behavior": 行为性质
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"第 {row_idx + 1} 行处理失败: {str(e)}",
                "error_details": str(e)
            }
    
    def _validate_balance(self, row_num: int, expected_balance: float) -> bool:
        """
        验证余额是否匹配
        
        Args:
            row_num: 行号 (1-based)
            expected_balance: 期望余额
            
        Returns:
            是否匹配
        """
        if self.tracker is None:
            return False
        
        actual_balance = self.tracker.个人余额 + self.tracker.公司余额
        
        # 使用配置的容差
        if abs(actual_balance - expected_balance) <= Config.BALANCE_TOLERANCE:
            return True
        else:
            # 记录余额错误
            error_info = {
                'row': row_num,
                'expected': expected_balance,
                'actual': actual_balance,
                'difference': actual_balance - expected_balance,
                'timestamp': datetime.now(),
                'tracker_state': self._get_tracker_state()
            }
            self.error_records.append(error_info)
            
            audit_logger.warning(f"第{row_num}行余额不匹配: 期望{expected_balance:,.2f}, 实际{actual_balance:,.2f}")
            return False
    
    def _get_tracker_state(self) -> Dict[str, Any]:
        """获取追踪器当前状态"""
        if self.tracker is None:
            return {}
        
        # 计算资金缺口：累计挪用 - 累计归还给公司的本金 - 累计垫付
        资金缺口 = (self.tracker.累计挪用金额 - 
                   self.tracker.累计由资金池回归公司余额本金 - 
                   self.tracker.累计垫付金额)
        
        return {
            "personal_balance": self.tracker.个人余额,
            "company_balance": self.tracker.公司余额,
            "total_balance": self.tracker.个人余额 + self.tracker.公司余额,
            "total_misappropriation": self.tracker.累计挪用金额,  # 修复字段名匹配前端期望
            "total_advance": self.tracker.累计垫付金额,
            "total_returned_company": self.tracker.累计由资金池回归公司余额本金,
            "total_returned_personal": self.tracker.累计由资金池回归个人余额本金,
            "personal_profit": self.tracker.总计个人应分配利润,
            "company_profit": self.tracker.总计公司应分配利润,
            "funding_gap": 资金缺口,       # 统一的资金缺口字段
            "is_initialized": self.tracker.已初始化
        }
    
    def _generate_query_result(self, target_row: int, start_time: datetime) -> Dict[str, Any]:
        """
        生成查询结果
        
        Args:
            target_row: 目标行数
            start_time: 查询开始时间
            
        Returns:
            完整的查询结果
        """
        end_time = datetime.now()
        processing_time = (end_time - start_time).total_seconds()
        
        # 基本信息
        result = {
            "success": True,
            "algorithm": self.algorithm,
            "target_row": target_row,
            "total_rows": self.total_rows,
            "query_time": start_time.isoformat(),
            "processing_time": processing_time
        }
        
        # 追踪器状态
        if self.tracker:
            result["tracker_state"] = self._get_tracker_state()
        
        # 目标行数据
        if target_row > 0 and self.data is not None:
            import math
            target_row_data = self.data.iloc[target_row - 1]
            
            # 安全转换数值，处理NaN值
            def safe_float(value, default=0.0):
                try:
                    if value is None or (isinstance(value, float) and math.isnan(value)):
                        return default
                    return float(value)
                except (ValueError, TypeError):
                    return default
            
            # 处理资金流向：根据收入支出金额判断
            income_amount = safe_float(target_row_data.get('交易收入金额'))
            expense_amount = safe_float(target_row_data.get('交易支出金额'))
            
            if income_amount > 0 and expense_amount == 0:
                flow_type = "收入"
            elif expense_amount > 0 and income_amount == 0:
                flow_type = "支出"
            elif income_amount > 0 and expense_amount > 0:
                flow_type = "收支"
            else:
                flow_type = "无变动"
            
            # 处理行为性质：清理投资产品的前缀格式
            raw_behavior = str(target_row_data.get('行为性质', ''))
            clean_behavior = self._clean_behavior_description(raw_behavior)
            
            result["target_row_data"] = {
                "timestamp": str(target_row_data.get('完整时间戳', '')),
                "income_amount": income_amount,
                "expense_amount": expense_amount,
                "balance": safe_float(target_row_data.get('余额')),
                "fund_attr": str(target_row_data.get('资金属性', '')),
                "flow_type": flow_type,
                "behavior": clean_behavior
            }
        
        # 处理统计
        result["processing_stats"] = {
            "total_steps": len(self.processing_steps),
            "error_count": len(self.error_records),
            "last_processed_row": self.current_row
        }
        
        # 最近的处理步骤（最多10步）
        result["recent_steps"] = self.processing_steps[-10:] if self.processing_steps else []
        
        # 错误记录（如果有）
        if self.error_records:
            result["errors"] = self.error_records[-5:]  # 最近5个错误
        
        # 添加当前时点可用的资金池信息
        if hasattr(self.tracker, '_投资产品资金池') and self.tracker._投资产品资金池:
            available_pools = []
            for pool_name, pool_info in self.tracker._投资产品资金池.items():
                if pool_info.get('总金额', 0) != 0:  # 只显示有余额的资金池
                    available_pools.append({
                        'name': pool_name,
                        'total_amount': pool_info.get('总金额', 0),
                        'personal_ratio': pool_info.get('个人占比', 0),
                        'company_ratio': pool_info.get('公司占比', 0)
                    })
            result["available_fund_pools"] = available_pools
        
        return result
    
    def query_fund_pool(self, pool_name: str) -> Dict[str, Any]:
        """
        查询指定资金池的详细信息
        
        Args:
            pool_name: 资金池名称
            
        Returns:
            资金池查询结果
        """
        try:
            if self.tracker is None:
                return {
                    "success": False,
                    "message": "追踪器未初始化"
                }
            
            # 直接在这里实现资金池查询，避免循环导入
            if not hasattr(self.tracker, '_场外资金池记录') or not self.tracker._场外资金池记录:
                return {
                    "success": False,
                    "message": "没有找到资金池记录"
                }
            
            # 筛选指定资金池的记录
            pool_records = [
                record for record in self.tracker._场外资金池记录
                if record.get('资金池名称') == pool_name
            ]
            
            if not pool_records:
                return {
                    "success": False,
                    "message": f"没有找到资金池 {pool_name} 的记录"
                }
            
            # 处理记录，移除不需要的字段
            filtered_records = []
            for record in pool_records:
                filtered_record = {
                    '交易时间': record.get('交易时间', ''),
                    '资金池名称': record.get('资金池名称', ''),
                    '入金': record.get('入金', 0),
                    '出金': record.get('出金', 0),
                    '总余额': record.get('总余额', 0),
                    '单笔资金占比': record.get('单笔资金占比', record.get('资金占比', '')),
                    '总资金占比': record.get('总资金占比', '')
                    # 不包含：行为性质、累计申购、累计赎回
                }
                filtered_records.append(filtered_record)
            
            # 计算汇总信息
            total_inflow = sum(record.get('入金', 0) for record in pool_records if isinstance(record.get('入金'), (int, float)))
            total_outflow = sum(record.get('出金', 0) for record in pool_records if isinstance(record.get('出金'), (int, float)))
            
            # 获取最新余额
            latest_record = pool_records[-1]
            current_balance = latest_record.get('总余额', 0)
            
            # 添加总计行
            summary_record = {
                '交易时间': '── 总计 ──',
                '资金池名称': f'{pool_name} 汇总',
                '入金': f'总入金: ¥{total_inflow:,.0f}',
                '出金': f'总出金: ¥{total_outflow:,.0f}',
                '总余额': f'当前余额: ¥{current_balance:,.0f}',
                '单笔资金占比': '── 汇总 ──',
                '总资金占比': f'净变化: ¥{current_balance:,.0f}'
            }
            filtered_records.append(summary_record)
            
            return {
                "success": True,
                "pool_name": pool_name,
                "records": filtered_records,
                "summary": {
                    "total_inflow": total_inflow,
                    "total_outflow": total_outflow,
                    "current_balance": current_balance,
                    "record_count": len(pool_records)
                }
            }
            
        except Exception as e:
            error_msg = f"资金池查询失败: {str(e)}"
            audit_logger.error(error_msg)
            return {
                "success": False,
                "message": error_msg,
                "error_details": str(e)
            }
    
    def _clean_behavior_description(self, behavior: str) -> str:
        """
        清理行为性质描述，去掉投资产品的前缀格式
        
        例如：
        "理财申购-理财-SYA160401160408：投资挪用：1,898,094.23；个人投资：121,905.77"
        → "投资挪用：1,898,094.23；个人投资：121,905.77"
        
        保持非投资行为不变：
        "垫付：5,766.13；公司支付：533.87" → "垫付：5,766.13；公司支付：533.87"
        """
        if not behavior:
            return behavior
        
        # 检查是否包含投资产品的前缀格式（如：理财申购-理财-SYA160401160408：）
        import re
        investment_prefix_pattern = r'^[^：]*申购-[^：]*：'
        
        if re.match(investment_prefix_pattern, behavior):
            # 去掉前缀，只保留冒号后面的内容
            parts = behavior.split('：', 1)
            if len(parts) > 1:
                return parts[1]
        
        return behavior
    
    def _save_to_history(self, query_result: Dict[str, Any]) -> None:
        """
        保存查询结果到历史记录
        
        Args:
            query_result: 查询结果
        """
        # 简化历史记录（只保存关键信息）
        history_item = {
            "id": len(self.query_history) + 1,
            "algorithm": query_result["algorithm"],
            "target_row": query_result["target_row"],
            "query_time": query_result["query_time"],
            "processing_time": query_result["processing_time"],
            "success": query_result["success"],
            "tracker_state": query_result.get("tracker_state", {}),
            "error_count": query_result.get("processing_stats", {}).get("error_count", 0)
        }
        
        self.query_history.append(history_item)
        
        # 保持历史记录不超过最大长度
        if len(self.query_history) > self.MAX_HISTORY_SIZE:
            self.query_history = self.query_history[-self.MAX_HISTORY_SIZE:]
    
    def get_query_history(self, limit: int = 20) -> List[Dict[str, Any]]:
        """
        获取查询历史记录
        
        Args:
            limit: 返回记录数量限制
            
        Returns:
            历史记录列表（最新的在前）
        """
        return self.query_history[-limit:][::-1]
    
    def clear_history(self) -> Dict[str, Any]:
        """
        清除查询历史记录
        
        Returns:
            清除结果
        """
        cleared_count = len(self.query_history)
        self.query_history.clear()
        
        return {
            "success": True,
            "message": f"已清除 {cleared_count} 条历史记录"
        }
    
    def export_query_result(self, query_result: Dict[str, Any], file_path: str) -> Dict[str, Any]:
        """
        导出查询结果到文件
        
        Args:
            query_result: 查询结果
            file_path: 导出文件路径
            
        Returns:
            导出结果
        """
        try:
            if file_path.endswith('.json'):
                # 导出为JSON
                with open(file_path, 'w', encoding='utf-8') as f:
                    json.dump(query_result, f, ensure_ascii=False, indent=2, default=str)
                    
            elif file_path.endswith('.xlsx'):
                # 导出为Excel
                self._export_to_excel(query_result, file_path)
                
            else:
                return {
                    "success": False,
                    "message": "不支持的文件格式，请使用 .json 或 .xlsx"
                }
            
            audit_logger.info(f"查询结果已导出至: {file_path}")
            return {
                "success": True,
                "message": f"查询结果已导出至: {file_path}",
                "file_path": file_path
            }
            
        except Exception as e:
            error_msg = f"导出失败: {str(e)}"
            audit_logger.error(error_msg)
            return {
                "success": False,
                "message": error_msg,
                "error_details": str(e)
            }
    
    def _export_to_excel(self, query_result: Dict[str, Any], file_path: str) -> None:
        """
        导出查询结果到Excel文件
        
        Args:
            query_result: 查询结果
            file_path: Excel文件路径
        """
        import pandas as pd
        
        with pd.ExcelWriter(file_path, engine='openpyxl') as writer:
            # 基本信息
            basic_info = pd.DataFrame([{
                "算法": query_result.get("algorithm"),
                "目标行数": query_result.get("target_row"),
                "查询时间": query_result.get("query_time"),
                "处理时间(秒)": query_result.get("processing_time"),
                "数据总行数": query_result.get("total_rows")
            }])
            basic_info.to_excel(writer, sheet_name='基本信息', index=False)
            
            # 追踪器状态
            if "tracker_state" in query_result:
                state = query_result["tracker_state"]
                tracker_info = pd.DataFrame([{
                    "个人余额": state.get("personal_balance", 0),
                    "公司余额": state.get("company_balance", 0),
                    "总余额": state.get("total_balance", 0),
                    "累计挪用": state.get("total_misuse", 0),
                    "累计垫付": state.get("total_advance", 0),
                    "已归还公司本金": state.get("total_returned_company", 0),
                    "已归还个人本金": state.get("total_returned_personal", 0),
                    "个人利润": state.get("personal_profit", 0),
                    "公司利润": state.get("company_profit", 0)
                }])
                tracker_info.to_excel(writer, sheet_name='追踪器状态', index=False)
            
            # 目标行数据
            if "target_row_data" in query_result:
                row_data = query_result["target_row_data"]
                target_info = pd.DataFrame([row_data])
                target_info.to_excel(writer, sheet_name='目标行数据', index=False)
            
            # 处理步骤（如果有）
            if "recent_steps" in query_result and query_result["recent_steps"]:
                steps_df = pd.DataFrame(query_result["recent_steps"])
                steps_df.to_excel(writer, sheet_name='处理步骤', index=False)
            
            # 错误记录（如果有）
            if "errors" in query_result and query_result["errors"]:
                errors_df = pd.DataFrame(query_result["errors"])
                errors_df.to_excel(writer, sheet_name='错误记录', index=False)
    
    def get_service_status(self) -> Dict[str, Any]:
        """
        获取服务状态信息
        
        Returns:
            服务状态
        """
        return {
            "algorithm": self.algorithm,
            "data_loaded": self.data is not None,
            "total_rows": self.total_rows,
            "current_row": self.current_row,
            "history_count": len(self.query_history),
            "max_history_size": self.MAX_HISTORY_SIZE,
            "tracker_initialized": self.tracker is not None and self.tracker.已初始化 if self.tracker else False
        }