"""
时点查询服务
基于debug_tool.py转换而来，支持特定时点查询和历史记录管理
"""

import pandas as pd
from datetime import datetime
from typing import Dict, List, Any, Optional, Tuple
import json

from core.interfaces.tracker_interface import ITracker
from core.factories.tracker_factory import TrackerFactory
from utils.data_processor import DataProcessor
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
        try:
            audit_logger.info(f"开始加载数据文件: {file_path}")
            
            # 使用数据处理器加载数据
            self.data = self.data_processor.预处理财务数据(file_path)
            
            if self.data is not None:
                self.total_rows = len(self.data)
                self.current_row = 0
                
                # 清除历史记录
                self.query_history.clear()
                self.processing_steps.clear()
                self.error_records.clear()
                
                result = {
                    "success": True,
                    "total_rows": self.total_rows,
                    "message": f"数据加载成功，共 {self.total_rows} 行",
                    "file_path": file_path
                }
                
                audit_logger.info(f"数据加载成功: {self.total_rows} 行")
                return result
            else:
                error_msg = "数据加载失败：预处理返回空数据"
                audit_logger.error(error_msg)
                return {
                    "success": False,
                    "message": error_msg,
                    "file_path": file_path
                }
                
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
            # 逐行处理到目标行
            for i in range(target_row):
                try:
                    step_result = self._process_single_row(i)
                    if not step_result["success"]:
                        return {
                            "success": False,
                            "message": f"第 {i + 1} 行处理失败: {step_result['message']}",
                            "failed_row": i + 1
                        }
                    
                    # 验证余额（如果数据有余额列）
                    if '余额' in self.data.columns:
                        expected_balance = self.data.iloc[i]['余额']
                        if not self._validate_balance(i + 1, expected_balance):
                            return {
                                "success": False,
                                "message": f"第 {i + 1} 行余额验证失败",
                                "failed_row": i + 1,
                                "balance_error": True
                            }
                
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
        
        return {
            "personal_balance": self.tracker.个人余额,
            "company_balance": self.tracker.公司余额,
            "total_balance": self.tracker.个人余额 + self.tracker.公司余额,
            "total_misuse": self.tracker.累计挪用金额,
            "total_advance": self.tracker.累计垫付金额,
            "total_returned": self.tracker.累计已归还公司本金,
            "personal_profit": self.tracker.总计个人分配利润,
            "company_profit": self.tracker.总计公司分配利润,
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
            target_row_data = self.data.iloc[target_row - 1]
            result["target_row_data"] = {
                "timestamp": str(target_row_data.get('完整时间戳', '')),
                "income_amount": float(target_row_data.get('交易收入金额', 0) or 0),
                "expense_amount": float(target_row_data.get('交易支出金额', 0) or 0),
                "balance": float(target_row_data.get('余额', 0) or 0),
                "fund_attr": str(target_row_data.get('资金属性', '')),
                "flow_type": str(target_row_data.get('资金流向类型', '')),
                "behavior": str(target_row_data.get('行为性质', ''))
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
        
        return result
    
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
                    "已归还本金": state.get("total_returned", 0),
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