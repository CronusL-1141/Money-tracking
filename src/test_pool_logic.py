#!/usr/bin/env python3
"""测试场外资金池盈亏计算逻辑"""

def current_logic(total_purchase, total_redemption, final_total_balance, final_personal_balance, final_company_balance):
    """当前的盈亏计算逻辑"""
    print(f"=== 当前逻辑计算 ===")
    print(f"总申购: ¥{total_purchase:,.0f}")
    print(f"总赎回: ¥{total_redemption:,.0f}")
    print(f"最终总余额: ¥{final_total_balance:,.0f}")
    print(f"最终个人余额: ¥{final_personal_balance:,.0f}")
    print(f"最终公司余额: ¥{final_company_balance:,.0f}")
    print()
    
    # 当前逻辑
    net_amount = total_purchase - total_redemption  # 净投入
    profit_loss = final_total_balance - net_amount if net_amount != 0 else 0  # 盈亏
    
    print(f"净投入金额: ¥{net_amount:,.0f} (申购-赎回)")
    print(f"盈亏金额: ¥{profit_loss:,.0f} (最终余额-净投入)")
    
    if profit_loss > 0:
        profit_status = "盈利"
    elif profit_loss < 0:
        profit_status = "亏损" 
    else:
        profit_status = "持平"
        
    print(f"状态: {profit_status}")
    print(f"个人{profit_status}: ¥{final_personal_balance:,.0f}")
    print(f"公司{profit_status}: ¥{final_company_balance:,.0f}")
    
    return profit_status, profit_loss

def proposed_logic(total_purchase, total_redemption, final_total_balance, final_personal_balance, final_company_balance):
    """用户建议的逻辑"""
    print(f"=== 建议逻辑计算 ===")
    print(f"总申购: ¥{total_purchase:,.0f}")
    print(f"总赎回: ¥{total_redemption:,.0f}")
    print(f"最终总余额: ¥{final_total_balance:,.0f}")
    print(f"最终个人余额: ¥{final_personal_balance:,.0f}")
    print(f"最终公司余额: ¥{final_company_balance:,.0f}")
    print()
    
    # 建议逻辑
    if final_total_balance < 0:
        # 资金池为负 → 赎回 > 申购 → 盈利
        profit_status = "盈利"
        # 盈利金额 = |最终余额| = 赎回超出申购的部分
        profit_amount = abs(final_total_balance)
    elif final_total_balance > 0:
        # 资金池为正 → 赎回 < 申购 → 对主账户是亏损（钱还在投资中）
        profit_status = "亏损"
        # 亏损金额 = 最终余额 = 还未赎回的投资金额
        profit_amount = final_total_balance
    else:
        profit_status = "持平"
        profit_amount = 0
    
    print(f"逻辑: 最终余额 {'< 0 → 盈利' if final_total_balance < 0 else '> 0 → 亏损' if final_total_balance > 0 else '= 0 → 持平'}")
    print(f"状态: {profit_status}")
    print(f"盈亏金额: ¥{profit_amount:,.0f}")
    print(f"个人{profit_status}: ¥{final_personal_balance:,.0f}")
    print(f"公司{profit_status}: ¥{final_company_balance:,.0f}")
    
    return profit_status, profit_amount

if __name__ == "__main__":
    print("场外资金池盈亏计算逻辑对比")
    print("=" * 60)
    print()
    
    # 测试用例1：盈利情况（赎回多于申购）
    print("📈 测试用例1：盈利情况")
    print("-" * 40)
    print("情况：申购1,000,000，赎回1,100,000（赚了100,000）")
    print("最终资金池余额：-100,000（负数表示已全部赎回还多得100,000）")
    print("个人余额：-20,000，公司余额：-80,000")
    print()
    
    current_logic(1000000, 1100000, -100000, -20000, -80000)
    print()
    proposed_logic(1000000, 1100000, -100000, -20000, -80000)
    print("\n" + "="*60 + "\n")
    
    # 测试用例2：亏损情况（赎回少于申购）  
    print("📉 测试用例2：亏损情况")
    print("-" * 40)
    print("情况：申购1,000,000，赎回500,000（还有500,000在投资中）")
    print("最终资金池余额：500,000（正数表示还有资金在投资）")
    print("个人余额：100,000，公司余额：400,000")
    print()
    
    current_logic(1000000, 500000, 500000, 100000, 400000)
    print()
    proposed_logic(1000000, 500000, 500000, 100000, 400000)
    print("\n" + "="*60 + "\n")
    
    # 测试用例3：完全平衡
    print("⚖️ 测试用例3：完全平衡")
    print("-" * 40)
    print("情况：申购1,000,000，赎回1,000,000（不赚不赔）")
    print("最终资金池余额：0")
    print("个人余额：0，公司余额：0")
    print()
    
    current_logic(1000000, 1000000, 0, 0, 0)
    print()
    proposed_logic(1000000, 1000000, 0, 0, 0)
    print("\n" + "="*60 + "\n")
    
    print("🤔 请确认哪种逻辑更符合业务需求：")
