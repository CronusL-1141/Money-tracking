"""
数据处理器模块
负责数据预处理、验证和格式化
"""

import pandas as pd
import numpy as np
from typing import Optional, Dict, Any
import warnings
warnings.filterwarnings('ignore')

from config import Config
from utils.logger import audit_logger
from utils.validators import DataValidator
from models.flow_analyzer import FlowAnalyzer


class DataProcessor:
    """数据处理器"""
    
    def __init__(self):
        """初始化数据处理器"""
        self.flow_analyzer = FlowAnalyzer()
        self.validator = DataValidator()
    
    def 预处理财务数据(self, file_path: str) -> Optional[pd.DataFrame]:
        """
        预处理财务数据：读取Excel文件，处理时间，按时间排序，初始化结果列
        
        Args:
            file_path: Excel文件路径
            
        Returns:
            预处理后的数据框，失败返回None
        """
        try:
            # 读取Excel文件
            audit_logger.info("正在读取Excel文件...")
            df = pd.read_excel(file_path)
            
            audit_logger.info(f"数据总行数: {len(df)}")
            audit_logger.info(f"数据总列数: {len(df.columns)}")
            
            # 检查列名
            self._check_columns(df)
            
            # 数据预处理
            audit_logger.info("正在预处理数据...")
            
            # 确保交易日期是datetime类型
            df['交易日期'] = pd.to_datetime(df['交易日期'])
            
            # 处理交易时间
            df['交易时间_格式化'] = df['交易时间'].apply(self.flow_analyzer.解析交易时间)
            
            # 创建完整时间戳
            df['完整时间戳'] = df.apply(
                lambda row: self.flow_analyzer.创建完整时间戳(row['交易日期'], row['交易时间_格式化']), 
                axis=1
            )
            
            # 添加原始索引列，用于保持相同时间交易的原始顺序
            df['原始索引'] = df.index
            
            # 按完整时间戳排序，相同时间的交易保持原始顺序
            df = df.sort_values(['完整时间戳', '原始索引'], kind='stable').reset_index(drop=True)
            
            # 删除临时的原始索引列
            df = df.drop('原始索引', axis=1)
            
            # 重新整理列顺序
            df = self._reorder_columns(df)
            
            # 显示时间戳示例
            self._show_timestamp_examples(df)
            
            # 初始化结果列
            df = self._initialize_result_columns(df)
            
            audit_logger.info("数据预处理完成")
            return df
            
        except Exception as e:
            audit_logger.error(f"数据预处理失败: {e}")
            import traceback
            traceback.print_exc()
            return None
    
    def _check_columns(self, df: pd.DataFrame) -> None:
        """检查数据框的列名"""
        audit_logger.info("=" * 60)
        audit_logger.info("列名检查")
        audit_logger.info("=" * 60)
        audit_logger.info("所有列名:")
        for i, col in enumerate(df.columns):
            audit_logger.info(f"{i+1}. '{col}' (类型: {df[col].dtype})")
        
        audit_logger.info(f"前5行数据:")
        audit_logger.info(df.head().to_string())
        audit_logger.info(f"数据信息:")
        # df.info() 返回None，改为输出有用的信息
        audit_logger.info(f"数据形状: {df.shape}, 列数: {len(df.columns)}, 内存使用: {df.memory_usage().sum()} bytes")
    
    def _reorder_columns(self, df: pd.DataFrame) -> pd.DataFrame:
        """重新排列列顺序，删除多余的日期时间字段，只保留完整时间戳"""
        原列顺序 = df.columns.tolist()
        新列顺序 = ['完整时间戳']  # 先添加完整时间戳作为第一列
        
        # 添加其他列（排除要删除的日期时间字段）
        要删除的字段 = ['交易日期', '交易时间', '交易时间_格式化']
        
        for col in 原列顺序:
            if col not in 要删除的字段 and col != '完整时间戳':
                新列顺序.append(col)
        
        # 重新排列DataFrame列顺序，并删除不需要的列
        return df.reindex(columns=新列顺序)
    
    def _show_timestamp_examples(self, df: pd.DataFrame) -> None:
        """显示时间戳示例"""
        audit_logger.info(f"完整时间戳示例:")
        if '完整时间戳' in df.columns:
            示例数据 = df[['完整时间戳']].head()
            audit_logger.info(示例数据.to_string())
        else:
            audit_logger.warning("完整时间戳列不存在")
    
    def _initialize_result_columns(self, df: pd.DataFrame) -> pd.DataFrame:
        """初始化结果列"""
        # 初始化结果列
        df['个人资金占比'] = 0.0
        df['公司资金占比'] = 0.0
        df['行为性质'] = ''
        df['累计挪用'] = 0.0
        df['累计垫付'] = 0.0
        df['累计由资金池回归公司余额本金'] = 0.0
        df['累计由资金池回归个人余额本金'] = 0.0
        df['累计非法所得'] = 0.0
        df['总计个人应分配利润'] = 0.0
        df['总计公司应分配利润'] = 0.0
        df['个人余额'] = 0.0
        df['公司余额'] = 0.0
        df['总余额'] = 0.0
        df['资金缺口'] = 0.0  # 新合并字段：替换公司应还和个人应还
        

        
        return df
    
    def 验证数据完整性(self, df: pd.DataFrame) -> Dict[str, Any]:
        """
        验证数据完整性
        
        Args:
            df: 数据框
            
        Returns:
            验证结果
        """
        return self.validator.validate_dataframe(df)
    
    def 处理单行交易(self, row: pd.Series, row_idx: int) -> Dict[str, Any]:
        """
        处理单行交易数据
        
        Args:
            row: 交易数据行
            row_idx: 行索引
            
        Returns:
            处理结果字典
        """
        try:
            # 提取数据
            交易收入金额 = float(row['交易收入金额']) if not np.isnan(row['交易收入金额']) else 0.0
            交易支出金额 = float(row['交易支出金额']) if not np.isnan(row['交易支出金额']) else 0.0
            资金属性 = str(row['资金属性']) if row['资金属性'] is not None and str(row['资金属性']) != 'nan' else ''
            完整时间戳 = row['完整时间戳']
            
            # 分析交易方向
            实际金额, 方向 = self.flow_analyzer.分析交易方向(交易收入金额, 交易支出金额)
            
            return {
                'row_idx': row_idx,
                '交易收入金额': 交易收入金额,
                '交易支出金额': 交易支出金额,
                '实际金额': 实际金额,
                '方向': 方向,
                '资金属性': 资金属性,
                '完整时间戳': 完整时间戳,
                'is_investment': Config.is_investment_product(资金属性)
            }
            
        except Exception as e:
            audit_logger.error(f"处理第{row_idx}行数据时出错: {e}")
            return {
                'row_idx': row_idx,
                'error': str(e),
                '实际金额': 0.0,
                '方向': '无'
            }
    
    def 批量处理交易(self, df: pd.DataFrame, start_idx: int = 0, end_idx: Optional[int] = None) -> list:
        """
        批量处理交易数据
        
        Args:
            df: 数据框
            start_idx: 开始索引
            end_idx: 结束索引（不包含）
            
        Returns:
            处理结果列表
        """
        if end_idx is None:
            end_idx = len(df)
        
        结果列表 = []
        
        for i in range(start_idx, end_idx):
            if i % Config.PROGRESS_INTERVAL == 0:
                audit_logger.info(f"处理进度: {i}/{len(df)}")
            
            row = df.iloc[i]
            结果 = self.处理单行交易(row, i)
            结果列表.append(结果)
        
        return 结果列表
    
    def 更新结果列(self, df: pd.DataFrame, row_idx: int, 结果: Dict[str, Any]) -> None:
        """
        更新数据框的结果列
        
        Args:
            df: 数据框
            row_idx: 行索引
            结果: 处理结果
        """
        if 'error' in 结果:
            return
    
    def 计算初始余额(self, df: pd.DataFrame, silent: bool = False) -> float:
        """
        计算初始余额
        
        Args:
            df: 数据框
            silent: 是否静默模式（不输出日志）
            
        Returns:
            初始余额
        """
        if len(df) == 0:
            return 0.0
        
        第一笔余额 = df.iloc[0]['余额'] if pd.notna(df.iloc[0]['余额']) else 0.0
        第一笔交易金额 = 0.0
        
        if pd.notna(df.iloc[0]['交易收入金额']):
            第一笔交易金额 = df.iloc[0]['交易收入金额']
        elif pd.notna(df.iloc[0]['交易支出金额']):
            第一笔交易金额 = -df.iloc[0]['交易支出金额']
        
        # 计算交易前余额（初始余额）
        初始余额 = 第一笔余额 - 第一笔交易金额
        
        # 非静默模式下才输出日志
        if 初始余额 > 0 and not silent:
            audit_logger.info(f"计算得出初始余额: {初始余额:,.2f}（第一笔余额{第一笔余额:,.2f} - 第一笔交易{第一笔交易金额:,.2f}）")
            audit_logger.info(f"将初始余额作为公司应收在FIFO账本簿中初始化")
        
        return 初始余额
    
    def 生成数据摘要(self, df: pd.DataFrame) -> Dict[str, Any]:
        """
        生成数据摘要
        
        Args:
            df: 数据框
            
        Returns:
            数据摘要字典
        """
        摘要 = {
            '总行数': len(df),
            '总列数': len(df.columns),
            '时间范围': {},
            '金额统计': {},
            '流向统计': {}
        }
        
        # 时间范围
        if '完整时间戳' in df.columns:
            摘要['时间范围'] = {
                '开始时间': df['完整时间戳'].min(),
                '结束时间': df['完整时间戳'].max(),
                '总天数': (df['完整时间戳'].max() - df['完整时间戳'].min()).days
            }
        
        # 金额统计
        if '交易收入金额' in df.columns and '交易支出金额' in df.columns:
            摘要['金额统计'] = {
                '总收入': Config.format_number(df['交易收入金额'].sum()),
                '总支出': Config.format_number(df['交易支出金额'].sum()),
                '净流入': Config.format_number(df['交易收入金额'].sum() - df['交易支出金额'].sum()),
                '最大收入': Config.format_number(df['交易收入金额'].max()),
                '最大支出': Config.format_number(df['交易支出金额'].max())
            }
        
        # 流向统计

        
        return 摘要
    
    def 保存结果(self, df: pd.DataFrame, output_file: Optional[str] = None) -> bool:
        """
        保存处理结果
        
        Args:
            df: 数据框
            output_file: 输出文件名
            
        Returns:
            是否保存成功
        """
        if output_file is None:
            output_file = Config.DEFAULT_OUTPUT_FILE
        
        try:
            df.to_excel(output_file, index=False)
            audit_logger.info(f"分析结果已保存到: {output_file}")
            return True
        except Exception as e:
            audit_logger.error(f"保存结果失败: {output_file}, 错误: {e}")
            return False 

 