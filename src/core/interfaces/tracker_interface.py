"""
追踪器接口抽象
定义FIFO和差额计算法的统一接口
"""

from abc import ABC, abstractmethod
from typing import Tuple, Dict, Any, Optional
import pandas as pd


class ITracker(ABC):
    """追踪器抽象接口"""
    
    @abstractmethod
    def 初始化余额(self, 初始余额: float, 余额类型: str = '公司') -> None:
        """
        初始化余额
        
        Args:
            初始余额: 初始余额金额
            余额类型: 余额类型（'个人' 或 '公司'）
        """
        pass
    
    @abstractmethod  
    def 处理资金流入(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """
        处理资金流入
        
        Args:
            金额: 流入金额
            资金属性: 资金属性描述
            交易日期: 交易日期
            
        Returns:
            (个人占比, 公司占比, 行为性质)
        """
        pass
    
    @abstractmethod
    def 处理资金流出(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """
        处理资金流出
        
        Args:
            金额: 流出金额
            资金属性: 资金属性描述
            交易日期: 交易日期
            
        Returns:
            (个人占比, 公司占比, 行为性质)
        """
        pass
    
    @abstractmethod
    def 处理投资产品赎回(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """
        处理投资产品赎回
        
        Args:
            金额: 赎回金额
            资金属性: 资金属性描述
            交易日期: 交易日期
            
        Returns:
            (个人占比, 公司占比, 行为性质)
        """
        pass
    
    @abstractmethod
    def 获取状态摘要(self) -> Dict[str, Any]:
        """
        获取追踪器状态摘要
        
        Returns:
            状态摘要字典
        """
        pass
    
    @abstractmethod
    def 获取当前资金占比(self) -> Tuple[float, float]:
        """
        获取当前个人和公司资金占比
        
        Returns:
            (个人占比, 公司占比)
        """
        pass
    
    @abstractmethod
    def 生成投资产品交易记录Excel(self, 文件名: str = "投资产品交易记录.xlsx") -> None:
        """
        生成投资产品交易记录的Excel文件
        
        Args:
            文件名: 保存的Excel文件名
        """
        pass
    
    # 通用属性访问接口
    @property
    @abstractmethod
    def 个人余额(self) -> float:
        """个人余额"""
        pass
        
    @property  
    @abstractmethod
    def 公司余额(self) -> float:
        """公司余额"""
        pass
        
    @property
    @abstractmethod
    def 累计挪用金额(self) -> float:
        """累计挪用金额"""
        pass
        
    @property
    @abstractmethod
    def 累计垫付金额(self) -> float:
        """累计垫付金额"""
        pass
        
    @property
    @abstractmethod
    def 累计已归还公司本金(self) -> float:
        """累计已归还公司本金"""
        pass
        
    @property
    @abstractmethod
    def 总计个人分配利润(self) -> float:
        """总计个人分配利润"""
        pass
        
    @property
    @abstractmethod
    def 总计公司分配利润(self) -> float:
        """总计公司分配利润"""
        pass
        
    @property
    @abstractmethod
    def 已初始化(self) -> bool:
        """是否已初始化"""
        pass