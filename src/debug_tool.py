import pandas as pd
import numpy as np
from datetime import datetime
from typing import Dict, List, Any, Optional
import warnings
warnings.filterwarnings('ignore')

# 导入新的模块化结构
from models.fifo_tracker import FIFO资金追踪器
from utils.data_processor import DataProcessor
from models.flow_analyzer import FlowAnalyzer
from models.behavior_analyzer import BehaviorAnalyzer
from models.investment_manager import InvestmentProductManager
from config import Config
from utils.logger import audit_logger

class DebugTracker:
    """调试追踪器"""
    
    def __init__(self):
        """初始化调试追踪器"""
        self.data = None
        self.total_rows = 0
        self.current_row = 0
        self.tracker = FIFO资金追踪器()
        self.data_processor = DataProcessor()
        self.error_records = []
        self.debug_history = []
        self.module_call_trace = []
    
    def load_data(self, file_path: str) -> bool:
        """加载数据"""
        try:
            self.data = self.data_processor.预处理财务数据(file_path)
            if self.data is not None:
                self.total_rows = len(self.data)
                print(f"✓ 数据加载成功，共 {self.total_rows} 行")
                return True
            else:
                print("✗ 数据加载失败")
                return False
        except Exception as e:
            print(f"✗ 数据加载出错: {e}")
            return False
    
    def reset(self):
        """重置追踪器状态"""
        self.current_row = 0
        self.tracker = FIFO资金追踪器()
        self.error_records = []
        self.debug_history = []
        self.module_call_trace = []
        
        # 重置后自动设置初始余额
        if self.data is not None:
            self._ensure_initial_balance()
            print("✓ 追踪器状态已重置")
        else:
            print("✓ 追踪器状态已重置")
    
    def _record_step(self, method: str, params: Dict[str, Any], result: str, row: int):
        """记录处理步骤"""
        step_info = {
            'step': len(self.debug_history) + 1,
            'method': method,
            'params': params,
            'result': result,
            'row': row,
            'timestamp': datetime.now()
        }
        self.debug_history.append(step_info)
    
    def _record_module_call(self, module_method: str, input_data: str, result: str):
        """记录模块调用"""
        call_info = {
            'module_method': module_method,
            'input_data': input_data,
            'result': result,
            'timestamp': datetime.now(),
            'row': self.current_row
        }
        self.module_call_trace.append(call_info)
        
        # 保持追踪记录不超过100条
        if len(self.module_call_trace) > 100:
            self.module_call_trace = self.module_call_trace[-100:]
    
    def _ensure_initial_balance(self):
        """确保初始余额已设置（通过模块化接口）"""
        if self.data is None:
            return
        
        # 通过DataProcessor模块计算初始余额
        初始余额 = self.data_processor.计算初始余额(self.data)
        
        if 初始余额 > 0:
            # 通过FIFO追踪器模块设置初始余额
            self.tracker.初始化余额(初始余额, '公司')
            print(f"✓ 初始余额设置完成: {初始余额:,.2f} (设为公司余额)")
            
            # 记录模块调用
            self._record_module_call("FIFO资金追踪器.初始化余额", 
                                    f"初始余额={初始余额:,.2f}, 类型=公司", 
                                    f"成功初始化")
        else:
            print("ℹ️ 无需设置初始余额")
    
    def _get_tracker_state(self):
        """获取追踪器当前状态"""
        return self.tracker.获取状态摘要()
    
    def _validate_balance_silent(self, row_idx, expected_balance):
        """验证余额一致性（发现不匹配时输出详细信息并停止）"""
        # 计算总余额（只计算银行卡余额，不包括投资产品）
        total_balance = self.tracker.个人余额 + self.tracker.公司余额
        
        total_balance = Config.format_number(total_balance)
        expected_balance = Config.format_number(expected_balance)
        
        if abs(total_balance - expected_balance) > Config.BALANCE_TOLERANCE:
            # 检测是否可能是同时间交易顺序问题
            self._check_same_time_transactions(row_idx)
            
            # 输出详细的余额不匹配信息
            print(f"\n💥💥💥 第{row_idx}行余额不匹配，立即停止处理! 💥💥💥")
            print("=" * 60)
            print(f"📊 余额不匹配详情:")
            print(f"   Excel原始余额: {expected_balance:,.2f}")
            print(f"   计算得出余额: {total_balance:,.2f}")
            print(f"   差异金额: {total_balance - expected_balance:,.2f}")
            print(f"   📊 计算余额构成:")
            print(f"     个人余额: {self.tracker.个人余额:,.2f}")
            print(f"     公司余额: {self.tracker.公司余额:,.2f}")
            
            # 显示投资产品总数
            if self.tracker.投资产品资金池:
                print(f"     投资产品总数: {len(self.tracker.投资产品资金池)} 个")
            
            # 显示FIFO队列状态
            print(f"   🔍 FIFO队列状态: {len(self.tracker.资金流入队列)} 项")
            if self.tracker.资金流入队列:
                print(f"   🔍 FIFO队列内容:")
                for i, (金额, 类型, 时间) in enumerate(self.tracker.资金流入队列):
                    print(f"     [{i+1}] 金额={金额:,.2f}, 类型={类型}, 时间={时间}")
            
            # 输出当前行的详细信息
            self._show_current_row_detail(row_idx)
            
            # 记录错误信息
            error_info = {
                'row': row_idx,
                'expected': expected_balance,
                'actual': total_balance,
                'difference': Config.format_number(total_balance - expected_balance),
                'tracker_state': self._get_tracker_state(),
                'module_calls': self.module_call_trace[-5:]
            }
            self.error_records.append(error_info)
            
            # 记录到审计日志
            audit_logger.error(f"第{row_idx}行余额不匹配，停止处理: 期望{expected_balance:,.2f}, 实际{total_balance:,.2f}")
            
            return False
        return True
    
    def _show_current_row_detail(self, row_num):
        """显示当前行的详细信息（用于余额不匹配时）"""
        if self.data is None or row_num < 1 or row_num > self.total_rows:
            return
            
        row_data = self.data.iloc[row_num-1]
        print(f"\n📋 第 {row_num} 行详细数据信息:")
        print("=" * 50)
        print(f"完整时间戳: {row_data['完整时间戳']}")
        print(f"交易收入金额: {row_data['交易收入金额']}")
        print(f"交易支出金额: {row_data['交易支出金额']}")
        print(f"余额: {row_data['余额']}")
        print(f"资金属性: {row_data['资金属性']}")
        print(f"资金流向类型: {row_data['资金流向类型']}")
        print(f"行为性质: {row_data['行为性质']}")
        print(f"个人资金占比: {row_data['个人资金占比']:.2%}")
        print(f"公司资金占比: {row_data['公司资金占比']:.2%}")
        
        # 检查当前行是否涉及投资产品，如果是则只显示相关投资产品信息
        资金属性 = str(row_data['资金属性']) if row_data['资金属性'] is not None and str(row_data['资金属性']) != 'nan' else ''
        from config import Config
        if Config.is_investment_product(资金属性):
            交易收入金额 = float(row_data['交易收入金额']) if not pd.isna(row_data['交易收入金额']) else 0.0
            交易支出金额 = float(row_data['交易支出金额']) if not pd.isna(row_data['交易支出金额']) else 0.0
            if 交易收入金额 > 0:
                print(f"\n💰 该行为投资产品赎回交易，相关产品信息:")
                self._show_investment_product_info(资金属性, "")
            elif 交易支出金额 > 0:
                print(f"\n💰 该行为投资产品购买交易，相关产品信息:")
                self._show_investment_product_info(资金属性, "")
        
        # 显示该行相关的模块调用
        related_calls = [call for call in self.module_call_trace if call['row'] == row_num]
        if related_calls:
            print(f"\n📞 该行相关的模块调用:")
            for call in related_calls[-3:]:  # 只显示最近3次调用
                print(f"  {call['timestamp'].strftime('%H:%M:%S')} - {call['module_method']}")
                print(f"    输入: {call['input_data']}")
                print(f"    结果: {call['result']}")
        
        print("=" * 60)
    
    def _check_same_time_transactions(self, row_idx):
        """检测是否存在同时间交易顺序问题"""
        if self.data is None:
            return
        
        # 获取当前行的时间戳
        current_time = self.data.iloc[row_idx-1]['完整时间戳']
        
        # 查找所有相同时间戳的交易
        same_time_rows = []
        for i in range(row_idx):
            if self.data.iloc[i]['完整时间戳'] == current_time:
                same_time_rows.append(i + 1)  # 转换为1基索引
        
        if len(same_time_rows) > 1:
            print(f"\n⚠️  检测到可能的同时间交易顺序问题:")
            print(f"   时间戳: {current_time}")
            print(f"   涉及行数: {same_time_rows}")
            print(f"   建议: 使用DataProcessor.优化同时间交易顺序()方法重新处理数据")
        else:
            print(f"\n   未检测到同时间交易顺序问题")
    
    def _show_investment_product_info(self, 产品属性: str, 交易类型: str = ""):
        """显示特定投资产品的信息"""
        if not self.tracker.投资产品资金池:
            return
        
        # 查找相关的投资产品（使用完整产品信息匹配）
        related_products = []
        for product_id, info in self.tracker.投资产品资金池.items():
            # 直接用完整的产品属性信息匹配，例如"理财-SL100613100620"
            if 产品属性 == product_id:
                related_products.append((product_id, info))
        
        if related_products:
            print(f"📊 {交易类型}涉及的投资产品:")
            for product_id, info in related_products:
                print(f"  {product_id}: {info['总金额']:,.2f} (个人占比:{info['最新个人占比']:.2%}, 公司占比:{info['最新公司占比']:.2%})")
    
    def _validate_balance(self, row_idx, expected_balance):
        """验证余额一致性"""
        # 计算总余额（只计算银行卡余额，不包括投资产品）
        total_balance = self.tracker.个人余额 + self.tracker.公司余额
        
        total_balance = Config.format_number(total_balance)
        expected_balance = Config.format_number(expected_balance)
        
        if abs(total_balance - expected_balance) > Config.BALANCE_TOLERANCE:
            # 打印详细的余额对比信息
            print(f"\n💥 第{row_idx}行余额不匹配详情:")
            print(f"   Excel原始余额: {expected_balance:,.2f}")
            print(f"   计算得出余额: {total_balance:,.2f}")
            print(f"   差异金额: {total_balance - expected_balance:,.2f}")
            print(f"   📊 计算余额构成:")
            print(f"     个人余额: {self.tracker.个人余额:,.2f}")
            print(f"     公司余额: {self.tracker.公司余额:,.2f}")
            
            # 不再自动显示所有投资产品详情
            # 只显示投资产品总数
            if self.tracker.投资产品资金池:
                print(f"     投资产品总数: {len(self.tracker.投资产品资金池)} 个")
            
            # 显示FIFO队列状态
            print(f"   🔍 FIFO队列状态: {len(self.tracker.资金流入队列)} 项")
            if self.tracker.资金流入队列:
                print(f"   🔍 FIFO队列内容:")
                for i, (金额, 类型, 时间) in enumerate(self.tracker.资金流入队列):
                    print(f"     [{i+1}] 金额={金额:,.2f}, 类型={类型}, 时间={时间}")
            
            # 显示最近的模块调用追踪
            print(f"   📞 最近的模块调用:")
            for call in self.module_call_trace[-3:]:
                print(f"     {call['module_method']}: {call['result']}")
            
            print()
            
            error_info = {
                'row': row_idx,
                'expected': expected_balance,
                'actual': total_balance,
                'difference': Config.format_number(total_balance - expected_balance),
                'tracker_state': self._get_tracker_state(),
                'module_calls': self.module_call_trace[-5:]  # 保存最近5次模块调用
            }
            self.error_records.append(error_info)
            
            # 记录到审计日志
            audit_logger.warning(f"第{row_idx}行余额不匹配: 期望{expected_balance:,.2f}, 实际{total_balance:,.2f}")
            
            return False
        return True
    
    def process_to_row(self, target_row):
        """处理数据到指定行数"""
        if self.data is None:
            print("✗ 请先加载数据")
            return False
        
        if target_row < 1 or target_row > self.total_rows:
            print(f"✗ 行数超出范围 (1-{self.total_rows})")
            return False
        
        print(f"\n开始处理数据到第 {target_row} 行...")
        
        # 在开始处理第一行之前，确保初始余额已设置
        if self.current_row == 0 and not self.tracker.已初始化:
            self._ensure_initial_balance()
        
        # 从当前位置继续处理
        start_row = self.current_row
        
        for i in range(start_row, target_row):
            try:
                # 使用模块化方法处理单行数据
                self._process_single_row_modular(i)
                
                # 验证余额（发现不匹配时立即停止）
                expected_balance = self.data.iloc[i]['余额']
                if not self._validate_balance_silent(i + 1, expected_balance):
                    print(f"\n⛔ 由于第{i + 1}行余额不匹配，处理已停止")
                    self.current_row = i + 1  # 更新当前行位置
                    return False
                
            except Exception as e:
                error_info = {
                    'row': i + 1,
                    'error': str(e),
                    'tracker_state': self._get_tracker_state(),
                    'module_calls': self.module_call_trace[-5:]
                }
                self.error_records.append(error_info)
                print(f"✗ 第 {i + 1} 行处理出错: {e}")
                
                # 记录到审计日志
                audit_logger.error(f"第{i + 1}行处理失败: {str(e)}")
                
                import traceback
                traceback.print_exc()
                return False
        
        self.current_row = target_row
        print(f"✓ 成功处理到第 {target_row} 行")
        
        # 只在最后输出详细的状态信息
        if target_row > 0:
            expected_balance = self.data.iloc[target_row-1]['余额']
            if not self._validate_balance(target_row, expected_balance):
                print(f"⚠️ 第 {target_row} 行余额验证失败")
        
        # 检查目标行是否涉及投资产品交易，如果是则显示相关信息
        if target_row > 0 and self.data is not None:
            target_row_data = self.data.iloc[target_row-1]
            交易收入金额 = float(target_row_data['交易收入金额']) if not pd.isna(target_row_data['交易收入金额']) else 0.0
            交易支出金额 = float(target_row_data['交易支出金额']) if not pd.isna(target_row_data['交易支出金额']) else 0.0
            资金属性 = str(target_row_data['资金属性']) if target_row_data['资金属性'] is not None and str(target_row_data['资金属性']) != 'nan' else ''
            
            # 检查是否为投资产品交易
            if Config.is_investment_product(资金属性):
                if 交易收入金额 > 0:
                    print(f"📊 第{target_row}行投资产品赎回:")
                    self._show_investment_product_info(资金属性, "投资产品赎回")
                elif 交易支出金额 > 0:
                    print(f"📊 第{target_row}行投资产品购买:")
                    self._show_investment_product_info(资金属性, "投资产品购买")
        
        # 自动显示状态
        self.show_status()
        return True
    
    def _process_single_row_modular(self, idx):
        """处理单行数据 - 使用模块化方法"""
        if self.data is None:
            return
            
        row = self.data.iloc[idx]
        
        # 使用数据处理器处理单行交易
        处理结果 = self.data_processor.处理单行交易(row, idx)
        
        self._record_module_call("DataProcessor.处理单行交易", 
                                f"第{idx+1}行", 
                                f"方向={处理结果['方向']}, 金额={处理结果['实际金额']}")
        
        # 记录开始处理
        self._record_step("处理单行数据", {
            '行号': idx + 1,
            '处理结果': 处理结果
        }, f"开始处理第{idx+1}行", idx + 1)
        
        # 根据交易方向使用相应的模块处理
        if 处理结果['方向'] == '收入':
            self._process_income_transaction_modular(row, 处理结果, idx)
        elif 处理结果['方向'] == '支出':
            self._process_expense_transaction_modular(row, 处理结果, idx)
        else:
            self._process_no_transaction_modular(row, 处理结果, idx)
        
        # 更新结果列
        self._update_result_columns_modular(idx)
    
    def _process_income_transaction_modular(self, row, 处理结果, idx):
        """处理收入交易 - 使用模块化方法"""
        if 处理结果['is_investment']:
                # 投资产品赎回
            个人占比, 公司占比, 行为性质 = self.tracker.处理投资产品赎回(
                处理结果['实际金额'], 
                处理结果['资金属性'], 
                处理结果['完整时间戳']
            )
            
            self._record_module_call("FIFO资金追踪器.处理投资产品赎回", 
                                    f"金额={处理结果['实际金额']}, 属性={处理结果['资金属性']}", 
                                    f"个人占比={个人占比:.2%}, 公司占比={公司占比:.2%}")
            else:
                # 普通收入
            个人占比, 公司占比, 行为性质 = self.tracker.处理资金流入(
                处理结果['实际金额'], 
                处理结果['资金属性'], 
                处理结果['完整时间戳']
            )
            
            self._record_module_call("FIFO资金追踪器.处理资金流入", 
                                    f"金额={处理结果['实际金额']}, 属性={处理结果['资金属性']}", 
                                    f"个人占比={个人占比:.2%}, 公司占比={公司占比:.2%}")
        
        # 记录结果
        self.data.at[idx, '个人资金占比'] = 个人占比
        self.data.at[idx, '公司资金占比'] = 公司占比
        self.data.at[idx, '行为性质'] = 行为性质
        self.data.at[idx, '资金流向类型'] = 处理结果['资金流向类型']
        
        self._record_step("处理收入交易", {
            '方向': '收入',
            '是否投资': 处理结果['is_investment'],
            '个人占比': 个人占比,
            '公司占比': 公司占比
        }, f"行为性质={行为性质}", idx + 1)
    
    def _process_expense_transaction_modular(self, row, 处理结果, idx):
        """处理支出交易 - 使用模块化方法"""
        # 资金支出
        个人占比, 公司占比, 行为性质 = self.tracker.处理资金流出(
            处理结果['实际金额'], 
            处理结果['资金属性'], 
            处理结果['完整时间戳']
        )
        
        self._record_module_call("FIFO资金追踪器.处理资金流出", 
                                f"金额={处理结果['实际金额']}, 属性={处理结果['资金属性']}", 
                                f"个人占比={个人占比:.2%}, 公司占比={公司占比:.2%}")
        
        # 记录结果
        self.data.at[idx, '个人资金占比'] = 个人占比
        self.data.at[idx, '公司资金占比'] = 公司占比
        self.data.at[idx, '行为性质'] = 行为性质
        self.data.at[idx, '资金流向类型'] = 处理结果['资金流向类型']
        
        self._record_step("处理支出交易", {
            '方向': '支出',
            '个人占比': 个人占比,
            '公司占比': 公司占比
        }, f"行为性质={行为性质}", idx + 1)
    
    def _process_no_transaction_modular(self, row, 处理结果, idx):
        """处理无交易情况 - 使用模块化方法"""
        self.data.at[idx, '个人资金占比'] = 0
        self.data.at[idx, '公司资金占比'] = 0
        self.data.at[idx, '行为性质'] = '无交易'
        self.data.at[idx, '资金流向类型'] = 处理结果['资金流向类型']
        
        self._record_step("处理无交易", {
            '方向': '无'
        }, "无交易", idx + 1)
    
    def _update_result_columns_modular(self, idx):
        """更新结果列 - 使用模块化方法"""
        # 记录当前余额、占比和行为性质
        self.data.at[idx, '累计挪用'] = self.tracker.累计挪用金额
        self.data.at[idx, '累计垫付'] = self.tracker.累计垫付金额
        self.data.at[idx, '累计非法所得'] = self.tracker.累计非法所得
        self.data.at[idx, '总计个人分配利润'] = self.tracker.总计个人分配利润
        self.data.at[idx, '总计公司分配利润'] = self.tracker.总计公司分配利润
        self.data.at[idx, '个人余额'] = self.tracker.个人余额
        self.data.at[idx, '公司余额'] = self.tracker.公司余额
        self.data.at[idx, '总余额'] = self.tracker.个人余额 + self.tracker.公司余额
        
        # 计算资金缺口
        资金缺口 = self.tracker.累计挪用金额 - self.tracker.累计由资金池回归公司余额本金 - self.tracker.累计垫付金额
        self.data.at[idx, '资金缺口'] = 资金缺口
    
    def show_status(self):
        """显示调试状态"""
        print("\n" + "="*50)
        print("🔍 调试状态")
        print("="*50)
        print(f"当前行数: {self.current_row:>14}")
        print(f"总行数: {self.total_rows:>16}")
        print(f"处理进度: {(self.current_row/self.total_rows)*100:.1f}%")
        print(f"错误数量: {len(self.error_records):>14}")
        
        print("\n💰 资金状态:")
        state = self._get_tracker_state()
        print(f"个人余额: {state['个人余额']:>15,.2f}")
        print(f"公司余额: {state['公司余额']:>15,.2f}")
        print(f"总余额: {state['总余额']:>15,.2f}")
        print(f"投资产品数量: {state['投资产品数量']:>10}")
        print(f"累计挪用金额: {state['累计挪用金额']:>15,.2f}")
        print(f"累计垫付金额: {state['累计垫付金额']:>15,.2f}")
        print(f"累计非法所得: {state['累计非法所得']:>15,.2f}")
        print(f"已初始化: {state['已初始化']}")
        
        # 不再自动显示所有投资产品详情
        # 只显示投资产品总数统计
        if self.tracker.投资产品资金池:
            print(f"\n📊 投资产品统计:")
            print(f"  总数量: {len(self.tracker.投资产品资金池)} 个")
            total_investment = sum(info['总金额'] for info in self.tracker.投资产品资金池.values())
            print(f"  总金额: {total_investment:,.2f}")
        
        # 显示FIFO队列详细状态
        print(f"\n🔍 FIFO资金流入队列状态:")
        print(f"  队列长度: {len(self.tracker.资金流入队列)}")
        
        if self.tracker.资金流入队列:
            # 计算FIFO队列总金额
            fifo_total = sum(item[0] for item in self.tracker.资金流入队列)
            balance_total = self.tracker.个人余额 + self.tracker.公司余额
            
            print(f"  队列总金额: {fifo_total:,.2f}")
            print(f"  余额总金额: {balance_total:,.2f}")
            print(f"  差异: {fifo_total - balance_total:,.2f}")
            
            if len(self.tracker.资金流入队列) <= 10:
            print(f"  队列内容:")
            for i, (金额, 类型, 时间) in enumerate(self.tracker.资金流入队列):
                print(f"    [{i+1}] 金额={金额:,.2f}, 类型={类型}, 时间={时间}")
        else:
                print(f"  队列前5项:")
                for i, (金额, 类型, 时间) in enumerate(self.tracker.资金流入队列[:5]):
                    print(f"    [{i+1}] 金额={金额:,.2f}, 类型={类型}, 时间={时间}")
                print(f"  队列后5项:")
                for i, (金额, 类型, 时间) in enumerate(self.tracker.资金流入队列[-5:]):
                    actual_idx = len(self.tracker.资金流入队列) - 5 + i + 1
                    print(f"    [{actual_idx}] 金额={金额:,.2f}, 类型={类型}, 时间={时间}")
        
        # 显示处理历史中的错误
        if self.error_records:
            print(f"\n⚠️ 发现 {len(self.error_records)} 个错误:")
            for error in self.error_records[-3:]:  # 显示最近3个错误
                print(f"  第{error['row']}行: {error.get('error', '余额不匹配')}")
        
        # 显示当前行的数据
        if self.current_row > 0 and self.data is not None:
            print(f"\n📋 当前行数据 (第{self.current_row}行):")
            current_row_data = self.data.iloc[self.current_row-1]
            print(f"  完整时间戳: {current_row_data['完整时间戳']}")
            print(f"  交易收入金额: {current_row_data['交易收入金额']}")
            print(f"  交易支出金额: {current_row_data['交易支出金额']}")
            print(f"  余额: {current_row_data['余额']}")
            print(f"  资金属性: {current_row_data['资金属性']}")
            print(f"  资金流向类型: {current_row_data['资金流向类型']}")
            print(f"  行为性质: {current_row_data['行为性质']}")
        
        # 显示最近的模块调用
        if self.module_call_trace:
            print(f"\n📞 最近的模块调用 (最近5次):")
            for call in self.module_call_trace[-5:]:
                print(f"  {call['timestamp'].strftime('%H:%M:%S')} - {call['module_method']}")
                print(f"    输入: {call['input_data']}")
                print(f"    结果: {call['result']}")
    
    def show_history(self, last_n=5):
        """显示最近的处理历史"""
        if not self.debug_history:
            print("暂无处理历史")
            return
        
        print(f"\n📜 最近 {min(last_n, len(self.debug_history))} 步处理历史:")
        print("-" * 80)
        
        for step in self.debug_history[-last_n:]:
            print(f"步骤 {step['step']} (第{step['row']}行) - {step['timestamp'].strftime('%H:%M:%S')}")
            print(f"  方法: {step['method']}")
            if step['params']:
                print(f"  参数: {step['params']}")
            print(f"  结果: {step['result']}")
            print()
    
    def show_errors(self):
        """显示所有错误记录"""
        if not self.error_records:
            print("✓ 暂无错误记录")
            return
        
        print(f"\n❌ 发现 {len(self.error_records)} 个错误:")
        print("-" * 80)
        
        for i, error in enumerate(self.error_records, 1):
            print(f"错误 {i} - 第{error['row']}行:")
            if 'expected' in error:
                print(f"  期望余额: {error['expected']:,.2f}")
                print(f"  实际余额: {error['actual']:,.2f}")
                print(f"  差额: {error['difference']:,.2f}")
            else:
                print(f"  错误信息: {error['error']}")
            
            # 显示相关的模块调用
            if 'module_calls' in error:
                print(f"  相关模块调用:")
                for call in error['module_calls']:
                    print(f"    {call['module_method']}: {call['result']}")
            
            print()
    
    def show_module_trace(self, last_n=10):
        """显示模块调用跟踪"""
        if not self.module_call_trace:
            print("暂无模块调用记录")
            return
        
        print(f"\n📞 最近 {min(last_n, len(self.module_call_trace))} 次模块调用:")
        print("-" * 80)
        
        for call in self.module_call_trace[-last_n:]:
            print(f"{call['timestamp'].strftime('%H:%M:%S')} - 第{call['row']}行")
            print(f"  模块方法: {call['module_method']}")
            print(f"  输入数据: {call['input_data']}")
            print(f"  调用结果: {call['result']}")
            print()
    
    def show_detail(self, row_num):
        """显示指定行的详细信息"""
        if self.data is None:
            print("✗ 请先加载数据")
            return
            
        if row_num < 1 or row_num > self.total_rows:
            print(f"✗ 行数超出范围 (1-{self.total_rows})")
            return
        
        if row_num > self.current_row:
            print(f"✗ 该行尚未处理，当前处理到第 {self.current_row} 行")
            return
        
        row_data = self.data.iloc[row_num-1]
        print(f"\n📋 第 {row_num} 行详细信息:")
        print("=" * 50)
        print(f"完整时间戳: {row_data['完整时间戳']}")
        print(f"交易收入金额: {row_data['交易收入金额']}")
        print(f"交易支出金额: {row_data['交易支出金额']}")
        print(f"余额: {row_data['余额']}")
        print(f"资金属性: {row_data['资金属性']}")
        print(f"资金流向类型: {row_data['资金流向类型']}")
        print(f"行为性质: {row_data['行为性质']}")
        print(f"个人资金占比: {row_data['个人资金占比']:.2%}")
        print(f"公司资金占比: {row_data['公司资金占比']:.2%}")
        print(f"个人余额: {row_data['个人余额']:,.2f}")
        print(f"公司余额: {row_data['公司余额']:,.2f}")
        print(f"总余额: {row_data['总余额']:,.2f}")
        print(f"累计挪用: {row_data['累计挪用']:,.2f}")
        print(f"累计垫付: {row_data['累计垫付']:,.2f}")
        print(f"累计非法所得: {row_data['累计非法所得']:,.2f}")
        
        # 显示该行相关的模块调用
        related_calls = [call for call in self.module_call_trace if call['row'] == row_num]
        if related_calls:
            print(f"\n📞 该行相关的模块调用:")
            for call in related_calls:
                print(f"  {call['timestamp'].strftime('%H:%M:%S')} - {call['module_method']}")
                print(f"    输入: {call['input_data']}")
                print(f"    结果: {call['result']}")
        
        # 显示该行的处理历史
        related_steps = [step for step in self.debug_history if step['row'] == row_num]
        if related_steps:
            print(f"\n📜 该行处理历史:")
            for step in related_steps:
                print(f"  步骤{step['step']} - {step['method']}: {step['result']}")

def main():
    """主函数 - 交互式debug工具"""
    print("="*60)
    print("FIFO资金追踪 Debug工具 (模块化版本)")
    print("使用完整的模块化架构，支持详细的调用追踪和溯源")
    print("="*60)
    
    debug_tracker = DebugTracker()
    
    # 加载数据
    file_path = "流水.xlsx"
    if not debug_tracker.load_data(file_path):
        return
    
    print("\n💡 可用命令:")
    print("  run <行数>         - 重置并运行到指定行数")
    print("  next [行数]        - 继续处理下一行或指定行数")
    print("  status             - 显示当前状态")
    print("  history [n]        - 显示最近n步历史(默认5)")
    print("  errors             - 显示所有错误")
    print("  detail <行数>      - 显示指定行的详细信息")
    print("  trace [n]          - 显示最近n次模块调用追踪(默认10)")
    print("  reset              - 重置追踪器")
    print("  quit               - 退出")
    
    while True:
        try:
            user_input = input(f"\n[{debug_tracker.current_row}/{debug_tracker.total_rows}] > ").strip()
            
            if not user_input:
                continue
                
            parts = user_input.split()
            command = parts[0].lower()
            
            if command == 'quit':
                print("👋 退出调试工具")
                break
                
            elif command == 'run':
                if len(parts) < 2:
                    print("用法: run <行数>")
                    continue
                try:
                    target_row = int(parts[1])
                    print(f"🔄 自动重置追踪器...")
                    debug_tracker.reset()
                    print(f"🚀 重新处理到第 {target_row} 行...")
                    debug_tracker.process_to_row(target_row)
                except ValueError:
                    print("请输入有效的行数")
                    
            elif command == 'next':
                if len(parts) > 1:
                    try:
                        step_count = int(parts[1])
                        target_row = debug_tracker.current_row + step_count
                    except ValueError:
                        print("请输入有效的行数")
                        continue
                else:
                    target_row = debug_tracker.current_row + 1
                
                if target_row <= debug_tracker.total_rows:
                    debug_tracker.process_to_row(target_row)
                else:
                    print("已到达数据末尾")
                    
            elif command == 'status':
                debug_tracker.show_status()
                
            elif command == 'history':
                n = 5
                if len(parts) > 1:
                    try:
                        n = int(parts[1])
                    except ValueError:
                        print("请输入有效的数字")
                        continue
                debug_tracker.show_history(n)
                
            elif command == 'errors':
                debug_tracker.show_errors()
                
            elif command == 'detail':
                if len(parts) < 2:
                    print("用法: detail <行数>")
                    continue
                try:
                    row_num = int(parts[1])
                    debug_tracker.show_detail(row_num)
                except ValueError:
                    print("请输入有效的行数")
                    
            elif command == 'trace':
                n = 10
                if len(parts) > 1:
                    try:
                        n = int(parts[1])
                    except ValueError:
                        print("请输入有效的数字")
                        continue
                debug_tracker.show_module_trace(n)
                    
            elif command == 'reset':
                debug_tracker.reset()
                
            else:
                print("❌ 未知命令。可用命令: run, next, status, history, errors, detail, trace, reset, quit")
                
        except KeyboardInterrupt:
            print("\n\n👋 退出...")
            break
        except Exception as e:
            print(f"❌ 发生错误: {e}")
            audit_logger.error(f"Debug工具出错: {str(e)}")
            import traceback
            traceback.print_exc()

if __name__ == "__main__":
    main() 