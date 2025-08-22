"""
主程序入口
整合所有模块，提供完整的FIFO资金追踪分析功能
"""

import time
import pandas as pd
from typing import Optional, Dict, Any

from config import Config
from utils.logger import audit_logger
from utils.data_processor import DataProcessor
from utils.visualization import VisualizationGenerator
from utils.flow_integrity_validator import FlowIntegrityValidator
from models.fifo_tracker import FIFO资金追踪器
from models.behavior_analyzer import BehaviorAnalyzer
from models.investment_manager import InvestmentProductManager
from models.flow_analyzer import FlowAnalyzer


class FIFO资金追踪分析器:
    """FIFO资金追踪分析器主类"""
    
    def __init__(self):
        """初始化分析器"""
        self.data_processor = DataProcessor()
        self.visualization = VisualizationGenerator()
        self.flow_validator = FlowIntegrityValidator()
        self.tracker = FIFO资金追踪器()
        self.behavior_analyzer = BehaviorAnalyzer()
        self.investment_manager = InvestmentProductManager()
        self.flow_analyzer = FlowAnalyzer()
        
        audit_logger.info("FIFO资金追踪分析器初始化完成")
    
    def 分析财务数据(self, file_path: str, output_file: Optional[str] = None) -> Optional[pd.DataFrame]:
        """
        分析财务数据，实现FIFO原则的资金追踪
        
        Args:
            file_path: Excel文件路径
            output_file: 输出文件名
            
        Returns:
            分析结果数据框，失败返回None
        """
        start_time = time.time()
        
        try:
            audit_logger.info("=" * 60)
            audit_logger.info("公款挪用与职务侵占审计分析 - FIFO资金追踪")
            audit_logger.info("=" * 60)
            
            # 1. 数据预处理
            df = self.data_processor.预处理财务数据(file_path)
            if df is None:
                return None
            
            # 2. 原始流水完整性验证
            validation_result = self.flow_validator.validate_flow_integrity(df)
            if not validation_result['is_valid']:
                audit_logger.warning(f"流水完整性验证发现{validation_result['errors_count']}个问题")
                
                if validation_result['optimization_failed']:
                    audit_logger.error("❌ 流水优化失败，无法自动修复数据完整性问题")
                    audit_logger.error("请检查源数据文件，可能存在缺失交易或计算错误")
                    
                    # 保存错误报告
                    error_report_file = "流水验证错误报告.txt"
                    self._save_error_report(validation_result, error_report_file)
                    audit_logger.info(f"📄 错误详情已保存至: {error_report_file}")
                    
                    return None  # 停止处理
                
                if validation_result['optimizations_count'] > 0:
                    audit_logger.info(f"已通过重排序修复{validation_result['optimizations_count']}个问题")
                    # 使用修复后的数据框进行后续处理
                    df = validation_result['result_dataframe']
                    audit_logger.info("✅ 使用修复后的数据继续处理（源文件保持不变）")
                
                # 如果仍有未修复的错误，显示详情但继续处理
                remaining_errors = validation_result['errors_count'] - validation_result['optimizations_count']
                if remaining_errors > 0:
                    audit_logger.warning(f"仍有{remaining_errors}个错误无法自动修复，建议人工核查")
            else:
                audit_logger.info("✅ 流水完整性验证通过")
            
            # 3. 数据验证
            validation_result = self.data_processor.验证数据完整性(df)
            if not validation_result['is_valid']:
                audit_logger.warning("数据验证发现问题，但继续处理")
                for error in validation_result['errors'][:5]:  # 只显示前5个错误
                    audit_logger.warning(error)  # error 本身就是字符串
            
            # 3. 计算初始余额
            初始余额 = self.data_processor.计算初始余额(df)
            if 初始余额 > 0:
                self.tracker.初始化余额(初始余额, '公司')
            
            # 4. 逐笔处理交易
            audit_logger.info("开始FIFO资金追踪分析...")
            self._process_transactions(df)
            
            # 5. 生成分析结果
            audit_logger.info("FIFO资金追踪完成！")
            self._generate_analysis_results(df)  # 显示详细分析结果
            
            # 6. 保存结果
            if output_file is None:
                output_file = Config.DEFAULT_OUTPUT_FILE
            
            # 如果数据被修复过，添加提示
            if validation_result.get('has_modifications', False):
                audit_logger.info("💾 保存修复后的数据（原始源文件保持不变）")
            
            self.data_processor.保存结果(df, output_file)
            
            # 6.5. 生成投资产品交易记录Excel
            self.tracker.生成投资产品交易记录Excel()
            
            # 7. 生成可视化
            # self._generate_visualizations(df)  # 注释掉可视化图表生成
            
            # 8. 生成报告
            # self._generate_report(df)  # 注释掉分析报告生成
            
            processing_time = time.time() - start_time
            audit_logger.log_performance("完整分析", processing_time, len(df))
            
            # 显示流水验证完成信息
            audit_logger.info("流水数据处理完成")
            
            return df
            
        except Exception as e:
            audit_logger.log_error(e, "分析财务数据失败")
            import traceback
            traceback.print_exc()
            return None
    
    def _process_transactions(self, df: pd.DataFrame) -> None:
        """处理所有交易"""
        for i, (idx, row) in enumerate(df.iterrows()):
            if i % Config.PROGRESS_INTERVAL == 0:
                audit_logger.info(f"处理进度: {i}/{len(df)}")
            
            # 处理单行交易
            处理结果 = self.data_processor.处理单行交易(row, i)
            
            # 根据交易方向处理
            if 处理结果['方向'] == '收入':
                self._process_income_transaction(row, 处理结果, df, i)
            elif 处理结果['方向'] == '支出':
                self._process_expense_transaction(row, 处理结果, df, i)
            else:
                self._process_no_transaction(row, 处理结果, df, i)
            
            # 更新结果列
            self._update_result_columns(df, i)
            
            # 注意：流水完整性验证已在预处理阶段完成
            # 此处只进行FIFO业务逻辑处理，不再验证原始数据完整性
    
    def _process_income_transaction(self, row: pd.Series, 处理结果: Dict[str, Any], df: pd.DataFrame, row_idx: int) -> None:
        """处理收入交易"""
        # 检查是否为投资产品赎回
        if 处理结果['is_investment']:
            # 投资产品赎回
            个人占比, 公司占比, 行为性质 = self.tracker.处理投资产品赎回(
                处理结果['实际金额'], 
                处理结果['资金属性'], 
                处理结果['完整时间戳']
            )
        else:
            # 普通收入
            个人占比, 公司占比, 行为性质 = self.tracker.处理资金流入(
                处理结果['实际金额'], 
                处理结果['资金属性'], 
                处理结果['完整时间戳']
            )
        
        # 记录结果
        df.iloc[row_idx, df.columns.get_loc('个人资金占比')] = 个人占比
        df.iloc[row_idx, df.columns.get_loc('公司资金占比')] = 公司占比
        df.iloc[row_idx, df.columns.get_loc('行为性质')] = 行为性质
    
    def _process_expense_transaction(self, row: pd.Series, 处理结果: Dict[str, Any], df: pd.DataFrame, row_idx: int) -> None:
        """处理支出交易"""
        # 资金支出
        个人占比, 公司占比, 行为性质 = self.tracker.处理资金流出(
            处理结果['实际金额'], 
            处理结果['资金属性'], 
            处理结果['完整时间戳']
        )
        
        # 记录结果
        df.iloc[row_idx, df.columns.get_loc('个人资金占比')] = 个人占比
        df.iloc[row_idx, df.columns.get_loc('公司资金占比')] = 公司占比
        df.iloc[row_idx, df.columns.get_loc('行为性质')] = 行为性质
    
    def _process_no_transaction(self, row: pd.Series, 处理结果: Dict[str, Any], df: pd.DataFrame, row_idx: int) -> None:
        """处理无交易情况"""
        df.iloc[row_idx, df.columns.get_loc('个人资金占比')] = 0
        df.iloc[row_idx, df.columns.get_loc('公司资金占比')] = 0
        df.iloc[row_idx, df.columns.get_loc('行为性质')] = '无交易'
    
    def _update_result_columns(self, df: pd.DataFrame, row_idx: int) -> None:
        """更新结果列"""
        # 记录当前余额、占比和行为性质
        df.iloc[row_idx, df.columns.get_loc('累计挪用')] = self.tracker.累计挪用金额
        df.iloc[row_idx, df.columns.get_loc('累计垫付')] = self.tracker.累计垫付金额
        df.iloc[row_idx, df.columns.get_loc('累计由资金池回归公司余额本金')] = self.tracker.累计由资金池回归公司余额本金
        df.iloc[row_idx, df.columns.get_loc('累计由资金池回归个人余额本金')] = self.tracker.累计由资金池回归个人余额本金
        df.iloc[row_idx, df.columns.get_loc('累计非法所得')] = self.tracker.累计非法所得
        df.iloc[row_idx, df.columns.get_loc('总计个人应分配利润')] = self.tracker.总计个人应分配利润
        df.iloc[row_idx, df.columns.get_loc('总计公司应分配利润')] = self.tracker.总计公司应分配利润
        df.iloc[row_idx, df.columns.get_loc('个人余额')] = self.tracker.个人余额
        df.iloc[row_idx, df.columns.get_loc('公司余额')] = self.tracker.公司余额
        df.iloc[row_idx, df.columns.get_loc('总余额')] = self.tracker.个人余额 + self.tracker.公司余额
        
        # 计算资金缺口：累计挪用 - 累计个人归还公司本金
        资金缺口 = (self.tracker.累计挪用金额 - 
                   self.tracker.累计由资金池回归个人余额本金)
        df.iloc[row_idx, df.columns.get_loc('资金缺口')] = 资金缺口
    
    def _generate_analysis_results(self, df: pd.DataFrame) -> None:
        """生成分析结果"""
        # 显示分析结果
        audit_logger.info("=" * 60)
        audit_logger.info("FIFO资金追踪结果")
        audit_logger.info("=" * 60)
        
        # 最终余额状况
        audit_logger.info(f"最终余额状况:")
        audit_logger.info(f"个人余额: {self.tracker.个人余额:,.2f}")
        audit_logger.info(f"公司余额: {self.tracker.公司余额:,.2f}")
        audit_logger.info(f"总余额: {self.tracker.个人余额 + self.tracker.公司余额:,.2f}")
        
        if self.tracker.个人余额 + self.tracker.公司余额 > 0:
            个人占比, 公司占比 = self.tracker.获取当前资金占比()
            audit_logger.info(f"个人资金占比: {个人占比:.2%}")
            audit_logger.info(f"公司资金占比: {公司占比:.2%}")
        
        # 挪用和垫付情况
        audit_logger.info(f"挪用和垫付情况:")
        audit_logger.info(f"累计挪用金额（个人使用公司资金，包括投资挪用）: {self.tracker.累计挪用金额:,.2f}")
        audit_logger.info(f"累计由资金池回归公司余额本金: {self.tracker.累计由资金池回归公司余额本金:,.2f}")
        audit_logger.info(f"累计由资金池回归个人余额本金: {self.tracker.累计由资金池回归个人余额本金:,.2f}")
        audit_logger.info(f"累计垫付金额（公司使用个人资金）: {self.tracker.累计垫付金额:,.2f}")
        audit_logger.info(f"总计个人应分配利润: {self.tracker.总计个人应分配利润:,.2f}")
        audit_logger.info(f"总计公司应分配利润: {self.tracker.总计公司应分配利润:,.2f}")
        
        # 计算资金缺口：累计挪用 - 累计归还给公司的本金
        资金缺口 = (self.tracker.累计挪用金额 - 
                   self.tracker.累计由资金池回归公司余额本金)
        
        audit_logger.info(f"汇总:")
        audit_logger.info(f"个人累计挪用: {self.tracker.累计挪用金额:,.2f}")
        audit_logger.info(f"公司累计垫付: {self.tracker.累计垫付金额:,.2f}")
        audit_logger.info(f"资金缺口: {资金缺口:,.2f} （挪用{self.tracker.累计挪用金额:,.2f} - 公司归还{self.tracker.累计由资金池回归公司余额本金:,.2f}）")
        
        # 投资产品明细表 - 已移至单独Excel文件
        # self._show_investment_products()
        
        # 可疑交易分析
        self._analyze_suspicious_transactions(df)
    
    def _show_investment_products(self) -> None:
        """显示投资产品明细表"""
        audit_logger.info("=" * 60)
        audit_logger.info("投资产品明细表")
        audit_logger.info("=" * 60)
        
        if len(self.tracker.投资产品资金池) > 0:
            for product_id, product_info in self.tracker.投资产品资金池.items():
                audit_logger.info(f"产品: {product_id}")
                audit_logger.info(f"  总金额: {product_info['总金额']:,.2f}")
                
                # 安全计算占比，避免除零错误
                if abs(product_info['总金额']) > 0.01:
                    个人占比 = product_info['个人金额'] / product_info['总金额']
                    公司占比 = product_info['公司金额'] / product_info['总金额']
                    audit_logger.info(f"  个人金额: {product_info['个人金额']:,.2f} ({个人占比:.2%})")
                    audit_logger.info(f"  公司金额: {product_info['公司金额']:,.2f} ({公司占比:.2%})")
                else:
                    audit_logger.info(f"  个人金额: {product_info['个人金额']:,.2f} (占比无法计算)")
                    audit_logger.info(f"  公司金额: {product_info['公司金额']:,.2f} (占比无法计算)")
                
                audit_logger.info(f"  累计申购: {product_info['累计申购']:,.2f}")
                audit_logger.info(f"  累计赎回: {product_info['累计赎回']:,.2f}")
                
                # 检查累计申购赎回相等但总金额不为0的情况
                if abs(product_info['累计申购'] - product_info['累计赎回']) < 0.01 and abs(product_info['总金额']) > 0.01:
                    audit_logger.warning(f"  注意: 累计申购赎回相等但总金额不为0，可能存在计算错误")
        else:
            audit_logger.info("无投资产品记录")
    
    def _analyze_suspicious_transactions(self, df: pd.DataFrame) -> None:
        """分析可疑交易"""
        audit_logger.info("=" * 60)
        audit_logger.info("可疑交易分析")
        audit_logger.info("=" * 60)
        
        # 查找挪用交易
        try:
            mask = df['行为性质'].astype(str).str.contains('挪用', na=False)
            挪用交易 = df[mask]
            if len(挪用交易) > 0:
                audit_logger.info(f"发现 {len(挪用交易)} 笔挪用交易")
        except Exception as e:
            audit_logger.error(f"显示挪用交易时出错: {e}")
        
        # 查找垫付交易
        try:
            mask = df['行为性质'].astype(str).str.contains('垫付', na=False)
            垫付交易 = df[mask]
            if len(垫付交易) > 0:
                audit_logger.info(f"发现 {len(垫付交易)} 笔垫付交易")
        except Exception as e:
            audit_logger.error(f"显示垫付交易时出错: {e}")
        
        # 查找大额异常交易
        try:
            大额交易 = df[df['总余额'] > Config.LARGE_AMOUNT_THRESHOLD]
            if len(大额交易) > 0:
                audit_logger.info(f"发现 {len(大额交易)} 笔大额交易")
        except Exception as e:
            audit_logger.error(f"显示大额交易时出错: {e}")
    
    def _generate_visualizations(self, df: pd.DataFrame) -> None:
        """生成可视化图表"""
        try:
            # 创建基础图表
            self.visualization.创建基础图表(df, "基础分析图表.png")
            
            # 创建投资产品图表
            if self.tracker.投资产品资金池:
                self.visualization.创建投资产品图表(self.tracker.投资产品资金池, "投资产品分析图表.png")
            
            # 创建异常交易图表
            异常分析 = self.flow_analyzer.分析异常交易(df)
            if any(异常分析.values()):
                self.visualization.创建异常交易图表(异常分析, "异常交易分析图表.png")
                
        except Exception as e:
            audit_logger.error(f"生成可视化图表失败: {e}")
    
    def _generate_report(self, df: pd.DataFrame) -> None:
        """生成分析报告"""
        try:
            分析结果 = {
                '基本信息': self.data_processor.生成数据摘要(df),
                '余额状况': self.tracker.获取状态摘要(),
                '挪用垫付情况': {
                    '累计挪用金额': self.tracker.累计挪用金额,
                    '累计垫付金额': self.tracker.累计垫付金额,
                    '累计非法所得': self.tracker.累计非法所得,
                    '总计个人分配利润': self.tracker.总计个人分配利润,
                    '总计公司分配利润': self.tracker.总计公司分配利润
                },
                '投资产品情况': self.investment_manager.获取投资产品统计(),
                '异常交易情况': self.flow_analyzer.分析异常交易(df)
            }
            
            报告内容 = self.visualization.生成分析报告(分析结果, "分析报告.txt")
            audit_logger.info("分析报告生成完成")
            
        except Exception as e:
            audit_logger.error(f"生成分析报告失败: {e}")
    
    def _save_error_report(self, validation_result: Dict, error_report_file: str) -> None:
        """保存验证错误报告"""
        try:
            with open(error_report_file, 'w', encoding='utf-8') as f:
                f.write("=" * 80 + "\n")
                f.write("流水完整性验证错误报告\n")
                f.write("=" * 80 + "\n")
                f.write(f"生成时间: {pd.Timestamp.now()}\n")
                f.write(f"总行数: {validation_result['total_rows']}\n")
                f.write(f"发现错误: {validation_result['errors_count']}个\n")
                f.write(f"成功修复: {validation_result['optimizations_count']}个\n")
                f.write(f"优化状态: {'失败' if validation_result['optimization_failed'] else '成功'}\n\n")
                
                f.write("错误详情:\n")
                f.write("-" * 80 + "\n")
                for i, error in enumerate(validation_result.get('errors', []), 1):
                    f.write(f"{i}. 第{error['row']}行: {error['message']}\n")
                    f.write(f"   时间: {error['timestamp']}\n\n")
                
                f.write("\n解决建议:\n")
                f.write("-" * 80 + "\n")
                f.write("1. 检查银行流水数据是否完整，确认没有遗漏交易记录\n")
                f.write("2. 验证余额计算是否正确，检查是否存在手工调整\n")
                f.write("3. 确认交易金额和时间戳的准确性\n")
                f.write("4. 检查是否存在同一时间的多笔交易顺序问题\n")
                f.write("5. 如有疑问，请联系数据提供方核实原始数据\n")
                
        except Exception as e:
            audit_logger.error(f"保存错误报告失败: {e}")
    



def main():
    """主函数"""
    # 创建分析器
    分析器 = FIFO资金追踪分析器()
    
    # 分析Excel文件
    df = 分析器.分析财务数据(Config.DEFAULT_INPUT_FILE)
    
    if df is not None:
        audit_logger.info("分析完成！")
    else:
        audit_logger.error("分析失败！")


if __name__ == "__main__":
    main() 