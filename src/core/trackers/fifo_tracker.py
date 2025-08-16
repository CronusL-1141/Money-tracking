"""
FIFO追踪器包装器
完全包装现有FIFO追踪器，保持所有逻辑不变
"""

from typing import Tuple, Dict, Any, Optional
import pandas as pd

from core.interfaces.tracker_interface import ITracker
from models.fifo_tracker import FIFO资金追踪器 as LegacyFIFOTracker


class FIFOTracker(ITracker):
    """FIFO追踪器包装器 - 包装现有FIFO逻辑，零修改"""
    
    def __init__(self):
        """初始化FIFO追踪器 - 使用现有实现"""
        # 完全包装现有的FIFO追踪器
        self._legacy_tracker = LegacyFIFOTracker()
    
    def 初始化余额(self, 初始余额: float, 余额类型: str = '公司') -> None:
        """初始化余额 - 委托给现有实现"""
        return self._legacy_tracker.初始化余额(初始余额, 余额类型)
    
    def 处理资金流入(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """处理资金流入 - 委托给现有实现"""
        return self._legacy_tracker.处理资金流入(金额, 资金属性, 交易日期)
    
    def 处理资金流出(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """处理资金流出 - 委托给现有实现"""
        return self._legacy_tracker.处理资金流出(金额, 资金属性, 交易日期)
    
    def 处理投资产品赎回(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """处理投资产品赎回 - 委托给现有实现"""
        return self._legacy_tracker.处理投资产品赎回(金额, 资金属性, 交易日期)
    
    def 获取状态摘要(self) -> Dict[str, Any]:
        """获取状态摘要 - 委托给现有实现"""
        return self._legacy_tracker.获取状态摘要()
    
    def 获取当前资金占比(self) -> Tuple[float, float]:
        """获取当前资金占比 - 委托给现有实现"""
        return self._legacy_tracker.获取当前资金占比()
    
    def 生成投资产品交易记录Excel(self, 文件名: str = "投资产品交易记录.xlsx") -> None:
        """生成投资产品交易记录Excel - 委托给现有实现"""
        return self._legacy_tracker.生成投资产品交易记录Excel(文件名)
    
    # 属性访问 - 委托给现有实现
    @property
    def 个人余额(self) -> float:
        """个人余额"""
        return self._legacy_tracker.个人余额
        
    @property  
    def 公司余额(self) -> float:
        """公司余额"""
        return self._legacy_tracker.公司余额
        
    @property
    def 累计挪用金额(self) -> float:
        """累计挪用金额"""
        return self._legacy_tracker.累计挪用金额
        
    @property
    def 累计垫付金额(self) -> float:
        """累计垫付金额"""
        return self._legacy_tracker.累计垫付金额
        
    @property
    def 累计已归还公司本金(self) -> float:
        """累计已归还公司本金"""
        return self._legacy_tracker.累计已归还公司本金
        
    @property
    def 总计个人分配利润(self) -> float:
        """总计个人分配利润"""
        return self._legacy_tracker.总计个人分配利润
        
    @property
    def 总计公司分配利润(self) -> float:
        """总计公司分配利润"""
        return self._legacy_tracker.总计公司分配利润
        
    @property
    def 已初始化(self) -> bool:
        """是否已初始化"""
        return self._legacy_tracker.已初始化
    
    # 提供对原始追踪器的直接访问（如果需要访问特定功能）
    @property
    def legacy_tracker(self) -> LegacyFIFOTracker:
        """访问原始FIFO追踪器（如果需要特定功能）"""
        return self._legacy_tracker