"""
追踪器工厂
创建不同类型的追踪器实例
"""

from typing import List
from core.interfaces.tracker_interface import ITracker
from core.trackers.fifo_adapter import FIFOTracker
from core.trackers.balance_method_tracker import BalanceMethodTracker


class TrackerFactory:
    """追踪器工厂类"""
    
    # 支持的算法类型
    SUPPORTED_ALGORITHMS = {
        "FIFO": "FIFO先进先出算法",
        "BALANCE_METHOD": "差额计算法（余额优先）"
    }
    
    @staticmethod
    def create_tracker(algorithm: str) -> ITracker:
        """
        创建追踪器实例
        
        Args:
            algorithm: 算法类型 ("FIFO" 或 "BALANCE_METHOD")
            
        Returns:
            追踪器实例
            
        Raises:
            ValueError: 不支持的算法类型
        """
        algorithm_upper = algorithm.upper()
        
        if algorithm_upper == "FIFO":
            return FIFOTracker()
        elif algorithm_upper == "BALANCE_METHOD":
            return BalanceMethodTracker()
        else:
            raise ValueError(
                f"不支持的算法类型: {algorithm}. "
                f"支持的算法: {list(TrackerFactory.SUPPORTED_ALGORITHMS.keys())}"
            )
    
    @staticmethod
    def get_available_algorithms() -> List[str]:
        """
        获取可用的算法列表
        
        Returns:
            算法类型列表
        """
        return list(TrackerFactory.SUPPORTED_ALGORITHMS.keys())
    
    @staticmethod
    def get_algorithm_description(algorithm: str) -> str:
        """
        获取算法描述
        
        Args:
            algorithm: 算法类型
            
        Returns:
            算法描述
        """
        algorithm_upper = algorithm.upper()
        return TrackerFactory.SUPPORTED_ALGORITHMS.get(
            algorithm_upper, 
            f"未知算法: {algorithm}"
        )
    
    @staticmethod
    def get_algorithms_info() -> dict:
        """
        获取所有算法信息
        
        Returns:
            算法信息字典
        """
        return TrackerFactory.SUPPORTED_ALGORITHMS.copy()