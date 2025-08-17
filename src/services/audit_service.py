"""
审计分析服务
重构main.py的功能，支持算法切换，最大化复用现有逻辑
"""

import time
import sys
from typing import Optional, Dict, Any
import pandas as pd

from core.interfaces.tracker_interface import ITracker
from core.factories.tracker_factory import TrackerFactory
from utils.data_processor import DataProcessor
from utils.flow_integrity_validator import FlowIntegrityValidator
from utils.logger import audit_logger
from config import Config


class AuditService:
    """审计分析服务 - 支持多种算法"""
    
    def __init__(self, algorithm: str = "FIFO"):
        """
        初始化审计服务
        
        Args:
            algorithm: 算法类型 ("FIFO" 或 "BALANCE_METHOD")
        """
        self.algorithm = algorithm
        
        # 创建追踪器（支持算法切换）
        self.tracker = TrackerFactory.create_tracker(algorithm)
        
        # 复用现有模块（100%复用）
        self.data_processor = DataProcessor()
        self.flow_validator = FlowIntegrityValidator()
        
        audit_logger.info(f"审计服务初始化完成，使用算法: {algorithm}")
    
    def analyze_financial_data(self, file_path: str, output_file: Optional[str] = None) -> Optional[pd.DataFrame]:
        """
        分析财务数据 - 完全复用main.py的逻辑，只替换追踪器
        
        Args:
            file_path: Excel文件路径
            output_file: 输出文件名
            
        Returns:
            分析结果数据框，失败返回None
        """
        start_time = time.time()
        
        try:
            audit_logger.info("=" * 60)
            audit_logger.info(f"公款挪用与职务侵占审计分析 - {self.algorithm}算法")
            audit_logger.info("=" * 60)
            
            # 1. 数据预处理（完全复用）
            print("📊 开始数据预处理...")
            df = self.data_processor.预处理财务数据(file_path)
            if df is None:
                print("❌ 数据预处理失败")
                return None
            print(f"✅ 数据预处理完成，共加载 {len(df):,} 条记录")
            
            # 2. 流水完整性验证（完全复用）
            print("🔍 开始流水完整性验证...")
            validation_result = self.flow_validator.validate_flow_integrity(df)
            if not validation_result['is_valid']:
                print(f"⚠️  流水完整性验证发现 {validation_result['errors_count']} 个问题")
                audit_logger.warning(f"流水完整性验证发现{validation_result['errors_count']}个问题")
                
                if validation_result['optimization_failed']:
                    print("❌ 流水优化失败，无法自动修复数据完整性问题")
                    audit_logger.error("❌ 流水优化失败，无法自动修复数据完整性问题")
                    
                    # 保存错误报告
                    error_report_file = f"流水验证错误报告_{self.algorithm}.txt"
                    self._save_error_report(validation_result, error_report_file)
                    print(f"📄 错误详情已保存至: {error_report_file}")
                    audit_logger.info(f"📄 错误详情已保存至: {error_report_file}")
                    return None
                
                if validation_result['optimizations_count'] > 0:
                    print(f"🔧 已通过重排序修复 {validation_result['optimizations_count']} 个问题")
                    audit_logger.info(f"已通过重排序修复{validation_result['optimizations_count']}个问题")
                    df = validation_result['result_dataframe']
                    print("✅ 使用修复后的数据继续处理（源文件保持不变）")
                    audit_logger.info("✅ 使用修复后的数据继续处理（源文件保持不变）")
            else:
                print("✅ 流水完整性验证通过")
                sys.stdout.flush()
                audit_logger.info("✅ 流水完整性验证通过")
            
            # 3. 数据验证（完全复用）
            print("🔎 开始数据验证...")
            validation_result = self.data_processor.验证数据完整性(df)
            if not validation_result['is_valid']:
                print("⚠️  数据验证发现问题，但继续处理")
                audit_logger.warning("数据验证发现问题，但继续处理")
                for error in validation_result['errors'][:5]:
                    audit_logger.warning(error)
            else:
                print("✅ 数据验证通过")
            
            # 4. 计算初始余额（完全复用）
            print("💰 计算初始余额...")
            初始余额 = self.data_processor.计算初始余额(df)
            if 初始余额 > 0:
                print(f"📊 初始余额: {初始余额:,.2f} 元")
                self.tracker.初始化余额(初始余额, '公司')
            else:
                print("📊 无初始余额")
            
            # 5. 逐笔处理交易（使用新的追踪器）
            print(f"🚀 开始 {self.algorithm} 资金追踪分析...")
            audit_logger.info(f"开始{self.algorithm}资金追踪分析...")
            self._process_transactions(df)
            
            # 6. 生成分析结果
            print("📈 生成分析结果...")
            audit_logger.info(f"{self.algorithm}资金追踪完成！")
            self._generate_analysis_results(df)
            
            # 7. 保存结果
            if output_file is None:
                output_file = f"{self.algorithm}_资金追踪结果.xlsx"
            
            print(f"💾 保存分析结果到: {output_file}")
            self.data_processor.保存结果(df, output_file)
            
            # 8. 生成投资产品交易记录Excel
            investment_file = f"投资产品交易记录_{self.algorithm}.xlsx"
            print(f"📋 生成投资产品交易记录: {investment_file}")
            self.tracker.生成投资产品交易记录Excel(investment_file)
            
            processing_time = time.time() - start_time
            audit_logger.log_performance(f"{self.algorithm}完整分析", processing_time, len(df))
            
            audit_logger.info("流水数据处理完成")
            return df
            
        except Exception as e:
            audit_logger.log_error(e, f"{self.algorithm}分析财务数据失败")
            import traceback
            traceback.print_exc()
            return None
    
    def _process_transactions(self, df: pd.DataFrame) -> None:
        """
        处理所有交易 - 复用main.py逻辑，使用新追踪器
        """
        total_count = len(df)
        print(f"📋 总共需要处理 {total_count:,} 条交易记录")
        sys.stdout.flush()
        
        for i, (idx, row) in enumerate(df.iterrows()):
            # 显示详细的处理进度（减少频率，避免日志过密）
            if i % (Config.PROGRESS_INTERVAL * 2) == 0:  # 每2000条显示一次
                progress_percent = (i / total_count) * 100
                print(f"⏳ 处理进度: {i:,}/{total_count:,} ({progress_percent:.1f}%)")
                sys.stdout.flush()  # 强制刷新输出
                audit_logger.info(f"处理进度: {i}/{len(df)}")
            
            # 处理单行交易（完全复用DataProcessor）
            处理结果 = self.data_processor.处理单行交易(row, i)
            
            # 根据交易方向处理（使用新追踪器）
            if 处理结果['方向'] == '收入':
                self._process_income_transaction(row, 处理结果, df, i)
            elif 处理结果['方向'] == '支出':
                self._process_expense_transaction(row, 处理结果, df, i)
            else:
                self._process_no_transaction(row, 处理结果, df, i)
            
            # 更新结果列
            self._update_result_columns(df, i)
        
        print(f"✅ 所有 {total_count:,} 条交易记录处理完成")
        sys.stdout.flush()
    
    def _process_income_transaction(self, row: pd.Series, 处理结果: Dict[str, Any], df: pd.DataFrame, row_idx: int) -> None:
        """处理收入交易 - 使用新追踪器"""
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
        """处理支出交易 - 使用新追踪器"""
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
        """更新结果列 - 使用新追踪器"""
        df.iloc[row_idx, df.columns.get_loc('累计挪用')] = self.tracker.累计挪用金额
        df.iloc[row_idx, df.columns.get_loc('累计垫付')] = self.tracker.累计垫付金额
        df.iloc[row_idx, df.columns.get_loc('累计已归还公司本金')] = self.tracker.累计已归还公司本金
        df.iloc[row_idx, df.columns.get_loc('累计非法所得')] = 0  # 差额法不使用此字段
        df.iloc[row_idx, df.columns.get_loc('总计个人分配利润')] = self.tracker.总计个人分配利润
        df.iloc[row_idx, df.columns.get_loc('总计公司分配利润')] = self.tracker.总计公司分配利润
        df.iloc[row_idx, df.columns.get_loc('个人余额')] = self.tracker.个人余额
        df.iloc[row_idx, df.columns.get_loc('公司余额')] = self.tracker.公司余额
        df.iloc[row_idx, df.columns.get_loc('总余额')] = self.tracker.个人余额 + self.tracker.公司余额
        
        # 计算应还金额
        个人应还金额 = max(0, self.tracker.累计挪用金额 - self.tracker.累计已归还公司本金)
        df.iloc[row_idx, df.columns.get_loc('个人应还')] = 个人应还金额
        df.iloc[row_idx, df.columns.get_loc('公司应还')] = self.tracker.累计垫付金额
    
    def _generate_analysis_results(self, df: pd.DataFrame) -> None:
        """生成分析结果 - 复用main.py逻辑"""
        audit_logger.info("=" * 60)
        audit_logger.info(f"{self.algorithm}资金追踪结果")
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
        audit_logger.info(f"累计挪用金额: {self.tracker.累计挪用金额:,.2f}")
        audit_logger.info(f"累计已归还公司本金: {self.tracker.累计已归还公司本金:,.2f}")
        audit_logger.info(f"累计垫付金额: {self.tracker.累计垫付金额:,.2f}")
        audit_logger.info(f"总计个人分配利润: {self.tracker.总计个人分配利润:,.2f}")
        audit_logger.info(f"总计公司分配利润: {self.tracker.总计公司分配利润:,.2f}")
        
        个人应还金额 = max(0, self.tracker.累计挪用金额 - self.tracker.累计已归还公司本金)
        净挪用 = 个人应还金额 - self.tracker.累计垫付金额
        
        audit_logger.info(f"汇总:")
        audit_logger.info(f"个人应还公司总金额: {个人应还金额:,.2f}")
        audit_logger.info(f"公司应还个人总金额: {self.tracker.累计垫付金额:,.2f}")
        audit_logger.info(f"净挪用金额: {净挪用:,.2f}")
        
        # 算法特定信息
        if self.algorithm == "BALANCE_METHOD":
            audit_logger.info(f"差额计算法特有指标:")
            audit_logger.info(f"净挪用金额（扣除归还）: {self.tracker.累计挪用金额 - self.tracker.累计已归还公司本金:,.2f}")
    
    def _save_error_report(self, validation_result: Dict, error_report_file: str) -> None:
        """保存验证错误报告 - 复用main.py逻辑"""
        try:
            with open(error_report_file, 'w', encoding='utf-8') as f:
                f.write("=" * 80 + "\n")
                f.write(f"流水完整性验证错误报告 - {self.algorithm}算法\n")
                f.write("=" * 80 + "\n")
                f.write(f"生成时间: {pd.Timestamp.now()}\n")
                f.write(f"算法类型: {self.algorithm}\n")
                f.write(f"总行数: {validation_result['total_rows']}\n")
                f.write(f"发现错误: {validation_result['errors_count']}个\n")
                f.write(f"成功修复: {validation_result['optimizations_count']}个\n")
                f.write(f"优化状态: {'失败' if validation_result['optimization_failed'] else '成功'}\n\n")
                
                f.write("错误详情:\n")
                f.write("-" * 80 + "\n")
                for i, error in enumerate(validation_result.get('errors', []), 1):
                    f.write(f"{i}. 第{error['row']}行: {error['message']}\n")
                    f.write(f"   时间: {error['timestamp']}\n\n")
                
        except Exception as e:
            audit_logger.error(f"保存错误报告失败: {e}")
    
    def get_algorithm_info(self) -> Dict[str, str]:
        """获取当前算法信息"""
        return {
            "algorithm": self.algorithm,
            "description": TrackerFactory.get_algorithm_description(self.algorithm)
        }
    
    def switch_algorithm(self, new_algorithm: str) -> bool:
        """
        切换算法
        
        Args:
            new_algorithm: 新算法类型
            
        Returns:
            是否切换成功
        """
        try:
            self.tracker = TrackerFactory.create_tracker(new_algorithm)
            self.algorithm = new_algorithm
            audit_logger.info(f"算法已切换至: {new_algorithm}")
            return True
        except ValueError as e:
            audit_logger.error(f"算法切换失败: {e}")
            return False