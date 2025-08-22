"""
行为分析器模块
负责分析交易行为性质，判断是否构成挪用或垫付
"""

from typing import Tuple
from config import Config
from utils.logger import audit_logger


class BehaviorAnalyzer:
    """行为分析器"""
    
    def __init__(self):
        """初始化行为分析器"""
        self.累计挪用金额 = 0.0
        self.累计垫付金额 = 0.0
    
    def 分析行为性质(self, 资金属性: str, 个人扣除: float, 公司扣除: float, 总金额: float) -> str:
        """
        分析交易的行为性质，判断是否构成挪用或垫付
        
        Args:
            资金属性: 资金属性描述
            个人扣除: 个人资金扣除金额
            公司扣除: 公司资金扣除金额
            总金额: 总交易金额
            
        Returns:
            行为性质描述
        """
        if 总金额 <= 0:
            return "无交易"
        
        行为描述 = []
        
        # 判断资金属性类型
        资金属性类型 = self._判断资金属性类型(资金属性)
        
        if 资金属性类型 == '个人':
            # 个人应付/支出
            if 公司扣除 > 0:
                # 个人支出使用了公司资金 - 构成挪用
                self.累计挪用金额 += 公司扣除
                self.累计挪用金额 = Config.format_number(self.累计挪用金额)
                行为描述.append(f"挪用：{公司扣除:,.2f}")
            if 个人扣除 > 0:
                # 个人支出使用了个人资金 - 正常
                行为描述.append(f"个人支付：{个人扣除:,.2f}")
                
        elif 资金属性类型 == '公司':
            # 公司应付/支出
            if 个人扣除 > 0:
                # 公司支出使用了个人资金 - 构成垫付
                self.累计垫付金额 += 个人扣除
                self.累计垫付金额 = Config.format_number(self.累计垫付金额)
                行为描述.append(f"垫付：{个人扣除:,.2f}")
            if 公司扣除 > 0:
                # 公司支出使用了公司资金 - 正常
                行为描述.append(f"公司支付：{公司扣除:,.2f}")
                
        else:
            # 其他类型
            if 个人扣除 > 0:
                行为描述.append(f"个人支付：{个人扣除:,.2f}")
            if 公司扣除 > 0:
                行为描述.append(f"公司支付：{公司扣除:,.2f}")
        
        return "；".join(行为描述) if 行为描述 else "无明确行为"
    
    def _判断资金属性类型(self, 资金属性: str) -> str:
        """
        判断资金属性是个人还是公司
        
        Args:
            资金属性: 资金属性描述
            
        Returns:
            资金属性类型（'个人'、'公司'、'其他'）
        """
        资金属性_str = str(资金属性).strip()
        
        if Config.is_personal_fund(资金属性):
            return '个人'
        elif Config.is_company_fund(资金属性):
            return '公司'
        else:
            return '其他'
    
    def 分析投资行为(self, 个人扣除: float, 公司扣除: float) -> Tuple[str, float]:
        """
        分析投资行为，投资是个人行为，使用公司资金就是挪用
        
        Args:
            个人扣除: 个人资金扣除金额
            公司扣除: 公司资金扣除金额
            
        Returns:
            (行为性质, 挪用金额)
        """
        挪用金额 = 0.0
        行为描述 = []
        
        if 公司扣除 > 0:
            # 投资使用公司资金构成挪用
            挪用金额 = 公司扣除
            行为描述.append(f"投资挪用：{公司扣除:,.2f}")
            
        if 个人扣除 > 0:
            行为描述.append(f"个人投资：{个人扣除:,.2f}")
        
        行为性质 = "；".join(行为描述) if 行为描述 else "无投资"
        
        return 行为性质, 挪用金额
    
    def 分析收益分配(self, 收益: float, 个人占比: float, 公司占比: float) -> Tuple[float, float, float]:
        """
        分析收益分配，计算个人收益、公司收益和非法所得
        
        Args:
            收益: 总收益金额
            个人占比: 个人资金占比
            公司占比: 公司资金占比
            
        Returns:
            (个人收益, 公司收益, 非法所得)
        """
        if 收益 <= 0:
            return 0.0, 0.0, 0.0
        
        个人收益 = 收益 * 个人占比
        公司收益 = 收益 * 公司占比
        非法所得 = 个人收益 + 公司收益
        
        # 格式化数值
        个人收益 = Config.format_number(个人收益)
        公司收益 = Config.format_number(公司收益)
        非法所得 = Config.format_number(非法所得)
        
        return 个人收益, 公司收益, 非法所得
    
    def 获取累计统计(self) -> Tuple[float, float]:
        """
        获取累计挪用和垫付金额
        
        Returns:
            (累计挪用金额, 累计垫付金额)
        """
        return self.累计挪用金额, self.累计垫付金额
    
    def 重置统计(self) -> None:
        """重置累计统计"""
        self.累计挪用金额 = 0.0
        self.累计垫付金额 = 0.0
        audit_logger.debug("行为分析器统计已重置") 