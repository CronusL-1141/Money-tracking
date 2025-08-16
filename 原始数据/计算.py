import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from collections import Counter, deque
import warnings
warnings.filterwarnings('ignore')

# 设置中文字体
plt.rcParams['font.sans-serif'] = ['SimHei', 'Microsoft YaHei']
plt.rcParams['axes.unicode_minus'] = False

def check_columns(df):
    """检查数据框的列名"""
    print("=" * 60)
    print("列名检查")
    print("=" * 60)
    print("所有列名:")
    for i, col in enumerate(df.columns):
        print(f"{i+1}. '{col}' (类型: {df[col].dtype})")
    
    print("\n前5行数据:")
    print(df.head())
    print("\n数据信息:")
    print(df.info())

class FIFO资金追踪器:
    def __init__(self):
        # 个人资金池余额
        self.个人余额 = 0.0
        # 公司资金池余额
        self.公司余额 = 0.0
        # 投资产品资金池字典，存储{产品编号: {'个人金额': 0, '公司金额': 0, '总金额': 0, '累计申购': 0, '累计赎回': 0}}
        self.投资产品资金池 = {}
        # 资金流入FIFO队列，存储(金额, 类型, 时间)
        self.资金流入队列 = deque()
        # 是否已初始化
        self.已初始化 = False
        # 累计挪用金额（个人使用公司资金，包括投资挪用）
        self.累计挪用金额 = 0.0
        # 累计垫付金额（公司使用个人资金）
        self.累计垫付金额 = 0.0
        # 累计非法所得（投资收益中的非法部分）
        self.累计非法所得 = 0.0
        # 总计个人分配利润
        self.总计个人分配利润 = 0.0
        # 总计公司分配利润
        self.总计公司分配利润 = 0.0
    
    def 格式化数值(self, 数值):
        """格式化数值，避免科学计数法显示，处理极小值"""
        if abs(数值) < 1e-8:
            return 0.0
        return round(数值, 2)
        
    def 解析投资产品编号(self, 资金属性):
        """解析投资产品的编号（理财、投资、保险、关联银行卡等）"""
        if isinstance(资金属性, str) and '-' in 资金属性:
            # 检查是否为投资类产品
            前缀 = 资金属性.split('-')[0]
            if 前缀 in ['理财', '投资', '保险', '关联银行卡']:
                return 资金属性  # 返回完整的产品编号
        return None
    
    def 判断资金属性类型(self, 资金属性):
        """判断资金属性是个人还是公司"""
        资金属性_str = str(资金属性).strip()
        if '个人' in 资金属性_str:
            return '个人'
        elif '公司' in 资金属性_str:
            return '公司'
        else:
            return '其他'
    
    def 初始化余额(self, 初始余额, 余额类型='公司'):
        """初始化余额，默认设为公司余额"""
        if not self.已初始化 and 初始余额 > 0:
            if 余额类型 == '公司':
                self.公司余额 = 初始余额
                self.资金流入队列.append((初始余额, '公司', None))
            else:
                self.个人余额 = 初始余额
                self.资金流入队列.append((初始余额, '个人', None))
            self.已初始化 = True
            print(f"初始化余额: {初始余额:,.2f} (设为{余额类型}余额)")
    
    def 处理资金流入(self, 金额, 资金属性, 交易日期):
        """处理资金流入，按FIFO原则分配"""
        if 金额 <= 0:
            return 0, 0, ""  # 返回个人占比, 公司占比, 行为性质
        
        # 判断资金属性类型
        资金属性_str = str(资金属性).strip()
        
        if '个人应收' in 资金属性_str or '个人应付' in 资金属性_str:
            # 个人资金
            self.个人余额 += 金额
            self.资金流入队列.append((金额, '个人', 交易日期))
            return 1.0, 0.0, f"个人资金流入：{金额:,.2f}"  # 100%个人
        elif '公司应收' in 资金属性_str or '公司应付' in 资金属性_str:
            # 公司资金
            self.公司余额 += 金额
            self.资金流入队列.append((金额, '公司', 交易日期))
            return 0.0, 1.0, f"公司资金流入：{金额:,.2f}"  # 100%公司
        else:
            # 其他情况，按当前余额比例分配
            total_balance = self.个人余额 + self.公司余额
            if total_balance == 0:
                # 如果总余额为0，按默认规则处理（这里可以根据需要调整）
                print(f"警告: 资金池为空，收到{金额:,.2f}，按默认规则处理")
                # 默认按1:1分配，或者根据业务规则调整
                个人金额 = 金额 / 2
                公司金额 = 金额 / 2
                self.个人余额 += 个人金额
                self.公司余额 += 公司金额
                self.资金流入队列.append((个人金额, '个人', 交易日期))
                self.资金流入队列.append((公司金额, '公司', 交易日期))
                return 0.5, 0.5, f"混合资金流入：个人{个人金额:,.2f}，公司{公司金额:,.2f}"
            else:
                个人占比 = self.个人余额 / total_balance
                公司占比 = self.公司余额 / total_balance
                个人金额 = 金额 * 个人占比
                公司金额 = 金额 * 公司占比
                self.个人余额 += 个人金额
                self.公司余额 += 公司金额
                self.资金流入队列.append((个人金额, '个人', 交易日期))
                self.资金流入队列.append((公司金额, '公司', 交易日期))
                return 个人占比, 公司占比, f"混合资金流入：个人{个人金额:,.2f}，公司{公司金额:,.2f}"
    
    def 处理资金流出(self, 金额, 资金属性, 交易日期):
        """处理资金流出，按FIFO原则分配"""
        if 金额 <= 0:
            return 0, 0, ""
        
        # 检查是否为投资产品申购
        投资产品编号 = self.解析投资产品编号(资金属性)
        if 投资产品编号:
            # 投资产品申购，按FIFO原则处理，投资是个人行为
            个人占比, 公司占比, 行为性质 = self.处理资金流出_投资(金额, 资金属性, 交易日期)
            
            # 更新投资产品资金池（支持再次申购，更新资金占比）
            个人金额 = 金额 * 个人占比
            公司金额 = 金额 * 公司占比
            
            if 投资产品编号 not in self.投资产品资金池:
                # 新建投资产品资金池
                self.投资产品资金池[投资产品编号] = {
                    '个人金额': 0, 
                    '公司金额': 0, 
                    '总金额': 0,
                    '累计申购': 0,
                    '累计赎回': 0,
                    '最新个人占比': 0,
                    '最新公司占比': 0
                }
            
            # 检查当前资金池状态
            当前总金额 = self.投资产品资金池[投资产品编号]['总金额']
            
            if 当前总金额 < 0:
                # 资金池为负数，说明之前有收益，再次申购时重置资金池
                print(f"投资产品{投资产品编号}资金池为负数({当前总金额:,.2f})，重置为新申购")
                self.投资产品资金池[投资产品编号]['个人金额'] = 个人金额
                self.投资产品资金池[投资产品编号]['公司金额'] = 公司金额
                self.投资产品资金池[投资产品编号]['总金额'] = 金额
                # 更新占比记录（与新入资的占比一致）
                self.投资产品资金池[投资产品编号]['最新个人占比'] = 个人占比
                self.投资产品资金池[投资产品编号]['最新公司占比'] = 公司占比
            else:
                # 正常累加新申购金额
                self.投资产品资金池[投资产品编号]['个人金额'] += 个人金额
                self.投资产品资金池[投资产品编号]['公司金额'] += 公司金额
                self.投资产品资金池[投资产品编号]['总金额'] += 金额
                # 更新占比记录（按新的总金额计算）
                新总金额 = self.投资产品资金池[投资产品编号]['总金额']
                if 新总金额 > 0:
                    self.投资产品资金池[投资产品编号]['最新个人占比'] = self.投资产品资金池[投资产品编号]['个人金额'] / 新总金额
                    self.投资产品资金池[投资产品编号]['最新公司占比'] = self.投资产品资金池[投资产品编号]['公司金额'] / 新总金额
            
            self.投资产品资金池[投资产品编号]['累计申购'] += 金额
            
            前缀 = 投资产品编号.split('-')[0]
            return 个人占比, 公司占比, f"{前缀}申购-{投资产品编号}：{行为性质}"
        else:
            return self.处理资金流出_普通(金额, 资金属性, 交易日期)
    
    def 处理资金流出_普通(self, 金额, 资金属性, 交易日期):
        """处理普通资金流出，并分析行为性质"""
        if 金额 <= 0:
            return 0, 0, ""
        
        # 检查是否有足够的资金
        total_available = self.个人余额 + self.公司余额
        if total_available <= 0:
            print(f"警告: 资金池已空，无法支出{金额:,.2f}")
            return 0, 0, f"资金池已空，无法支出{金额:,.2f}"
        
        # 记录原始交易金额
        原始金额 = 金额
        实际扣除金额 = min(金额, total_available)
        资金缺口 = 金额 - 实际扣除金额
        
        if total_available < 金额:
            print(f"警告: 资金不足! 需要{金额:,.2f}, 可用{total_available:,.2f}，实际扣除{实际扣除金额:,.2f}，资金缺口{资金缺口:,.2f}")
        
        # 修复：如果FIFO队列为空但余额不为空，重新构建队列
        if len(self.资金流入队列) == 0 and total_available > 0:
            print(f"警告: FIFO队列为空但余额不为空，重新构建队列")
            # 按当前余额重新构建队列
            if self.个人余额 > 0:
                self.资金流入队列.append((self.个人余额, '个人', 交易日期))
            if self.公司余额 > 0:
                self.资金流入队列.append((self.公司余额, '公司', 交易日期))
        
        # 按FIFO原则从资金流入队列中扣除（使用实际扣除金额）
        剩余金额 = 实际扣除金额
        个人扣除 = 0
        公司扣除 = 0
        
        while 剩余金额 > 0 and self.资金流入队列:
            流入金额, 流入类型, 流入时间 = self.资金流入队列.popleft()
            
            if 流入金额 <= 剩余金额:
                # 完全扣除这笔流入
                if 流入类型 == '个人':
                    个人扣除 += 流入金额
                    self.个人余额 = max(0, self.个人余额 - 流入金额)
                else:
                    公司扣除 += 流入金额
                    self.公司余额 = max(0, self.公司余额 - 流入金额)
                剩余金额 -= 流入金额
            else:
                # 部分扣除这笔流入
                if 流入类型 == '个人':
                    个人扣除 += 剩余金额
                    self.个人余额 = max(0, self.个人余额 - 剩余金额)
                    # 将剩余部分重新放回队列
                    self.资金流入队列.appendleft((流入金额 - 剩余金额, '个人', 流入时间))
                else:
                    公司扣除 += 剩余金额
                    self.公司余额 = max(0, self.公司余额 - 剩余金额)
                    # 将剩余部分重新放回队列
                    self.资金流入队列.appendleft((流入金额 - 剩余金额, '公司', 流入时间))
                剩余金额 = 0
        
        # 计算占比（基于原始金额）
        if 原始金额 > 0:
            个人占比 = 个人扣除 / 原始金额
            公司占比 = 公司扣除 / 原始金额
        else:
            个人占比 = 0
            公司占比 = 0
        
        # 分析行为性质
        资金属性类型 = self.判断资金属性类型(资金属性)
        基础行为性质 = self.分析行为性质(资金属性类型, 个人扣除, 公司扣除, 原始金额)
        
        # 添加资金不足的说明
        if 资金缺口 > 0:
            行为性质 = f"{基础行为性质}；资金缺口：{资金缺口:,.2f}"
        else:
            行为性质 = 基础行为性质
        
        return 个人占比, 公司占比, 行为性质
    
    def 处理资金流出_投资(self, 金额, 资金属性, 交易日期):
        """处理投资资金流出，投资是个人行为，使用公司资金就是挪用"""
        if 金额 <= 0:
            return 0, 0, ""
        
        # 检查是否有足够的资金
        total_available = self.个人余额 + self.公司余额
        if total_available <= 0:
            print(f"警告: 资金池已空，无法支出{金额:,.2f}")
            return 0, 0, "资金池已空"
        
        if total_available < 金额:
            print(f"警告: 资金不足! 需要{金额:,.2f}, 可用{total_available:,.2f}，只扣除现有余额")
            金额 = total_available  # 只扣除现有余额
        
        # 按FIFO原则从资金流入队列中扣除
        剩余金额 = 金额
        个人扣除 = 0
        公司扣除 = 0
        
        while 剩余金额 > 0 and self.资金流入队列:
            流入金额, 流入类型, 流入时间 = self.资金流入队列.popleft()
            
            if 流入金额 <= 剩余金额:
                # 完全扣除这笔流入
                if 流入类型 == '个人':
                    个人扣除 += 流入金额
                    self.个人余额 = max(0, self.个人余额 - 流入金额)
                else:
                    公司扣除 += 流入金额
                    self.公司余额 = max(0, self.公司余额 - 流入金额)
                剩余金额 -= 流入金额
            else:
                # 部分扣除这笔流入
                if 流入类型 == '个人':
                    个人扣除 += 剩余金额
                    self.个人余额 = max(0, self.个人余额 - 剩余金额)
                    # 将剩余部分重新放回队列
                    self.资金流入队列.appendleft((流入金额 - 剩余金额, '个人', 流入时间))
                else:
                    公司扣除 += 剩余金额
                    self.公司余额 = max(0, self.公司余额 - 剩余金额)
                    # 将剩余部分重新放回队列
                    self.资金流入队列.appendleft((流入金额 - 剩余金额, '公司', 流入时间))
                剩余金额 = 0
        
        # 计算占比
        total_deducted = 个人扣除 + 公司扣除
        if total_deducted > 0:
            个人占比 = 个人扣除 / total_deducted
            公司占比 = 公司扣除 / total_deducted
        else:
            个人占比 = 0
            公司占比 = 0
        
        # 投资是个人行为，使用公司资金就是挪用
        if 公司扣除 > 0:
            self.累计挪用金额 += 公司扣除
            self.累计挪用金额 = self.格式化数值(self.累计挪用金额)
            
        # 构造行为性质描述
        行为描述 = []
        if 公司扣除 > 0:
            行为描述.append(f"投资挪用：{公司扣除:,.2f}")
        if 个人扣除 > 0:
            行为描述.append(f"个人投资：{个人扣除:,.2f}")
        
        行为性质 = "；".join(行为描述) if 行为描述 else "无投资"
        
        return 个人占比, 公司占比, 行为性质
    
    def 分析行为性质(self, 资金属性类型, 个人扣除, 公司扣除, 总金额):
        """分析交易的行为性质，判断是否构成挪用或垫付"""
        if 总金额 <= 0:
            return "无交易"
        
        行为描述 = []
        
        if 资金属性类型 == '个人':
            # 个人应付/支出
            if 公司扣除 > 0:
                # 个人支出使用了公司资金 - 构成挪用
                self.累计挪用金额 += 公司扣除
                self.累计挪用金额 = self.格式化数值(self.累计挪用金额)
                行为描述.append(f"挪用：{公司扣除:,.2f}")
            if 个人扣除 > 0:
                # 个人支出使用了个人资金 - 正常
                行为描述.append(f"个人支付：{个人扣除:,.2f}")
        elif 资金属性类型 == '公司':
            # 公司应付/支出
            if 个人扣除 > 0:
                # 公司支出使用了个人资金 - 构成垫付
                self.累计垫付金额 += 个人扣除
                self.累计垫付金额 = self.格式化数值(self.累计垫付金额)
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
    
    def 处理投资产品赎回(self, 金额, 资金属性, 交易日期):
        """处理投资产品赎回，按总体比例返还资金，计算收益和非法所得"""
        if 金额 <= 0:
            return 0, 0, ""
        
        投资产品编号 = self.解析投资产品编号(资金属性)
        if not 投资产品编号:
            return 0, 0, ""
        
        # 查找对应的投资产品记录
        if 投资产品编号 not in self.投资产品资金池:
            # 没有找到对应的投资产品记录，说明没有申购记录，
            # 按照用户要求，这种收入应该算做个人应收来计算
            self.个人余额 += 金额
            self.资金流入队列.append((金额, '个人', 交易日期))
            前缀 = 投资产品编号.split('-')[0]
            return 1.0, 0.0, f"{前缀}收入-{投资产品编号}：个人应收{金额:,.2f}（无申购记录）"
        
        产品信息 = self.投资产品资金池[投资产品编号]
        总金额 = 产品信息['总金额']
        个人金额 = 产品信息['个人金额']
        公司金额 = 产品信息['公司金额']
        累计申购 = 产品信息['累计申购']
        累计赎回 = 产品信息['累计赎回']
        最新个人占比 = 产品信息['最新个人占比']
        最新公司占比 = 产品信息['最新公司占比']
        
        # 检查是否有有效的占比记录
        if 最新个人占比 == 0 and 最新公司占比 == 0:
            return 0, 0, f"错误：投资产品{投资产品编号}从未有过有效资金池，无法分配收益"
        
        # 赎回时统一使用最新记录的占比，不管资金池是正数、0还是负数
        个人返还 = 金额 * 最新个人占比
        公司返还 = 金额 * 最新公司占比
        
        # 计算收益情况
        if 总金额 > 0:
            # 正常赎回逻辑
            赎回比例 = 金额 / 总金额 if 总金额 > 0 else 0
            对应申购成本 = 总金额 * 赎回比例
            收益 = 金额 - 对应申购成本
            
            # 更新投资产品资金池
            self.投资产品资金池[投资产品编号]['个人金额'] -= 个人金额 * 赎回比例
            self.投资产品资金池[投资产品编号]['公司金额'] -= 公司金额 * 赎回比例
            self.投资产品资金池[投资产品编号]['总金额'] -= 对应申购成本
        else:
            # 资金池为0或负数，纯收益分配
            收益 = 金额
            
            # 更新资金池（继续减少，保持负数状态）
            self.投资产品资金池[投资产品编号]['个人金额'] -= 个人返还
            self.投资产品资金池[投资产品编号]['公司金额'] -= 公司返还
            self.投资产品资金池[投资产品编号]['总金额'] -= 金额
        
        self.投资产品资金池[投资产品编号]['累计赎回'] += 金额
        
        # 如果有收益，按比例计算非法所得
        if 收益 > 0:
            个人收益 = 收益 * 最新个人占比
            公司收益 = 收益 * 最新公司占比
            非法所得 = 个人收益 + 公司收益
            if abs(非法所得) < 1e-8:
                非法所得 = 0.0
            非法所得 = self.格式化数值(非法所得)
            self.累计非法所得 += 非法所得
            self.累计非法所得 = self.格式化数值(self.累计非法所得)
            self.总计个人分配利润 += 个人收益
            self.总计个人分配利润 = self.格式化数值(self.总计个人分配利润)
            self.总计公司分配利润 += 公司收益
            self.总计公司分配利润 = self.格式化数值(self.总计公司分配利润)
        else:
            个人收益 = 0
            公司收益 = 0
            非法所得 = 0
        
        # 返还到资金池并加入FIFO队列
        if 个人返还 > 0:
            self.个人余额 += 个人返还
            self.资金流入队列.append((个人返还, '个人', 交易日期))
        
        if 公司返还 > 0:
            self.公司余额 += 公司返还
            self.资金流入队列.append((公司返还, '公司', 交易日期))
        
        前缀 = 投资产品编号.split('-')[0]
        
        # 构造行为性质描述
        if 收益 > 0:
            if 最新个人占比 > 0 and 最新公司占比 > 0:
                行为性质 = f"{前缀}赎回-{投资产品编号}：个人{个人返还:,.2f}，公司{公司返还:,.2f}，收益{收益:,.2f}（个人收益{个人收益:,.2f}，公司收益{公司收益:,.2f}，非法所得{非法所得:,.2f}）"
            elif 最新个人占比 > 0:
                行为性质 = f"{前缀}赎回-{投资产品编号}：个人{个人返还:,.2f}，收益{收益:,.2f}（个人收益{个人收益:,.2f}，非法所得{非法所得:,.2f}）"
            else:
                行为性质 = f"{前缀}赎回-{投资产品编号}：公司{公司返还:,.2f}，收益{收益:,.2f}（公司收益{公司收益:,.2f}，非法所得{非法所得:,.2f}）"
        elif 收益 < 0:
            行为性质 = f"{前缀}赎回-{投资产品编号}：个人{个人返还:,.2f}，公司{公司返还:,.2f}，亏损{abs(收益):,.2f}"
        else:
            行为性质 = f"{前缀}赎回-{投资产品编号}：个人{个人返还:,.2f}，公司{公司返还:,.2f}，无收益"
        
        return 最新个人占比, 最新公司占比, 行为性质
    
    def 获取当前资金占比(self):
        """获取当前个人和公司资金占比"""
        total_balance = self.个人余额 + self.公司余额
        if total_balance == 0:
            return 0, 0
        return self.个人余额 / total_balance, self.公司余额 / total_balance

def preprocess_financial_data(file_path):
    """
    预处理财务数据：读取Excel文件，处理时间，按时间排序（保持相同时间的原始顺序），初始化结果列
    """
    try:
        # 读取Excel文件
        print("正在读取Excel文件...")
        df = pd.read_excel(file_path)
        
        print(f"数据总行数: {len(df)}")
        print(f"数据总列数: {len(df.columns)}")
        
        # 检查列名
        check_columns(df)
        
        # 数据预处理
        print("\n正在预处理数据...")
        
        # 确保交易日期是datetime类型
        df['交易日期'] = pd.to_datetime(df['交易日期'])
        
        # 处理交易时间，将其转换为标准时间格式
        def 解析交易时间(时间值):
            """将交易时间转换为时分秒格式"""
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
        
        # 创建完整时间戳
        df['交易时间_格式化'] = df['交易时间'].apply(解析交易时间)
        df['完整时间戳'] = pd.to_datetime(df['交易日期'].dt.strftime('%Y-%m-%d') + ' ' + df['交易时间_格式化'])
        
        # 添加原始索引列，用于保持相同时间交易的原始顺序
        df['原始索引'] = df.index
        
        # 按完整时间戳排序，相同时间的交易保持原始顺序
        df = df.sort_values(['完整时间戳', '原始索引'], kind='stable').reset_index(drop=True)
        
        # 删除临时的原始索引列
        df = df.drop('原始索引', axis=1)
        
        # 重新整理列顺序，将时间相关字段放在一起
        原列顺序 = df.columns.tolist()
        新列顺序 = []
        
        # 找到交易时间的位置
        交易时间_位置 = 原列顺序.index('交易时间') if '交易时间' in 原列顺序 else -1
        
        # 重新排列列顺序
        for i, col in enumerate(原列顺序):
            if col == '交易时间':
                新列顺序.append(col)
                新列顺序.append('交易时间_格式化')
                新列顺序.append('完整时间戳')
            elif col not in ['交易时间_格式化', '完整时间戳']:
                新列顺序.append(col)
        
        # 重新排列DataFrame列顺序
        df = df[新列顺序]
        
        # 修复pandas DataFrame操作方法调用问题
        print(f"完整时间戳示例:")
        示例数据 = df[['交易日期', '交易时间', '交易时间_格式化', '完整时间戳']].head()
        print(示例数据.to_string())
        
        # 初始化结果列
        df['个人资金占比'] = 0.0
        df['公司资金占比'] = 0.0
        df['资金流向类型'] = ''
        df['行为性质'] = ''
        df['累计挪用'] = 0.0
        df['累计垫付'] = 0.0
        df['累计非法所得'] = 0.0
        df['总计个人分配利润'] = 0.0
        df['总计公司分配利润'] = 0.0
        df['个人余额'] = 0.0
        df['公司余额'] = 0.0
        df['总余额'] = 0.0
        df['公司应还'] = 0.0
        df['个人应还'] = 0.0
        
        return df
        
    except Exception as e:
        print(f"数据预处理失败: {e}")
        import traceback
        traceback.print_exc()
        return None

def analyze_financial_data(file_path):
    """
    分析财务数据，实现FIFO原则的资金追踪
    """
    print("=" * 60)
    print("公款挪用与职务侵占审计分析 - FIFO资金追踪")
    print("=" * 60)
    
    try:
        # 调用预处理函数
        df = preprocess_financial_data(file_path)
        if df is None:
            return None
        
        # 初始化FIFO追踪器
        追踪器 = FIFO资金追踪器()
        
        # 在FIFO账本簿中初始化余额（不插入到交易流水中）
        if len(df) > 0:
            第一笔余额 = df.iloc[0]['余额'] if pd.notna(df.iloc[0]['余额']) else 0.0
            第一笔交易金额 = 0.0
            if pd.notna(df.iloc[0]['交易收入金额']):
                第一笔交易金额 = df.iloc[0]['交易收入金额']
            elif pd.notna(df.iloc[0]['交易支出金额']):
                第一笔交易金额 = -df.iloc[0]['交易支出金额']
            
            # 计算交易前余额（初始余额）
            初始余额 = 第一笔余额 - 第一笔交易金额
            
            if 初始余额 > 0:
                print(f"计算得出初始余额: {初始余额:,.2f}（第一笔余额{第一笔余额:,.2f} - 第一笔交易{第一笔交易金额:,.2f}）")
                print(f"将初始余额作为公司应收在FIFO账本簿中初始化")
                # 在FIFO账本簿中初始化为公司应收，时间戳设为第一笔交易前1秒
                初始时间戳 = df.iloc[0]['完整时间戳'] - pd.Timedelta(seconds=1)
                追踪器.初始化余额(初始余额, '公司')
        
        # 初始化结果列
        df['个人资金占比'] = 0.0
        df['公司资金占比'] = 0.0
        df['资金流向类型'] = ''
        df['行为性质'] = ''
        df['累计挪用'] = 0.0
        df['累计垫付'] = 0.0
        df['累计非法所得'] = 0.0
        df['总计个人分配利润'] = 0.0
        df['总计公司分配利润'] = 0.0
        df['个人余额'] = 0.0
        df['公司余额'] = 0.0
        df['总余额'] = 0.0
        df['公司应还'] = 0.0
        df['个人应还'] = 0.0
        
        print("\n开始FIFO资金追踪分析...")
        
        # 逐笔处理交易
        for i, (idx, row) in enumerate(df.iterrows()):
            if i % 1000 == 0:
                print(f"处理进度: {i}/{len(df)}")
            
            交易收入金额 = float(row['交易收入金额']) if not np.isnan(row['交易收入金额']) else 0.0
            交易支出金额 = float(row['交易支出金额']) if not np.isnan(row['交易支出金额']) else 0.0
            资金属性 = str(row['资金属性']) if row['资金属性'] is not None and str(row['资金属性']) != 'nan' else ''
            完整时间戳 = row['完整时间戳']  # 使用完整时间戳而不是仅仅日期
            
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
            
            # 处理资金流向
            if 方向 == '收入':
                # 检查是否为投资产品赎回
                投资产品编号 = 追踪器.解析投资产品编号(资金属性)
                if 投资产品编号:
                    # 投资产品赎回
                    个人占比, 公司占比, 行为性质 = 追踪器.处理投资产品赎回(实际金额, 资金属性, 完整时间戳)
                    前缀 = 投资产品编号.split('-')[0]
                    df.at[idx, '资金流向类型'] = f'{前缀}赎回-{投资产品编号}'
                else:
                    # 普通收入
                    个人占比, 公司占比, 行为性质 = 追踪器.处理资金流入(实际金额, 资金属性, 完整时间戳)
                    df.at[idx, '资金流向类型'] = '资金流入'
            elif 方向 == '支出':
                # 资金支出
                个人占比, 公司占比, 行为性质 = 追踪器.处理资金流出(实际金额, 资金属性, 完整时间戳)
                df.at[idx, '资金流向类型'] = '资金支出'
            else:
                个人占比, 公司占比, 行为性质 = 0, 0, '无交易'
                df.at[idx, '资金流向类型'] = '无交易'
            
            # 记录当前余额、占比和行为性质
            df.at[idx, '个人资金占比'] = 个人占比
            df.at[idx, '公司资金占比'] = 公司占比
            df.at[idx, '行为性质'] = 行为性质
            df.at[idx, '累计挪用'] = 追踪器.累计挪用金额
            df.at[idx, '累计垫付'] = 追踪器.累计垫付金额
            df.at[idx, '累计非法所得'] = 追踪器.累计非法所得
            df.at[idx, '总计个人分配利润'] = 追踪器.总计个人分配利润
            df.at[idx, '总计公司分配利润'] = 追踪器.总计公司分配利润
            df.at[idx, '个人余额'] = 追踪器.个人余额
            df.at[idx, '公司余额'] = 追踪器.公司余额
            df.at[idx, '总余额'] = 追踪器.个人余额 + 追踪器.公司余额
            
            # 计算应还金额
            总挪用 = 追踪器.累计挪用金额 + 追踪器.累计非法所得
            df.at[idx, '个人应还'] = 总挪用  # 个人应该还给公司的总金额
            df.at[idx, '公司应还'] = 追踪器.累计垫付金额  # 公司应该还给个人的总金额
        
        print("\nFIFO资金追踪完成！")
        
        # 显示分析结果
        print("\n" + "=" * 60)
        print("FIFO资金追踪结果")
        print("=" * 60)
        
        print(f"\n最终余额状况:")
        print(f"个人余额: {追踪器.个人余额:,.2f}")
        print(f"公司余额: {追踪器.公司余额:,.2f}")
        print(f"总余额: {追踪器.个人余额 + 追踪器.公司余额:,.2f}")
        
        if 追踪器.个人余额 + 追踪器.公司余额 > 0:
            个人占比, 公司占比 = 追踪器.获取当前资金占比()
            print(f"个人资金占比: {个人占比:.2%}")
            print(f"公司资金占比: {公司占比:.2%}")
        
        print(f"\n挪用和垫付情况:")
        print(f"累计挪用金额（个人使用公司资金，包括投资挪用）: {追踪器.累计挪用金额:,.2f}")
        print(f"累计非法所得（投资收益中的非法部分）: {追踪器.累计非法所得:,.2f}")
        print(f"累计垫付金额（公司使用个人资金）: {追踪器.累计垫付金额:,.2f}")
        print(f"总计个人分配利润: {追踪器.总计个人分配利润:,.2f}")
        print(f"总计公司分配利润: {追踪器.总计公司分配利润:,.2f}")
        
        总挪用 = 追踪器.累计挪用金额 + 追踪器.累计非法所得
        净挪用 = 总挪用 - 追踪器.累计垫付金额
        
        print(f"\n汇总:")
        print(f"个人应还公司总金额: {总挪用:,.2f}")
        print(f"公司应还个人总金额: {追踪器.累计垫付金额:,.2f}")
        print(f"净挪用金额: {净挪用:,.2f}")
        
        # 创建投资产品明细表
        print("\n" + "=" * 60)
        print("投资产品明细表")
        print("=" * 60)
        
        if len(追踪器.投资产品资金池) > 0:
            for product_id, product_info in 追踪器.投资产品资金池.items():
                print(f"\n产品: {product_id}")
                print(f"  总金额: {product_info['总金额']:,.2f}")
                
                # 安全计算占比，避免除零错误
                if abs(product_info['总金额']) > 0.01:
                    个人占比 = product_info['个人金额'] / product_info['总金额']
                    公司占比 = product_info['公司金额'] / product_info['总金额']
                    print(f"  个人金额: {product_info['个人金额']:,.2f} ({个人占比:.2%})")
                    print(f"  公司金额: {product_info['公司金额']:,.2f} ({公司占比:.2%})")
                else:
                    print(f"  个人金额: {product_info['个人金额']:,.2f} (占比无法计算)")
                    print(f"  公司金额: {product_info['公司金额']:,.2f} (占比无法计算)")
                
                print(f"  累计申购: {product_info['累计申购']:,.2f}")
                print(f"  累计赎回: {product_info['累计赎回']:,.2f}")
                
                # 检查累计申购赎回相等但总金额不为0的情况
                if abs(product_info['累计申购'] - product_info['累计赎回']) < 0.01 and abs(product_info['总金额']) > 0.01:
                    print(f"  ⚠️  注意: 累计申购赎回相等但总金额不为0，可能存在计算错误")
        else:
            print("无投资产品记录")
        
        # 可疑交易分析
        print("\n" + "=" * 60)
        print("可疑交易分析")
        print("=" * 60)
        
        # 查找挪用交易
        try:
            mask = df['行为性质'].astype(str).str.contains('挪用', na=False)
            挪用交易 = df[mask]
            if len(挪用交易) > 0:
                print(f"\n发现 {len(挪用交易)} 笔挪用交易:")
                display_cols = ['交易日期', '交易收入金额', '交易支出金额', '资金属性', '行为性质', '累计挪用']
                挪用交易_display = 挪用交易[display_cols]
                if len(挪用交易_display) > 10:
                    挪用交易_display = 挪用交易_display.iloc[:10]
                print(挪用交易_display.to_string())
        except Exception as e:
            print(f"显示挪用交易时出错: {e}")
        
        # 查找垫付交易
        try:
            mask = df['行为性质'].astype(str).str.contains('垫付', na=False)
            垫付交易 = df[mask]
            if len(垫付交易) > 0:
                print(f"\n发现 {len(垫付交易)} 笔垫付交易:")
                display_cols = ['交易日期', '交易收入金额', '交易支出金额', '资金属性', '行为性质', '累计垫付']
                垫付交易_display = 垫付交易[display_cols]
                if len(垫付交易_display) > 10:
                    垫付交易_display = 垫付交易_display.iloc[:10]
                print(垫付交易_display.to_string())
        except Exception as e:
            print(f"显示垫付交易时出错: {e}")
        
        # 查找大额异常交易
        try:
            大额交易 = df[df['总余额'] > 1000000]  # 超过100万的交易
            if len(大额交易) > 0:
                print(f"\n发现 {len(大额交易)} 笔大额交易:")
                display_cols = ['交易日期', '交易收入金额', '交易支出金额', '资金属性', '资金流向类型', '总余额']
                大额交易_display = 大额交易[display_cols]
                if len(大额交易_display) > 10:
                    大额交易_display = 大额交易_display.iloc[:10]
                print(大额交易_display.to_string())
        except Exception as e:
            print(f"显示大额交易时出错: {e}")
        
        # 保存结果到新的Excel文件
        输出文件名 = "FIFO资金追踪结果.xlsx"
        df.to_excel(输出文件名, index=False)
        print(f"\n分析结果已保存到: {输出文件名}")
        
        return df
        
    except Exception as e:
        print(f"处理数据时出错: {e}")
        import traceback
        traceback.print_exc()
        return None

def create_visualizations(df):
    """
    创建可视化图表
    """
    if df is None:
        return
    
    # 创建图表
    fig, axes = plt.subplots(2, 2, figsize=(15, 12))
    fig.suptitle('FIFO资金追踪分析图表', fontsize=16)
    
    # 1. 个人和公司余额变化趋势
    axes[0, 0].plot(df['交易日期'], df['个人余额'], label='个人余额', alpha=0.7)
    axes[0, 0].plot(df['交易日期'], df['公司余额'], label='公司余额', alpha=0.7)
    axes[0, 0].set_title('个人和公司余额变化趋势')
    axes[0, 0].set_xlabel('交易日期')
    axes[0, 0].set_ylabel('余额')
    axes[0, 0].legend()
    axes[0, 0].tick_params(axis='x', rotation=45)
    
    # 2. 资金流向类型分布
    if '资金流向类型' in df.columns:
        流向统计 = df['资金流向类型'].value_counts()
        axes[0, 1].pie(流向统计.values, labels=流向统计.index, autopct='%1.1f%%')
        axes[0, 1].set_title('资金流向类型分布')
    
    # 3. 个人资金占比分布
    axes[1, 0].hist(df['个人资金占比'], bins=20, alpha=0.7, color='blue')
    axes[1, 0].set_title('个人资金占比分布')
    axes[1, 0].set_xlabel('个人资金占比')
    axes[1, 0].set_ylabel('频次')
    
    # 4. 总余额变化
    axes[1, 1].plot(df['交易日期'], df['总余额'], color='green', alpha=0.7)
    axes[1, 1].set_title('总余额变化趋势')
    axes[1, 1].set_xlabel('交易日期')
    axes[1, 1].set_ylabel('总余额')
    axes[1, 1].tick_params(axis='x', rotation=45)
    
    plt.tight_layout()
    plt.show()

if __name__ == "__main__":
    # 分析Excel文件
    df = analyze_financial_data("流水.xlsx")
    
    # 如果数据读取成功，创建可视化
    if df is not None:
        create_visualizations(df)
