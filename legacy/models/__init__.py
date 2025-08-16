"""
模型包 - 包含核心业务逻辑模型
"""

# 导入所有模型类
from .fifo_tracker import FIFO资金追踪器
from .investment_manager import InvestmentProductManager
from .flow_analyzer import FlowAnalyzer
from .behavior_analyzer import BehaviorAnalyzer

__all__ = [
    'FIFO资金追踪器',
    'InvestmentProductManager', 
    'FlowAnalyzer',
    'BehaviorAnalyzer'
] 