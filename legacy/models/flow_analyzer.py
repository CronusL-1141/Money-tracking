"""
资金流向分析器模块
负责分析资金流向类型和方向
"""

from typing import Tuple, Optional
import pandas as pd
from config import Config
from utils.logger import audit_logger


class FlowAnalyzer:
    """资金流向分析器"""
    
    def __init__(self):
        """初始化资金流向分析器"""
        pass
    
    def 分析交易方向(self, 交易收入金额: float, 交易支出金额: float) -> Tuple[float, str]:
        """
        分析交易方向和金额
        
        Args:
            交易收入金额: 交易收入金额
            交易支出金额: 交易支出金额
            
        Returns:
            (实际金额, 方向)
        """
        # 确定实际交易金额和方向
        if 交易收入金额 > 0 and 交易支出金额 == 0:
            # 只有收入有值，可能是收入
            实际金额 = 交易收入金额
            方向 = '收入'
        elif 交易收入金额 == 0 and 交易支出金额 > 0:
            # 只有支出有值，可能是支出
            实际金额 = 交易支出金额
            方向 = '支出'
        elif 交易收入金额 > 0 and 交易支出金额 > 0:
            # 两个都有值，取较大的作为实际金额
            实际金额 = max(交易收入金额, 交易支出金额)
            方向 = '收入' if 交易收入金额 > 交易支出金额 else '支出'
        else:
            实际金额 = 0.0
            方向 = '无'
        
        return 实际金额, 方向
    
    def 分析资金流向类型(self, 方向: str, 资金属性: str) -> str:
        """
        分析资金流向类型
        
        Args:
            方向: 交易方向（'收入'、'支出'、'无'）
            资金属性: 资金属性描述
            
        Returns:
            资金流向类型
        """
        if 方向 == '收入':
            # 检查是否为投资产品赎回
            if Config.is_investment_product(资金属性):
                前缀 = 资金属性.split('-')[0]
                return f'{前缀}赎回'
            else:
                return '资金流入'
        elif 方向 == '支出':
            # 检查是否为投资产品申购
            if Config.is_investment_product(资金属性):
                前缀 = 资金属性.split('-')[0]
                return f'{前缀}申购'
            else:
                return '资金支出'
        else:
            return '无交易'
    
    def 解析投资产品编号(self, 资金属性: str) -> Optional[str]:
        """
        解析投资产品的编号（理财、投资、保险、关联银行卡等）
        
        Args:
            资金属性: 资金属性描述
            
        Returns:
            投资产品编号，如果不是投资产品返回None
        """
        if Config.is_investment_product(资金属性):
            return 资金属性  # 返回完整的产品编号
        return None
    
    def 解析交易时间(self, 时间值) -> str:
        """
        将交易时间转换为时分秒格式
        
        Args:
            时间值: 原始时间值
            
        Returns:
            格式化的时间字符串
        """
        if pd.isna(时间值) or 时间值 == 0:
            return "00:00:00"
        
        try:
            # 如果是datetime.time对象，直接转换
            if hasattr(时间值, 'hour'):
                return f"{时间值.hour:02d}:{时间值.minute:02d}:{时间值.second:02d}"
            
            # 如果是数字，按原逻辑处理
            时间数字 = int(时间值)
            if 时间数字 == 0:
                return "00:00:00"
            
            时间str = str(时间数字).zfill(6)  # 补齐到6位
            小时 = 时间str[:2]
            分钟 = 时间str[2:4]
            秒 = 时间str[4:6]
            
            # 验证时间有效性
            if int(小时) >= 24 or int(分钟) >= 60 or int(秒) >= 60:
                return "00:00:00"
            return f"{小时}:{分钟}:{秒}"
        except:
            return "00:00:00"
    
    def 创建完整时间戳(self, 交易日期: pd.Timestamp, 交易时间: str) -> pd.Timestamp:
        """
        创建完整的交易时间戳
        
        Args:
            交易日期: 交易日期
            交易时间: 格式化的交易时间
            
        Returns:
            完整的交易时间戳
        """
        try:
            完整时间戳 = pd.to_datetime(
                交易日期.strftime('%Y-%m-%d') + ' ' + 交易时间
            )
            return 完整时间戳
        except Exception as e:
            audit_logger.warning(f"创建时间戳失败: {e}")
            return 交易日期
    
    def 分析异常交易(self, df: pd.DataFrame) -> dict:
        """
        分析异常交易
        
        Args:
            df: 交易数据框
            
        Returns:
            异常交易分析结果
        """
        异常分析 = {
            '大额交易': [],
            '同时收支': [],
            '异常时间': [],
            '异常金额': []
        }
        
        # 分析大额交易
        for amount_col in ['交易收入金额', '交易支出金额']:
            if amount_col in df.columns:
                大额交易 = df[df[amount_col] > Config.LARGE_AMOUNT_THRESHOLD]
                if len(大额交易) > 0:
                    异常分析['大额交易'].extend(大额交易.index.tolist())
        
        # 分析同时有收入和支出的交易
        if '交易收入金额' in df.columns and '交易支出金额' in df.columns:
            同时收支 = df[
                (df['交易收入金额'].notna() & (df['交易收入金额'] > 0)) &
                (df['交易支出金额'].notna() & (df['交易支出金额'] > 0))
            ]
            if len(同时收支) > 0:
                异常分析['同时收支'].extend(同时收支.index.tolist())
        
        # 分析异常金额（负数或零）
        for amount_col in ['交易收入金额', '交易支出金额']:
            if amount_col in df.columns:
                异常金额 = df[
                    (df[amount_col] < 0) | 
                    (df[amount_col].isna())
                ]
                if len(异常金额) > 0:
                    异常分析['异常金额'].extend(异常金额.index.tolist())
        
        return 异常分析
    
    def 生成流向统计(self, df: pd.DataFrame) -> dict:
        """
        生成资金流向统计
        
        Args:
            df: 交易数据框
            
        Returns:
            流向统计字典
        """
        if '资金流向类型' not in df.columns:
            return {}
        
        流向统计 = df['资金流向类型'].value_counts().to_dict()
        
        # 按类型分组统计
        收入统计 = df[df['交易收入金额'] > 0]['交易收入金额'].sum()
        支出统计 = df[df['交易支出金额'] > 0]['交易支出金额'].sum()
        
        return {
            '流向类型统计': 流向统计,
            '总收入': Config.format_number(收入统计),
            '总支出': Config.format_number(支出统计),
            '净流入': Config.format_number(收入统计 - 支出统计),
            '交易笔数': len(df)
        }
    
    def 分析资金流向趋势(self, df: pd.DataFrame) -> dict:
        """
        分析资金流向趋势
        
        Args:
            df: 交易数据框
            
        Returns:
            趋势分析结果
        """
        if '完整时间戳' not in df.columns:
            return {}
        
        # 按日期分组统计
        df['交易日期'] = pd.to_datetime(df['完整时间戳']).dt.date
        日期统计 = df.groupby('交易日期').agg({
            '交易收入金额': 'sum',
            '交易支出金额': 'sum'
        }).reset_index()
        
        # 计算每日净流入
        日期统计['净流入'] = 日期统计['交易收入金额'] - 日期统计['交易支出金额']
        
        return {
            '日期统计': 日期统计.to_dict('records'),
            '最大单日收入': Config.format_number(日期统计['交易收入金额'].max()),
            '最大单日支出': Config.format_number(日期统计['交易支出金额'].max()),
            '最大单日净流入': Config.format_number(日期统计['净流入'].max()),
            '最大单日净流出': Config.format_number(日期统计['净流入'].min())
        } 