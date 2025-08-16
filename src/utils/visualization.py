"""
可视化模块
负责生成图表和报告
"""

import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
from typing import Dict, Any, Optional
import warnings
warnings.filterwarnings('ignore')

from config import Config
from utils.logger import audit_logger


class VisualizationGenerator:
    """可视化生成器"""
    
    def __init__(self):
        """初始化可视化生成器"""
        self._setup_matplotlib()
    
    def _setup_matplotlib(self):
        """设置matplotlib中文字体"""
        plt.rcParams['font.sans-serif'] = Config.CHINESE_FONTS
        plt.rcParams['axes.unicode_minus'] = False
    
    def 创建基础图表(self, df: pd.DataFrame, save_path: Optional[str] = None) -> None:
        """
        创建基础分析图表
        
        Args:
            df: 数据框
            save_path: 保存路径
        """
        try:
            # 创建图表
            fig, axes = plt.subplots(2, 2, figsize=Config.FIGURE_SIZE)
            fig.suptitle('FIFO资金追踪分析图表', fontsize=Config.FONT_SIZE)
            
            # 1. 个人和公司余额变化趋势
            self._plot_balance_trend(axes[0, 0], df)
            
            # 2. 资金流向类型分布
            self._plot_flow_distribution(axes[0, 1], df)
            
            # 3. 个人资金占比分布
            self._plot_personal_ratio_distribution(axes[1, 0], df)
            
            # 4. 总余额变化
            self._plot_total_balance_trend(axes[1, 1], df)
            
            plt.tight_layout()
            
            if save_path:
                plt.savefig(save_path, dpi=300, bbox_inches='tight')
                audit_logger.info(f"图表已保存到: {save_path}")
            
            plt.show()
            
        except Exception as e:
            audit_logger.error(e, "创建基础图表失败")
    
    def _plot_balance_trend(self, ax, df: pd.DataFrame):
        """绘制余额变化趋势"""
        if '交易日期' in df.columns and '个人余额' in df.columns and '公司余额' in df.columns:
            ax.plot(df['交易日期'], df['个人余额'], label='个人余额', alpha=0.7)
            ax.plot(df['交易日期'], df['公司余额'], label='公司余额', alpha=0.7)
            ax.set_title('个人和公司余额变化趋势')
            ax.set_xlabel('交易日期')
            ax.set_ylabel('余额')
            ax.legend()
            ax.tick_params(axis='x', rotation=45)
        else:
            ax.text(0.5, 0.5, '缺少必要数据列', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('个人和公司余额变化趋势')
    
    def _plot_flow_distribution(self, ax, df: pd.DataFrame):
        """绘制资金流向类型分布"""
        if '资金流向类型' in df.columns:
            try:
                流向统计 = df['资金流向类型'].value_counts()
                if len(流向统计) > 0:
                    ax.pie(流向统计.values, labels=流向统计.index, autopct='%1.1f%%')
                    ax.set_title('资金流向类型分布')
                else:
                    ax.text(0.5, 0.5, '无流向数据', ha='center', va='center', transform=ax.transAxes)
                    ax.set_title('资金流向类型分布')
            except Exception as e:
                ax.text(0.5, 0.5, f'绘图错误: {str(e)}', ha='center', va='center', transform=ax.transAxes)
                ax.set_title('资金流向类型分布')
        else:
            ax.text(0.5, 0.5, '缺少资金流向类型列', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('资金流向类型分布')
    
    def _plot_personal_ratio_distribution(self, ax, df: pd.DataFrame):
        """绘制个人资金占比分布"""
        if '个人资金占比' in df.columns:
            try:
                ax.hist(df['个人资金占比'], bins=20, alpha=0.7, color='blue')
                ax.set_title('个人资金占比分布')
                ax.set_xlabel('个人资金占比')
                ax.set_ylabel('频次')
            except Exception as e:
                ax.text(0.5, 0.5, f'绘图错误: {str(e)}', ha='center', va='center', transform=ax.transAxes)
                ax.set_title('个人资金占比分布')
        else:
            ax.text(0.5, 0.5, '缺少个人资金占比列', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('个人资金占比分布')
    
    def _plot_total_balance_trend(self, ax, df: pd.DataFrame):
        """绘制总余额变化趋势"""
        if '交易日期' in df.columns and '总余额' in df.columns:
            ax.plot(df['交易日期'], df['总余额'], color='green', alpha=0.7)
            ax.set_title('总余额变化趋势')
            ax.set_xlabel('交易日期')
            ax.set_ylabel('总余额')
            ax.tick_params(axis='x', rotation=45)
        else:
            ax.text(0.5, 0.5, '缺少必要数据列', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('总余额变化趋势')
    
    def 创建投资产品图表(self, 投资产品数据: Dict[str, Any], save_path: Optional[str] = None) -> None:
        """
        创建投资产品分析图表
        
        Args:
            投资产品数据: 投资产品数据字典
            save_path: 保存路径
        """
        try:
            if not 投资产品数据:
                audit_logger.warning("无投资产品数据，跳过图表创建")
                return
            
            fig, axes = plt.subplots(2, 2, figsize=(15, 12))
            fig.suptitle('投资产品分析图表', fontsize=16)
            
            # 1. 投资产品金额分布
            self._plot_investment_amount_distribution(axes[0, 0], 投资产品数据)
            
            # 2. 申购赎回对比
            self._plot_purchase_redemption_comparison(axes[0, 1], 投资产品数据)
            
            # 3. 个人公司占比对比
            self._plot_personal_company_ratio_comparison(axes[1, 0], 投资产品数据)
            
            # 4. 投资产品收益分析
            self._plot_investment_profit_analysis(axes[1, 1], 投资产品数据)
            
            plt.tight_layout()
            
            if save_path:
                plt.savefig(save_path, dpi=300, bbox_inches='tight')
                audit_logger.info(f"投资产品图表已保存到: {save_path}")
            
            plt.show()
            
        except Exception as e:
            audit_logger.error(e, "创建投资产品图表失败")
    
    def _plot_investment_amount_distribution(self, ax, 投资产品数据: Dict[str, Any]):
        """绘制投资产品金额分布"""
        try:
            产品名称 = list(投资产品数据.keys())
            总金额 = [info['总金额'] for info in 投资产品数据.values()]
            
            if 产品名称 and 总金额:
                ax.bar(产品名称, 总金额, alpha=0.7)
                ax.set_title('投资产品金额分布')
                ax.set_xlabel('投资产品')
                ax.set_ylabel('总金额')
                ax.tick_params(axis='x', rotation=45)
            else:
                ax.text(0.5, 0.5, '无投资产品数据', ha='center', va='center', transform=ax.transAxes)
                ax.set_title('投资产品金额分布')
        except Exception as e:
            ax.text(0.5, 0.5, f'绘图错误: {str(e)}', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('投资产品金额分布')
    
    def _plot_purchase_redemption_comparison(self, ax, 投资产品数据: Dict[str, Any]):
        """绘制申购赎回对比"""
        try:
            产品名称 = list(投资产品数据.keys())
            累计申购 = [info['累计申购'] for info in 投资产品数据.values()]
            累计赎回 = [info['累计赎回'] for info in 投资产品数据.values()]
            
            if 产品名称 and 累计申购 and 累计赎回:
                x = range(len(产品名称))
                width = 0.35
                
                ax.bar([i - width/2 for i in x], 累计申购, width, label='累计申购', alpha=0.7)
                ax.bar([i + width/2 for i in x], 累计赎回, width, label='累计赎回', alpha=0.7)
                
                ax.set_title('申购赎回对比')
                ax.set_xlabel('投资产品')
                ax.set_ylabel('金额')
                ax.set_xticks(x)
                ax.set_xticklabels(产品名称, rotation=45)
                ax.legend()
            else:
                ax.text(0.5, 0.5, '无申购赎回数据', ha='center', va='center', transform=ax.transAxes)
                ax.set_title('申购赎回对比')
        except Exception as e:
            ax.text(0.5, 0.5, f'绘图错误: {str(e)}', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('申购赎回对比')
    
    def _plot_personal_company_ratio_comparison(self, ax, 投资产品数据: Dict[str, Any]):
        """绘制个人公司占比对比"""
        try:
            产品名称 = list(投资产品数据.keys())
            个人占比 = [info['最新个人占比'] for info in 投资产品数据.values()]
            公司占比 = [info['最新公司占比'] for info in 投资产品数据.values()]
            
            if 产品名称 and 个人占比 and 公司占比:
                x = range(len(产品名称))
                width = 0.35
                
                ax.bar([i - width/2 for i in x], 个人占比, width, label='个人占比', alpha=0.7)
                ax.bar([i + width/2 for i in x], 公司占比, width, label='公司占比', alpha=0.7)
                
                ax.set_title('个人公司占比对比')
                ax.set_xlabel('投资产品')
                ax.set_ylabel('占比')
                ax.set_xticks(x)
                ax.set_xticklabels(产品名称, rotation=45)
                ax.legend()
            else:
                ax.text(0.5, 0.5, '无占比数据', ha='center', va='center', transform=ax.transAxes)
                ax.set_title('个人公司占比对比')
        except Exception as e:
            ax.text(0.5, 0.5, f'绘图错误: {str(e)}', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('个人公司占比对比')
    
    def _plot_investment_profit_analysis(self, ax, 投资产品数据: Dict[str, Any]):
        """绘制投资产品收益分析"""
        try:
            产品名称 = list(投资产品数据.keys())
            收益 = []
            
            for info in 投资产品数据.values():
                if '累计申购' in info and '累计赎回' in info:
                    收益.append(info['累计赎回'] - info['累计申购'])
                else:
                    收益.append(0)
            
            if 产品名称 and any(收益):
                colors = ['green' if p > 0 else 'red' if p < 0 else 'gray' for p in 收益]
                ax.bar(产品名称, 收益, color=colors, alpha=0.7)
                ax.set_title('投资产品收益分析')
                ax.set_xlabel('投资产品')
                ax.set_ylabel('收益')
                ax.tick_params(axis='x', rotation=45)
                ax.axhline(y=0, color='black', linestyle='-', alpha=0.3)
            else:
                ax.text(0.5, 0.5, '无收益数据', ha='center', va='center', transform=ax.transAxes)
                ax.set_title('投资产品收益分析')
        except Exception as e:
            ax.text(0.5, 0.5, f'绘图错误: {str(e)}', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('投资产品收益分析')
    
    def 创建异常交易图表(self, 异常分析: Dict[str, Any], save_path: Optional[str] = None) -> None:
        """
        创建异常交易分析图表
        
        Args:
            异常分析: 异常分析结果
            save_path: 保存路径
        """
        try:
            fig, axes = plt.subplots(2, 2, figsize=(15, 12))
            fig.suptitle('异常交易分析图表', fontsize=16)
            
            # 1. 异常类型统计
            self._plot_anomaly_type_statistics(axes[0, 0], 异常分析)
            
            # 2. 大额交易分布
            self._plot_large_amount_distribution(axes[0, 1], 异常分析)
            
            # 3. 异常时间分布
            self._plot_anomaly_time_distribution(axes[1, 0], 异常分析)
            
            # 4. 异常金额分布
            self._plot_anomaly_amount_distribution(axes[1, 1], 异常分析)
            
            plt.tight_layout()
            
            if save_path:
                plt.savefig(save_path, dpi=300, bbox_inches='tight')
                audit_logger.info(f"异常交易图表已保存到: {save_path}")
            
            plt.show()
            
        except Exception as e:
            audit_logger.error(e, "创建异常交易图表失败")
    
    def _plot_anomaly_type_statistics(self, ax, 异常分析: Dict[str, Any]):
        """绘制异常类型统计"""
        try:
            异常类型 = list(异常分析.keys())
            异常数量 = [len(异常分析[类型]) for 类型 in 异常类型]
            
            if 异常类型 and any(异常数量):
                ax.bar(异常类型, 异常数量, alpha=0.7)
                ax.set_title('异常类型统计')
                ax.set_xlabel('异常类型')
                ax.set_ylabel('数量')
                ax.tick_params(axis='x', rotation=45)
            else:
                ax.text(0.5, 0.5, '无异常数据', ha='center', va='center', transform=ax.transAxes)
                ax.set_title('异常类型统计')
        except Exception as e:
            ax.text(0.5, 0.5, f'绘图错误: {str(e)}', ha='center', va='center', transform=ax.transAxes)
            ax.set_title('异常类型统计')
    
    def _plot_large_amount_distribution(self, ax, 异常分析: Dict[str, Any]):
        """绘制大额交易分布"""
        # 这里需要实际的交易数据，暂时显示占位符
        ax.text(0.5, 0.5, '需要实际交易数据', ha='center', va='center', transform=ax.transAxes)
        ax.set_title('大额交易分布')
    
    def _plot_anomaly_time_distribution(self, ax, 异常分析: Dict[str, Any]):
        """绘制异常时间分布"""
        # 这里需要实际的时间数据，暂时显示占位符
        ax.text(0.5, 0.5, '需要实际时间数据', ha='center', va='center', transform=ax.transAxes)
        ax.set_title('异常时间分布')
    
    def _plot_anomaly_amount_distribution(self, ax, 异常分析: Dict[str, Any]):
        """绘制异常金额分布"""
        # 这里需要实际的金额数据，暂时显示占位符
        ax.text(0.5, 0.5, '需要实际金额数据', ha='center', va='center', transform=ax.transAxes)
        ax.set_title('异常金额分布')
    
    def 生成分析报告(self, 分析结果: Dict[str, Any], save_path: Optional[str] = None) -> str:
        """
        生成分析报告
        
        Args:
            分析结果: 分析结果字典
            save_path: 保存路径
            
        Returns:
            报告内容
        """
        try:
            报告内容 = []
            报告内容.append("=" * 80)
            报告内容.append("FIFO资金追踪分析报告")
            报告内容.append("=" * 80)
            报告内容.append("")
            
            # 基本信息
            if '基本信息' in 分析结果:
                报告内容.append("【基本信息】")
                for key, value in 分析结果['基本信息'].items():
                    报告内容.append(f"  {key}: {value}")
                报告内容.append("")
            
            # 余额状况
            if '余额状况' in 分析结果:
                报告内容.append("【余额状况】")
                for key, value in 分析结果['余额状况'].items():
                    报告内容.append(f"  {key}: {value:,.2f}")
                报告内容.append("")
            
            # 挪用垫付情况
            if '挪用垫付情况' in 分析结果:
                报告内容.append("【挪用垫付情况】")
                for key, value in 分析结果['挪用垫付情况'].items():
                    报告内容.append(f"  {key}: {value:,.2f}")
                报告内容.append("")
            
            # 投资产品情况
            if '投资产品情况' in 分析结果:
                报告内容.append("【投资产品情况】")
                for key, value in 分析结果['投资产品情况'].items():
                    报告内容.append(f"  {key}: {value}")
                报告内容.append("")
            
            # 异常交易情况
            if '异常交易情况' in 分析结果:
                报告内容.append("【异常交易情况】")
                for key, value in 分析结果['异常交易情况'].items():
                    报告内容.append(f"  {key}: {value}")
                报告内容.append("")
            
            报告内容.append("=" * 80)
            报告内容.append("报告生成完成")
            报告内容.append("=" * 80)
            
            完整报告 = "\n".join(报告内容)
            
            if save_path:
                with open(save_path, 'w', encoding='utf-8') as f:
                    f.write(完整报告)
                audit_logger.info(f"分析报告已保存到: {save_path}")
            
            return 完整报告
            
        except Exception as e:
            audit_logger.error(e, "生成分析报告失败")
            return f"生成报告失败: {str(e)}" 