#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
脱敏流水数据生成器
独立工具，生成10,000条完全脱敏的银行流水+投资理财数据
"""

import pandas as pd
import numpy as np
from datetime import datetime, timedelta
import random
import string

class SampleDataGenerator:
    def __init__(self):
        self.current_balance = 100000.0  # 初始余额10万
        self.current_time = datetime(2019, 1, 1, 9, 0, 0)
        self.transactions = []
        self.fund_pools = []
        self.active_pools = {}  # 已申购的资金池
        
    def initialize_fund_pools(self):
        """初始化资金池产品库"""
        # 1. 理财产品（420个）
        for i in range(420):
            code = f"理财-{self.random_letter_code(4)}{2019 + i//20:04d}{(i%12)+1:02d}{(i%28)+1:02d}{i:04d}"
            self.fund_pools.append(("wealth", code))
        
        # 2. 投资产品（20个）
        investment_names = [
            "蓝筹精选001", "成长动力基金", "价值发现", "科技创新主题", "消费升级ETF",
            "医药健康指数", "新能源产业", "金融地产", "制造业龙头", "数字经济",
            "绿色低碳", "高端装备", "生物医药", "人工智能", "新材料产业",
            "量化策略", "债券基金", "货币基金", "混合基金", "指数基金"
        ]
        for name in investment_names:
            self.fund_pools.append(("investment", f"投资-{name}"))
        
        # 3. 保险产品（30个）
        insurance_names = [
            "安康终身001", "福泽年金", "如意重疾险", "康宁医疗", "财富增值",
            "平安一生", "健康无忧", "财富传承", "安享晚年", "保障至尊",
            "理财增利", "医疗无忧", "意外保障", "教育金", "养老金计划",
            "重疾保障", "定期寿险", "终身寿险", "年金保险", "万能险",
            "分红险", "投连险", "健康险", "意外险", "旅游险",
            "车险综合", "家财险", "责任险", "信用险", "保证险"
        ]
        for name in insurance_names:
            self.fund_pools.append(("insurance", f"保险-{name}"))
        
        # 4. 关联银行卡（12个）
        bank_names = [
            "华夏银行", "民生银行", "光大银行", "浦发银行", "兴业银行", "中信银行",
            "平安银行", "广发银行", "江苏银行", "北京银行", "上海银行", "宁波银行"
        ]
        for name in bank_names:
            self.fund_pools.append(("bank", f"关联银行卡-{name}"))
    
    def random_letter_code(self, length):
        """生成随机字母代码"""
        return ''.join(random.choices(string.ascii_uppercase, k=length))
    
    def generate_transactions(self, count):
        """生成指定数量的交易记录"""
        print(f"开始生成{count}条脱敏流水数据...")
        
        for i in range(count):
            # 决定交易类型
            rand = random.random()
            if rand < 0.10:
                self.generate_personal_transaction(True)   # 个人应收
            elif rand < 0.20:
                self.generate_personal_transaction(False)  # 个人应付
            elif rand < 0.42:
                self.generate_company_transaction(True)    # 公司应收
            elif rand < 0.63:
                self.generate_company_transaction(False)   # 公司应付
            elif rand < 0.82:
                self.generate_fund_pool_purchase()         # 资金池申购
            else:
                self.generate_fund_pool_redemption()       # 资金池赎回
            
            # 推进时间
            self.advance_time()
            
            # 进度报告
            if (i + 1) % 1000 == 0:
                print(f"已生成 {i+1}/{count} 条记录")
        
        print(f"数据生成完成！共{len(self.transactions)}条记录")
    
    def generate_personal_transaction(self, is_receivable):
        """生成个人交易"""
        amount = self.generate_amount("small")
        
        if is_receivable:
            income, expense = amount, 0
            fund_attr = "个人应收"
        else:
            income, expense = 0, amount
            fund_attr = "个人应付"
        
        self.add_transaction(income, expense, fund_attr)
    
    def generate_company_transaction(self, is_receivable):
        """生成公司交易"""
        amount = self.generate_amount("medium")
        
        if is_receivable:
            income, expense = amount, 0
            fund_attr = "公司应收"
        else:
            income, expense = 0, amount
            fund_attr = "公司应付"
        
        self.add_transaction(income, expense, fund_attr)
    
    def generate_fund_pool_purchase(self):
        """生成资金池申购"""
        pool_type, pool_code = random.choice(self.fund_pools)
        amount = self.generate_amount("large")
        
        # 申购 = 支出
        self.add_transaction(0, amount, pool_code)
        
        # 记录到活跃资金池
        self.active_pools[pool_code] = {
            'amount': amount,
            'purchase_time': self.current_time
        }
    
    def generate_fund_pool_redemption(self):
        """生成资金池赎回"""
        if not self.active_pools:
            # 如果没有活跃资金池，生成公司交易代替
            self.generate_company_transaction(True)
            return
        
        # 选择一个活跃资金池
        pool_code = random.choice(list(self.active_pools.keys()))
        pool_info = self.active_pools.pop(pool_code)
        
        # 赎回金额：原金额的50%-150%（可盈可亏）
        profit_loss_ratio = random.uniform(0.5, 1.5)
        redemption_amount = pool_info['amount'] * profit_loss_ratio
        
        # 赎回 = 收入
        self.add_transaction(redemption_amount, 0, pool_code)
    
    def generate_amount(self, range_type):
        """生成交易金额"""
        if range_type == "small":
            return round(random.uniform(10, 1000), 2)
        elif range_type == "medium":
            return round(random.uniform(1000, 10000), 2)
        elif range_type == "large":
            return round(random.uniform(10000, 100000), 2)
        else:  # huge
            return round(random.uniform(100000, 1000000), 2)
    
    def add_transaction(self, income, expense, fund_attribute):
        """添加交易记录"""
        # 计算新余额
        new_balance = self.current_balance + income - expense
        
        # 防止余额为负
        if new_balance < 0:
            return  # 跳过此交易
        
        # 生成交易时间码（6位数字）
        time_code = (self.current_time.hour * 10000 + 
                    self.current_time.minute * 100 + 
                    self.current_time.second)
        
        # 创建交易记录
        transaction = {
            '交易日期': self.current_time.date(),
            '交易时间': time_code,
            '交易收入金额': income if income > 0 else None,
            '交易支出金额': expense if expense > 0 else None,
            '余额': new_balance,
            '资金属性': fund_attribute
        }
        
        self.transactions.append(transaction)
        self.current_balance = new_balance
    
    def advance_time(self):
        """推进时间"""
        # 随机推进1-72小时
        hours = random.randint(1, 72)
        minutes = random.randint(0, 59)
        seconds = random.randint(0, 59)
        
        self.current_time += timedelta(hours=hours, minutes=minutes, seconds=seconds)
    
    def export_to_excel(self, filename):
        """导出到Excel文件"""
        print(f"正在导出到Excel文件: {filename}")
        
        # 创建DataFrame
        df = pd.DataFrame(self.transactions)
        
        # 确保列的顺序
        df = df[['交易日期', '交易时间', '交易收入金额', '交易支出金额', '余额', '资金属性']]
        
        # 导出到Excel
        df.to_excel(filename, index=False, sheet_name="需计算")
        
        print(f"Excel文件导出完成！")
        print(f"记录总数: {len(df)}")
        print(f"时间跨度: {df['交易日期'].min()} 至 {df['交易日期'].max()}")
        print(f"最终余额: {self.current_balance:,.2f}元")

def main():
    print("=== 脱敏流水数据生成器 ===")
    print("正在初始化...")
    
    generator = SampleDataGenerator()
    generator.initialize_fund_pools()
    
    print("资金池产品库初始化完成！")
    print(f"- 理财产品: 420个")
    print(f"- 投资产品: 20个")
    print(f"- 保险产品: 30个")
    print(f"- 关联银行卡: 12个")
    print(f"- 总计: {len(generator.fund_pools)}个产品")
    
    # 生成10,000条交易记录
    generator.generate_transactions(10000)
    
    # 导出到Excel
    generator.export_to_excel("脱敏流水.xlsx")
    
    print("\n=== 生成完成 ===")
    print("文件位置: 脱敏流水.xlsx")
    print("可直接用于FLUX系统测试和演示")

if __name__ == "__main__":
    main()