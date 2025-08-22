"""
投资产品管理器模块
负责管理投资产品的资金池和相关信息
"""

from typing import Dict, Any, Optional, Tuple
from config import Config
from utils.logger import audit_logger


class InvestmentProductManager:
    """投资产品管理器"""
    
    def __init__(self):
        """初始化投资产品管理器"""
        self.投资产品资金池 = {}
    
    def 创建投资产品(self, 产品编号: str) -> Dict[str, Any]:
        """
        创建新的投资产品资金池
        
        Args:
            产品编号: 投资产品编号
            
        Returns:
            产品信息字典
        """
        if 产品编号 not in self.投资产品资金池:
            self.投资产品资金池[产品编号] = {
                '个人金额': 0.0, 
                '公司金额': 0.0, 
                '总金额': 0.0,
                '累计申购': 0.0,
                '累计赎回': 0.0,
                '最新个人占比': 0.0,
                '最新公司占比': 0.0
            }
            # audit_logger.info(f"创建投资产品资金池: {产品编号}")
        
        return self.投资产品资金池[产品编号]
    
    def 更新投资产品(self, 产品编号: str, 金额: float, 个人占比: float, 公司占比: float) -> None:
        """
        更新投资产品资金池（申购时）
        
        Args:
            产品编号: 投资产品编号
            金额: 申购金额
            个人占比: 个人资金占比
            公司占比: 公司资金占比
        """
        if 产品编号 not in self.投资产品资金池:
            self.创建投资产品(产品编号)
        
        产品信息 = self.投资产品资金池[产品编号]
        个人金额 = 金额 * 个人占比
        公司金额 = 金额 * 公司占比
        
        # 检查当前资金池状态
        当前总金额 = 产品信息['总金额']
        
        if 当前总金额 < 0:
            # 资金池为负数，说明之前有收益，再次申购时重置资金池
            # audit_logger.info(f"投资产品{产品编号}资金池为负数({当前总金额:,.2f})，重置为新申购")
            产品信息['个人金额'] = 个人金额
            产品信息['公司金额'] = 公司金额
            产品信息['总金额'] = 金额
            # 更新占比记录（与新入资的占比一致）
            产品信息['最新个人占比'] = 个人占比
            产品信息['最新公司占比'] = 公司占比
        else:
            # 正常累加新申购金额
            产品信息['个人金额'] += 个人金额
            产品信息['公司金额'] += 公司金额
            产品信息['总金额'] += 金额
            # 更新占比记录（按新的总金额计算）
            新总金额 = 产品信息['总金额']
            if 新总金额 > 0:
                产品信息['最新个人占比'] = 产品信息['个人金额'] / 新总金额
                产品信息['最新公司占比'] = 产品信息['公司金额'] / 新总金额
        
        产品信息['累计申购'] += 金额
        
        audit_logger.log_investment_product(
            product_id=产品编号,
            action="申购",
            amount=金额,
            personal_amount=个人金额,
            company_amount=公司金额
        )
    
    def 处理投资产品赎回(self, 产品编号: str, 金额: float) -> Tuple[float, float, float, str]:
        """
        处理投资产品赎回
        
        Args:
            产品编号: 投资产品编号
            金额: 赎回金额
            
        Returns:
            (个人返还, 公司返还, 收益, 行为性质)
        """
        if 产品编号 not in self.投资产品资金池:
            # 没有找到对应的投资产品记录
            return 0.0, 0.0, 0.0, f"错误：投资产品{产品编号}不存在"
        
        产品信息 = self.投资产品资金池[产品编号]
        总金额 = 产品信息['总金额']
        最新个人占比 = 产品信息['最新个人占比']
        最新公司占比 = 产品信息['最新公司占比']
        
        # 检查是否有有效的占比记录
        if 最新个人占比 == 0 and 最新公司占比 == 0:
            return 0.0, 0.0, 0.0, f"错误：投资产品{产品编号}从未有过有效资金池，无法分配收益"
        
        # 赎回时统一使用最新记录的占比
        个人返还 = 金额 * 最新个人占比
        公司返还 = 金额 * 最新公司占比
        
        # 计算收益情况
        if 总金额 > 0:
            # 正常赎回逻辑
            赎回比例 = 金额 / 总金额 if 总金额 > 0 else 0
            对应申购成本 = 总金额 * 赎回比例
            收益 = 金额 - 对应申购成本
            
            # 更新投资产品资金池
            产品信息['个人金额'] -= 产品信息['个人金额'] * 赎回比例
            产品信息['公司金额'] -= 产品信息['公司金额'] * 赎回比例
            产品信息['总金额'] -= 对应申购成本
        else:
            # 资金池为0或负数，纯收益分配
            收益 = 金额
            
            # 更新资金池（继续减少，保持负数状态）
            产品信息['个人金额'] -= 个人返还
            产品信息['公司金额'] -= 公司返还
            产品信息['总金额'] -= 金额
        
        产品信息['累计赎回'] += 金额
        
        # 构造行为性质描述
        前缀 = 产品编号.split('-')[0]
        if 收益 > 0:
            if 最新个人占比 > 0 and 最新公司占比 > 0:
                行为性质 = f"{前缀}赎回-{产品编号}：个人{个人返还:,.2f}，公司{公司返还:,.2f}，收益{收益:,.2f}"
            elif 最新个人占比 > 0:
                行为性质 = f"{前缀}赎回-{产品编号}：个人{个人返还:,.2f}，收益{收益:,.2f}"
            else:
                行为性质 = f"{前缀}赎回-{产品编号}：公司{公司返还:,.2f}，收益{收益:,.2f}"
        elif 收益 < 0:
            行为性质 = f"{前缀}赎回-{产品编号}：个人{个人返还:,.2f}，公司{公司返还:,.2f}，亏损{abs(收益):,.2f}"
        else:
            行为性质 = f"{前缀}赎回-{产品编号}：个人{个人返还:,.2f}，公司{公司返还:,.2f}，无收益"
        
        return 个人返还, 公司返还, 收益, 行为性质
    
    def 获取投资产品信息(self, 产品编号: str) -> Optional[Dict[str, Any]]:
        """
        获取投资产品信息
        
        Args:
            产品编号: 投资产品编号
            
        Returns:
            产品信息字典，如果不存在返回None
        """
        return self.投资产品资金池.get(产品编号)
    
    def 获取所有投资产品(self) -> Dict[str, Dict[str, Any]]:
        """
        获取所有投资产品信息
        
        Returns:
            所有投资产品信息字典
        """
        return self.投资产品资金池.copy()
    
    def 验证投资产品数据(self, 产品编号: str) -> list:
        """
        验证投资产品数据的一致性
        
        Args:
            产品编号: 投资产品编号
            
        Returns:
            错误信息列表
        """
        if 产品编号 not in self.投资产品资金池:
            return [f"投资产品{产品编号}不存在"]
        
        产品信息 = self.投资产品资金池[产品编号]
        errors = []
        
        # 检查总金额计算
        calculated_total = 产品信息['个人金额'] + 产品信息['公司金额']
        actual_total = 产品信息['总金额']
        
        if abs(calculated_total - actual_total) > Config.EPSILON:
            errors.append(
                f"总金额计算错误："
                f"个人金额({产品信息['个人金额']:,.2f}) + "
                f"公司金额({产品信息['公司金额']:,.2f}) = "
                f"{calculated_total:,.2f} ≠ "
                f"总金额({actual_total:,.2f})"
            )
        
        # 检查占比计算
        if actual_total > 0:
            expected_personal_ratio = 产品信息['个人金额'] / actual_total
            expected_company_ratio = 产品信息['公司金额'] / actual_total
            
            if abs(expected_personal_ratio - 产品信息['最新个人占比']) > Config.EPSILON:
                errors.append(
                    f"个人占比计算错误："
                    f"期望 {expected_personal_ratio:.4f}，"
                    f"实际 {产品信息['最新个人占比']:.4f}"
                )
            
            if abs(expected_company_ratio - 产品信息['最新公司占比']) > Config.EPSILON:
                errors.append(
                    f"公司占比计算错误："
                    f"期望 {expected_company_ratio:.4f}，"
                    f"实际 {产品信息['最新公司占比']:.4f}"
                )
        
        return errors
    
    def 获取投资产品统计(self) -> Dict[str, Any]:
        """
        获取投资产品统计信息
        
        Returns:
            统计信息字典
        """
        total_products = len(self.投资产品资金池)
        total_investment = sum(info['总金额'] for info in self.投资产品资金池.values())
        total_purchase = sum(info['累计申购'] for info in self.投资产品资金池.values())
        total_redemption = sum(info['累计赎回'] for info in self.投资产品资金池.values())
        
        return {
            '产品总数': total_products,
            '总投资金额': Config.format_number(total_investment),
            '累计申购': Config.format_number(total_purchase),
            '累计赎回': Config.format_number(total_redemption),
            '净投资': Config.format_number(total_purchase - total_redemption)
        }
    
    def 清理无效产品(self) -> int:
        """
        清理无效的投资产品（总金额为0且无申购赎回记录）
        
        Returns:
            清理的产品数量
        """
        清理数量 = 0
        待清理 = []
        
        for 产品编号, 产品信息 in self.投资产品资金池.items():
            if (abs(产品信息['总金额']) < Config.EPSILON and 
                产品信息['累计申购'] == 0 and 
                产品信息['累计赎回'] == 0):
                待清理.append(产品编号)
        
        for 产品编号 in 待清理:
            del self.投资产品资金池[产品编号]
            清理数量 += 1
            audit_logger.info(f"清理无效投资产品: {产品编号}")
        
        return 清理数量 