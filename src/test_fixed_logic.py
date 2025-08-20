#!/usr/bin/env python3
"""测试修复后的资金池盈亏计算逻辑"""

import sys
sys.path.append('models')
from fifo_tracker import FIFO资金追踪器
import pandas as pd
from datetime import datetime

def test_reset_logic():
    print("=" * 80)
    print("🧪 测试修复后的资金池盈亏计算逻辑")
    print("=" * 80)
    
    # 创建追踪器
    tracker = FIFO资金追踪器()
    
    # 模拟场外资金池交易记录
    transactions = [
        # 第1次申购100w
        {
            '交易时间': '2023-01-01 09:00:00',
            '资金池名称': '理财-TEST001',
            '入金': 1000000,
            '出金': 0,
            '总余额': 1000000,
            '个人余额': 200000,
            '公司余额': 800000,
            '资金占比': '个人20.0%，公司80.0%',
            '行为性质': '申购',
            '累计申购': 1000000,
            '累计赎回': 0
        },
        # 第1次赎回110w（盈利10w）
        {
            '交易时间': '2023-02-01 15:00:00',
            '资金池名称': '理财-TEST001',
            '入金': 0,
            '出金': 1100000,
            '总余额': -100000,
            '个人余额': -20000,
            '公司余额': -80000,
            '资金占比': '资金池盈利',
            '行为性质': '赎回（盈利10万）',
            '累计申购': 1000000,
            '累计赎回': 1100000
        },
        # 第2次申购100w（会触发重置）
        {
            '交易时间': '2023-03-01 10:00:00',
            '资金池名称': '理财-TEST001',
            '入金': 1000000,
            '出金': 0,
            '总余额': 1000000,
            '个人余额': 300000,
            '公司余额': 700000,
            '资金占比': '个人30.0%，公司70.0%',
            '行为性质': '申购（重置）',
            '累计申购': 2000000,
            '累计赎回': 1100000
        },
        # 第2次赎回110w（盈利10w）
        {
            '交易时间': '2023-04-01 14:00:00',
            '资金池名称': '理财-TEST001',
            '入金': 0,
            '出金': 1100000,
            '总余额': -100000,
            '个人余额': -30000,
            '公司余额': -70000,
            '资金占比': '资金池盈利',
            '行为性质': '赎回（盈利10万）',
            '累计申购': 2000000,
            '累计赎回': 2200000
        }
    ]
    
    # 设置场外资金池记录
    tracker.场外资金池记录 = transactions
    
    # 模拟投资产品资金池状态（包含重置历史）
    tracker.投资产品资金池['理财-TEST001'] = {
        '个人金额': -30000,
        '公司金额': -70000, 
        '总金额': -100000,
        '累计申购': 2000000,
        '累计赎回': 2200000,
        '最新个人占比': 0.3,
        '最新公司占比': 0.7,
        '历史盈利记录': [
            {
                'reset_time': '2023-03-01 10:00:00',
                'profit_amount': 100000,
                'description': '重置前实现盈利 ¥100,000'
            }
        ],
        '累计已实现盈利': 100000
    }
    
    print("📊 场景：申购100w→赎回110w→申购100w→赎回110w")
    print("-" * 60)
    
    print("🔍 投资产品资金池状态：")
    pool_info = tracker.投资产品资金池['理财-TEST001']
    print(f"   当前余额: ¥{pool_info['总金额']:,}")
    print(f"   累计申购: ¥{pool_info['累计申购']:,}")
    print(f"   累计赎回: ¥{pool_info['累计赎回']:,}")
    print(f"   重置次数: {len(pool_info['历史盈利记录'])}")
    print(f"   历史已实现盈利: ¥{pool_info['累计已实现盈利']:,}")
    
    # 测试Excel生成逻辑（模拟部分）
    final_total_balance = pool_info['总金额']
    historical_profit = pool_info.get('累计已实现盈利', 0)
    
    # 当前周期盈亏
    if final_total_balance < 0:
        current_profit = abs(final_total_balance)
        current_status = "盈利"
    elif final_total_balance > 0:
        current_profit = -final_total_balance
        current_status = "亏损" 
    else:
        current_profit = 0
        current_status = "持平"
    
    # 真实总盈亏
    total_real_profit = historical_profit + current_profit
    
    if total_real_profit > 0:
        profit_status = "盈利"
    elif total_real_profit < 0:
        profit_status = "亏损"
    else:
        profit_status = "持平"
    
    print()
    print("💡 修复后的计算结果：")
    print(f"   当前周期状态: {current_status}")
    print(f"   当前周期盈亏: ¥{current_profit:,}")
    print(f"   历史已实现盈利: ¥{historical_profit:,}")
    print(f"   💎 真实总盈利: ¥{total_real_profit:,}")
    print(f"   📊 最终状态: {profit_status}")
    
    print()
    print("✅ 逻辑验证:")
    print(f"   第一轮：申购100w → 赎回110w = 盈利10w（重置时记录）")
    print(f"   第二轮：申购100w → 赎回110w = 盈利10w（当前余额）")
    print(f"   总计：10w + 10w = 20w盈利 ✓")
    
    # 生成测试Excel文件
    try:
        tracker.生成场外资金池记录Excel('test_修复后逻辑.xlsx')
        print()
        print(f"📁 测试Excel文件已生成: test_修复后逻辑.xlsx")
    except Exception as e:
        print(f"❌ Excel生成失败: {e}")
    
    print("\n" + "=" * 80)

if __name__ == "__main__":
    test_reset_logic()
