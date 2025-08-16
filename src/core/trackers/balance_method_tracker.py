"""
差额计算法追踪器
实现余额优先扣除的差额计算法，最大化复用现有模块
"""

from typing import Tuple, Dict, Any, Optional
import pandas as pd

from core.interfaces.tracker_interface import ITracker
from config import Config
from utils.logger import audit_logger
from models.behavior_analyzer import BehaviorAnalyzer


class BalanceMethodTracker(ITracker):
    """差额计算法追踪器 - 个人余额优先扣除"""
    
    def __init__(self):
        """初始化差额计算法追踪器"""
        # 简化的余额管理（替代FIFO队列）
        self._个人余额 = 0.0
        self._公司余额 = 0.0
        self._已初始化 = False
        
        # 累计统计
        self._累计挪用金额 = 0.0
        self._累计垫付金额 = 0.0  
        self._累计已归还公司本金 = 0.0
        self._累计非法所得 = 0.0
        self._总计个人分配利润 = 0.0
        self._总计公司分配利润 = 0.0
        
        # 投资产品管理（简化版）
        self._投资产品资金池 = {}
        self._投资产品交易记录 = []
        
        # 复用现有的行为分析器
        self._行为分析器 = BehaviorAnalyzer()
        
        audit_logger.debug("差额计算法追踪器初始化完成")
    
    def 初始化余额(self, 初始余额: float, 余额类型: str = '公司') -> None:
        """初始化余额"""
        if not self._已初始化 and 初始余额 > 0:
            if 余额类型 == '公司':
                self._公司余额 = 初始余额
            else:
                self._个人余额 = 初始余额
            self._已初始化 = True
            audit_logger.debug(f"差额计算法初始化余额: {初始余额:,.2f} (设为{余额类型}余额)")
    
    def 处理资金流入(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """
        处理资金流入 - 复用FIFO的分配规则
        收入分配规则与FIFO相同，确保一致性
        """
        if 金额 <= 0:
            return 0, 0, ""
        
        # 复用FIFO的收入分配逻辑
        if Config.is_personal_fund(资金属性):
            # 个人资金
            self._个人余额 += 金额
            return 1.0, 0.0, f"个人资金流入：{金额:,.2f}"
            
        elif Config.is_company_fund(资金属性):
            # 公司资金
            self._公司余额 += 金额
            return 0.0, 1.0, f"公司资金流入：{金额:,.2f}"
            
        else:
            # 其他情况，按当前余额比例分配（与FIFO逻辑一致）
            total_balance = self._个人余额 + self._公司余额
            if total_balance == 0:
                # 如果总余额为0，按默认规则处理
                audit_logger.warning(f"资金池为空，收到{金额:,.2f}，按默认规则处理")
                # 默认按1:1分配
                个人金额 = 金额 / 2
                公司金额 = 金额 / 2
                self._个人余额 += 个人金额
                self._公司余额 += 公司金额
                return 0.5, 0.5, f"混合资金流入：个人{个人金额:,.2f}，公司{公司金额:,.2f}"
            else:
                个人占比 = self._个人余额 / total_balance
                公司占比 = self._公司余额 / total_balance
                个人金额 = 金额 * 个人占比
                公司金额 = 金额 * 公司占比
                self._个人余额 += 个人金额
                self._公司余额 += 公司金额
                return 个人占比, 公司占比, f"混合资金流入：个人{个人金额:,.2f}，公司{公司金额:,.2f}"
    
    def 处理资金流出(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """
        处理资金流出 - 差额计算法核心逻辑
        个人余额优先扣除，不足部分算挪用
        """
        if 金额 <= 0:
            return 0, 0, ""
        
        # 检查是否为投资产品申购
        if Config.is_investment_product(资金属性):
            return self._处理投资产品申购(金额, 资金属性, 交易日期)
        else:
            return self._处理普通资金流出(金额, 资金属性, 交易日期)
    
    def _处理普通资金流出(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """
        处理普通资金流出 - 差额计算法核心
        根据资金属性决定优先扣除哪个余额
        """
        # 检查总可用资金
        total_available = self._个人余额 + self._公司余额
        if total_available <= 0:
            audit_logger.warning(f"资金池已空，无法支出{金额:,.2f}")
            return 0, 0, f"资金池已空，无法支出{金额:,.2f}"
        
        # 差额计算法核心逻辑：根据支出性质决定优先扣除逻辑
        if Config.is_personal_fund(资金属性):
            # 个人应付支出：个人余额优先扣除
            个人扣除 = min(金额, self._个人余额)
            剩余金额 = 金额 - 个人扣除
            公司扣除 = min(剩余金额, self._公司余额)
            
            # 个人支出使用公司资金 = 挪用
            if 公司扣除 > 0:
                self._累计挪用金额 += 公司扣除
                
        elif Config.is_company_fund(资金属性):
            # 公司应付支出：公司余额优先扣除
            公司扣除 = min(金额, self._公司余额)
            剩余金额 = 金额 - 公司扣除
            个人扣除 = min(剩余金额, self._个人余额)
            
            # 公司支出使用个人资金 = 垫付
            if 个人扣除 > 0:
                self._累计垫付金额 += 个人扣除
                
        else:
            # 其他情况：按原个人优先逻辑（保持向后兼容）
            个人扣除 = min(金额, self._个人余额)
            剩余金额 = 金额 - 个人扣除
            公司扣除 = min(剩余金额, self._公司余额)
        
        资金缺口 = 金额 - 个人扣除 - 公司扣除
        
        # 更新余额
        self._个人余额 -= 个人扣除
        self._公司余额 -= 公司扣除
        
        # 格式化数值
        self._累计挪用金额 = Config.format_number(self._累计挪用金额)
        self._累计垫付金额 = Config.format_number(self._累计垫付金额)
        
        # 复用BehaviorAnalyzer进行行为分析
        行为性质 = self._行为分析器.分析行为性质(资金属性, 个人扣除, 公司扣除, 金额)
        
        # 计算占比
        实际支出 = 个人扣除 + 公司扣除
        if 实际支出 > 0:
            个人占比 = 个人扣除 / 金额
            公司占比 = 公司扣除 / 金额
        else:
            个人占比 = 0
            公司占比 = 0
        
        # 添加资金缺口说明
        if 资金缺口 > Config.BALANCE_TOLERANCE:
            行为性质 = f"{行为性质}；资金缺口：{资金缺口:,.2f}"
        
        return 个人占比, 公司占比, 行为性质
    
    def _处理投资产品申购(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """
        处理投资产品申购 - 投资通常为个人行为，个人余额优先扣除
        """
        # 投资产品申购：个人余额优先扣除（投资通常是个人行为）
        个人扣除 = min(金额, self._个人余额)
        剩余金额 = 金额 - 个人扣除
        公司扣除 = min(剩余金额, self._公司余额)
        
        # 更新余额
        self._个人余额 -= 个人扣除
        self._公司余额 -= 公司扣除
        
        # 投资使用公司资金算挪用（个人投资用公司钱）
        if 公司扣除 > 0:
            self._累计挪用金额 += 公司扣除
            self._累计挪用金额 = Config.format_number(self._累计挪用金额)
        
        # 计算占比
        总扣除 = 个人扣除 + 公司扣除
        if 总扣除 > 0:
            个人占比 = 个人扣除 / 总扣除
            公司占比 = 公司扣除 / 总扣除
        else:
            个人占比 = 0
            公司占比 = 0
        
        # 更新投资产品资金池（记录占比用于赎回）
        self._更新投资产品资金池(资金属性, 金额, 个人占比, 公司占比)
        
        # 构造行为性质描述
        前缀 = 资金属性.split('-')[0]
        行为描述 = []
        if 公司扣除 > 0:
            行为描述.append(f"投资挪用：{公司扣除:,.2f}")
        if 个人扣除 > 0:
            行为描述.append(f"个人投资：{个人扣除:,.2f}")
        
        行为性质 = "；".join(行为描述) if 行为描述 else "无投资"
        
        return 个人占比, 公司占比, f"{前缀}申购-{资金属性}：{行为性质}"
    
    def _更新投资产品资金池(self, 投资产品编号: str, 金额: float, 个人占比: float, 公司占比: float) -> None:
        """更新投资产品资金池 - 简化版投资产品管理"""
        if 投资产品编号 not in self._投资产品资金池:
            self._投资产品资金池[投资产品编号] = {
                '总金额': 0,
                '个人占比': 0,
                '公司占比': 0,
                '累计申购': 0,
                '累计赎回': 0
            }
        
        产品信息 = self._投资产品资金池[投资产品编号]
        产品信息['总金额'] += 金额
        产品信息['累计申购'] += 金额
        
        # 更新占比（按最新申购计算）
        if 产品信息['总金额'] > 0:
            产品信息['个人占比'] = 个人占比
            产品信息['公司占比'] = 公司占比
        
        # 记录交易
        交易记录 = {
            '资金池名称': 投资产品编号,
            '入金': 金额,
            '出金': 0,
            '总余额': 产品信息['总金额'],
            '资金占比': f"个人{个人占比:.1%}，公司{公司占比:.1%}",
            '行为性质': f"入金（个人{金额*个人占比:,.0f}，公司{金额*公司占比:,.0f}）",
            '累计申购': 产品信息['累计申购'],
            '累计赎回': 产品信息['累计赎回']
        }
        self._投资产品交易记录.append(交易记录)
    
    def 处理投资产品赎回(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """
        处理投资产品赎回 - 按申购时占比分配收益
        """
        if 金额 <= 0:
            return 0, 0, ""
        
        投资产品编号 = 资金属性
        if not Config.is_investment_product(资金属性):
            return 0, 0, ""
        
        # 查找投资产品记录
        if 投资产品编号 not in self._投资产品资金池:
            # 无申购记录，按个人应收处理
            self._个人余额 += 金额
            前缀 = 投资产品编号.split('-')[0]
            return 1.0, 0.0, f"{前缀}收入-{投资产品编号}：个人应收{金额:,.2f}（无申购记录）"
        
        产品信息 = self._投资产品资金池[投资产品编号]
        个人占比 = 产品信息['个人占比']
        公司占比 = 产品信息['公司占比']
        
        # 按申购占比分配赎回收益
        个人返还 = 金额 * 个人占比
        公司返还 = 金额 * 公司占比
        
        # 归还到余额
        self._个人余额 += 个人返还
        self._公司余额 += 公司返还
        
        # 计算本金归还（用于抵消挪用）
        申购总额 = 产品信息['总金额']
        if 申购总额 > 0:
            赎回比例 = min(金额 / 申购总额, 1.0)
            归还的公司本金 = 申购总额 * 公司占比 * 赎回比例
            if 归还的公司本金 > 0:
                self._累计已归还公司本金 += 归还的公司本金
                self._累计已归还公司本金 = Config.format_number(self._累计已归还公司本金)
        
        # 更新产品信息
        产品信息['总金额'] -= min(金额, 申购总额)
        产品信息['累计赎回'] += 金额
        
        # 计算收益
        收益 = max(0, 金额 - min(金额, 申购总额))
        if 收益 > 0:
            个人收益 = 收益 * 个人占比
            公司收益 = 收益 * 公司占比
            self._总计个人分配利润 += 个人收益
            self._总计公司分配利润 += 公司收益
            self._总计个人分配利润 = Config.format_number(self._总计个人分配利润)
            self._总计公司分配利润 = Config.format_number(self._总计公司分配利润)
        
        # 记录交易
        交易记录 = {
            '资金池名称': 投资产品编号,
            '入金': 0,
            '出金': 金额,
            '总余额': 产品信息['总金额'],
            '资金占比': f"个人{个人占比:.1%}，公司{公司占比:.1%}",
            '行为性质': f"出金（个人{个人返还:,.0f}，公司{公司返还:,.0f}，收益{收益:,.0f}）",
            '累计申购': 产品信息['累计申购'],
            '累计赎回': 产品信息['累计赎回']
        }
        self._投资产品交易记录.append(交易记录)
        
        前缀 = 投资产品编号.split('-')[0]
        return 个人占比, 公司占比, f"{前缀}赎回-{投资产品编号}：个人{个人返还:,.2f}，公司{公司返还:,.2f}"
    
    def 获取状态摘要(self) -> Dict[str, Any]:
        """获取追踪器状态摘要"""
        return {
            '个人余额': Config.format_number(self._个人余额),
            '公司余额': Config.format_number(self._公司余额),
            '总余额': Config.format_number(self._个人余额 + self._公司余额),
            '投资产品数量': len(self._投资产品资金池),
            '累计挪用金额': Config.format_number(self._累计挪用金额),
            '累计垫付金额': Config.format_number(self._累计垫付金额),
            '累计非法所得': Config.format_number(self._累计非法所得),
            '累计已归还公司本金': Config.format_number(self._累计已归还公司本金),
            '总计个人分配利润': Config.format_number(self._总计个人分配利润),
            '总计公司分配利润': Config.format_number(self._总计公司分配利润),
            '已初始化': self._已初始化,
            '净挪用金额': Config.format_number(self._累计挪用金额 - self._累计已归还公司本金)
        }
    
    def 获取当前资金占比(self) -> Tuple[float, float]:
        """获取当前个人和公司资金占比"""
        total_balance = self._个人余额 + self._公司余额
        if total_balance == 0:
            return 0, 0
        return self._个人余额 / total_balance, self._公司余额 / total_balance
    
    def 生成投资产品交易记录Excel(self, 文件名: str = "投资产品交易记录.xlsx") -> None:
        """生成投资产品交易记录Excel"""
        if not self._投资产品交易记录:
            audit_logger.info("没有投资产品交易记录，跳过Excel生成")
            return
        
        try:
            import pandas as pd
            df = pd.DataFrame(self._投资产品交易记录)
            df.to_excel(文件名, index=False, engine='openpyxl')
            audit_logger.info(f"✅ 投资产品交易记录已保存至: {文件名}")
            audit_logger.info(f"📊 共记录 {len(self._投资产品交易记录)} 笔投资产品交易")
        except Exception as e:
            audit_logger.error(f"❌ 生成投资产品交易记录Excel失败: {e}")
    
    # 属性访问
    @property
    def 个人余额(self) -> float:
        return self._个人余额
        
    @property  
    def 公司余额(self) -> float:
        return self._公司余额
        
    @property
    def 累计挪用金额(self) -> float:
        return self._累计挪用金额
        
    @property
    def 累计垫付金额(self) -> float:
        return self._累计垫付金额
        
    @property
    def 累计已归还公司本金(self) -> float:
        return self._累计已归还公司本金
        
    @property
    def 总计个人分配利润(self) -> float:
        return self._总计个人分配利润
        
    @property
    def 总计公司分配利润(self) -> float:
        return self._总计公司分配利润
        
    @property
    def 已初始化(self) -> bool:
        return self._已初始化