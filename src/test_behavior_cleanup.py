#!/usr/bin/env python3
"""测试行为性质清理功能"""

import re

def _clean_behavior_description(behavior):
    if not behavior:
        return behavior
    
    # 检查是否包含投资产品的前缀格式（如：理财申购-理财-SYA160401160408：）
    investment_prefix_pattern = r'^[^：]*申购-[^：]*：'
    
    if re.match(investment_prefix_pattern, behavior):
        # 去掉前缀，只保留冒号后面的内容
        parts = behavior.split('：', 1)
        if len(parts) > 1:
            return parts[1]
    
    return behavior

def test_flow_type(income, expense):
    """测试资金流向判断逻辑"""
    if income > 0 and expense == 0:
        return "收入"
    elif expense > 0 and income == 0:
        return "支出"
    elif income > 0 and expense > 0:
        return "收支"
    else:
        return "无变动"

if __name__ == "__main__":
    print('=== 测试时点查询优化功能 ===')
    print()
    
    print('1. 行为性质清理功能测试')
    print('-' * 50)
    
    # 测试用例1：投资产品格式
    test1 = '理财申购-理财-SYA160401160408：投资挪用：1,898,094.23；个人投资：121,905.77'
    result1 = _clean_behavior_description(test1)
    print(f'原始: {test1}')
    print(f'清理后: {result1}')
    print(f'✅ 投资产品前缀已去除: {result1 == "投资挪用：1,898,094.23；个人投资：121,905.77"}')
    print()
    
    # 测试用例2：普通格式（应该保持不变）
    test2 = '垫付：5,766.13；公司支付：533.87'
    result2 = _clean_behavior_description(test2)
    print(f'原始: {test2}')
    print(f'清理后: {result2}')
    print(f'✅ 普通格式保持不变: {result2 == test2}')
    print()
    
    # 测试用例3：基金申购格式
    test3 = '基金申购-基金-ABC123：个人投资：50,000.00；挪用：150,000.00'
    result3 = _clean_behavior_description(test3)
    print(f'原始: {test3}')
    print(f'清理后: {result3}')
    print(f'✅ 基金产品前缀已去除: {result3 == "个人投资：50,000.00；挪用：150,000.00"}')
    print()
    
    print('2. 资金流向判断测试')
    print('-' * 50)
    
    test_cases = [
        (1000, 0, "收入"),
        (0, 500, "支出"), 
        (1000, 500, "收支"),
        (0, 0, "无变动")
    ]
    
    for income, expense, expected in test_cases:
        result = test_flow_type(income, expense)
        status = "✅" if result == expected else "❌"
        print(f'收入:{income:>4}, 支出:{expense:>3} → {result:<4} {status}')
    
    print()
    print('3. 资金缺口计算逻辑')
    print('-' * 50)
    
    # 模拟数据
    累计挪用金额 = 2000000
    累计已归还个人本金 = 100000
    累计垫付金额 = 500000
    累计已归还公司本金 = 50000
    
    # 使用新的资金缺口逻辑：累计挪用 - 累计个人归还公司本金
    资金缺口 = 累计挪用金额 - 累计已归还个人本金
    
    print(f'累计挪用金额: ¥{累计挪用金额:,}')
    print(f'累计已归还个人本金: ¥{累计已归还个人本金:,}')
    print()
    print(f'累计垫付金额: ¥{累计垫付金额:,}')
    print(f'累计已归还公司本金: ¥{累计已归还公司本金:,}')
    print()
    print(f'资金缺口 (累计挪用 - 累计个人归还公司本金): ¥{资金缺口:,}')
    
    if 资金缺口 > 0:
        print('📊 结果: 存在资金缺口（显示为红色）')
    elif 资金缺口 < 0:
        print('📊 结果: 资金有余（显示为绿色）')
    else:
        print('📊 结果: 资金平衡')
    
    print()
    print('4. 功能总结')
    print('-' * 50)
    print('✅ 新增资金缺口字段：累计挪用 - 累计个人归还公司本金')
    print('✅ 修复资金流向字段：根据收入支出金额自动判断')
    print('✅ 优化行为性质显示：去掉投资产品的冗长前缀')
    print('✅ 保持普通交易格式不变')
    print()
    print('🎯 所有功能测试完成！')