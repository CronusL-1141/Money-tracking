"""
差额计算法修复验证测试
专门测试用户提出的场景
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'src'))

from core.factories.tracker_factory import TrackerFactory


def test_user_scenario():
    """测试用户提出的具体场景"""
    print("🧪 测试用户场景：公司余额20万，个人余额10万，公司应付支出10万")
    
    # 创建差额计算法追踪器
    tracker = TrackerFactory.create_tracker("BALANCE_METHOD")
    
    print("\n--- 初始设置 ---")
    # 模拟初始状态：公司余额20万，个人余额10万
    tracker._公司余额 = 200000  # 直接设置用于测试
    tracker._个人余额 = 100000
    tracker._已初始化 = True
    
    print(f"公司余额: {tracker.公司余额:,.2f}")
    print(f"个人余额: {tracker.个人余额:,.2f}")
    
    print("\n--- 处理公司应付支出10万 ---")
    
    # 公司应付支出10万
    个人占比, 公司占比, 行为性质 = tracker.处理资金流出(100000, "公司应付", None)
    
    print(f"个人占比: {个人占比:.1%}")
    print(f"公司占比: {公司占比:.1%}")
    print(f"行为性质: {行为性质}")
    
    print("\n--- 结果检验 ---")
    print(f"处理后公司余额: {tracker.公司余额:,.2f}")
    print(f"处理后个人余额: {tracker.个人余额:,.2f}")
    print(f"累计挪用金额: {tracker.累计挪用金额:,.2f}")
    print(f"累计垫付金额: {tracker.累计垫付金额:,.2f}")
    
    # 验证结果
    expected_company_balance = 100000  # 20万 - 10万 = 10万
    expected_personal_balance = 100000  # 个人余额不变
    expected_misuse = 0  # 不应该有挪用
    expected_advance = 0  # 不应该有垫付
    
    print(f"\n--- 验证期望结果 ---")
    print(f"期望公司余额: {expected_company_balance:,.2f}")
    print(f"期望个人余额: {expected_personal_balance:,.2f}")
    print(f"期望挪用金额: {expected_misuse:,.2f}")
    print(f"期望垫付金额: {expected_advance:,.2f}")
    
    # 检查是否正确
    success = True
    if abs(tracker.公司余额 - expected_company_balance) > 0.01:
        print(f"❌ 公司余额错误！实际: {tracker.公司余额:,.2f}, 期望: {expected_company_balance:,.2f}")
        success = False
    else:
        print(f"✅ 公司余额正确")
    
    if abs(tracker.个人余额 - expected_personal_balance) > 0.01:
        print(f"❌ 个人余额错误！实际: {tracker.个人余额:,.2f}, 期望: {expected_personal_balance:,.2f}")
        success = False
    else:
        print(f"✅ 个人余额正确")
    
    if abs(tracker.累计挪用金额 - expected_misuse) > 0.01:
        print(f"❌ 挪用金额错误！实际: {tracker.累计挪用金额:,.2f}, 期望: {expected_misuse:,.2f}")
        success = False
    else:
        print(f"✅ 挪用金额正确")
    
    if abs(tracker.累计垫付金额 - expected_advance) > 0.01:
        print(f"❌ 垫付金额错误！实际: {tracker.累计垫付金额:,.2f}, 期望: {expected_advance:,.2f}")
        success = False
    else:
        print(f"✅ 垫付金额正确")
    
    return success


def test_reverse_scenario():
    """测试相反场景：个人应付支出"""
    print("\n" + "="*60)
    print("🧪 测试相反场景：公司余额20万，个人余额10万，个人应付支出10万")
    
    # 创建差额计算法追踪器
    tracker = TrackerFactory.create_tracker("BALANCE_METHOD")
    
    print("\n--- 初始设置 ---")
    # 模拟初始状态：公司余额20万，个人余额10万  
    tracker._公司余额 = 200000
    tracker._个人余额 = 100000
    tracker._已初始化 = True
    
    print(f"公司余额: {tracker.公司余额:,.2f}")
    print(f"个人余额: {tracker.个人余额:,.2f}")
    
    print("\n--- 处理个人应付支出10万 ---")
    
    # 个人应付支出10万
    个人占比, 公司占比, 行为性质 = tracker.处理资金流出(100000, "个人应付", None)
    
    print(f"个人占比: {个人占比:.1%}")
    print(f"公司占比: {公司占比:.1%}")
    print(f"行为性质: {行为性质}")
    
    print("\n--- 结果检验 ---")
    print(f"处理后公司余额: {tracker.公司余额:,.2f}")
    print(f"处理后个人余额: {tracker.个人余额:,.2f}")
    print(f"累计挪用金额: {tracker.累计挪用金额:,.2f}")
    print(f"累计垫付金额: {tracker.累计垫付金额:,.2f}")
    
    # 验证结果（个人应付：个人余额优先扣除）
    expected_company_balance = 200000  # 公司余额不变
    expected_personal_balance = 0      # 10万 - 10万 = 0
    expected_misuse = 0       # 个人钱够，不需要挪用
    expected_advance = 0      # 不涉及垫付
    
    print(f"\n--- 验证期望结果 ---")
    print(f"期望公司余额: {expected_company_balance:,.2f}")
    print(f"期望个人余额: {expected_personal_balance:,.2f}")
    print(f"期望挪用金额: {expected_misuse:,.2f}")
    print(f"期望垫付金额: {expected_advance:,.2f}")
    
    # 检查是否正确
    success = True
    if abs(tracker.公司余额 - expected_company_balance) > 0.01:
        print(f"❌ 公司余额错误！")
        success = False
    else:
        print(f"✅ 公司余额正确")
    
    if abs(tracker.个人余额 - expected_personal_balance) > 0.01:
        print(f"❌ 个人余额错误！")
        success = False
    else:
        print(f"✅ 个人余额正确")
    
    return success


def test_cross_usage_scenario():
    """测试跨用场景：个人应付15万（超出个人余额）"""
    print("\n" + "="*60)
    print("🧪 测试跨用场景：公司余额20万，个人余额10万，个人应付支出15万")
    
    # 创建差额计算法追踪器
    tracker = TrackerFactory.create_tracker("BALANCE_METHOD")
    
    print("\n--- 初始设置 ---")
    # 模拟初始状态：公司余额20万，个人余额10万
    tracker._公司余额 = 200000
    tracker._个人余额 = 100000
    tracker._已初始化 = True
    
    print(f"公司余额: {tracker.公司余额:,.2f}")
    print(f"个人余额: {tracker.个人余额:,.2f}")
    
    print("\n--- 处理个人应付支出15万 ---")
    
    # 个人应付支出15万（超出个人余额5万）
    个人占比, 公司占比, 行为性质 = tracker.处理资金流出(150000, "个人应付", None)
    
    print(f"个人占比: {个人占比:.1%}")
    print(f"公司占比: {公司占比:.1%}")
    print(f"行为性质: {行为性质}")
    
    print("\n--- 结果检验 ---")
    print(f"处理后公司余额: {tracker.公司余额:,.2f}")
    print(f"处理后个人余额: {tracker.个人余额:,.2f}")
    print(f"累计挪用金额: {tracker.累计挪用金额:,.2f}")
    print(f"累计垫付金额: {tracker.累计垫付金额:,.2f}")
    
    # 验证结果
    # 个人应付15万：个人余额10万 + 公司余额5万（挪用）
    expected_company_balance = 150000  # 20万 - 5万 = 15万
    expected_personal_balance = 0      # 10万 - 10万 = 0
    expected_misuse = 50000   # 挪用5万
    expected_advance = 0      # 不涉及垫付
    
    print(f"\n--- 验证期望结果 ---")
    print(f"期望公司余额: {expected_company_balance:,.2f}")
    print(f"期望个人余额: {expected_personal_balance:,.2f}")
    print(f"期望挪用金额: {expected_misuse:,.2f}")
    print(f"期望垫付金额: {expected_advance:,.2f}")
    
    # 检查是否正确
    success = True
    checks = [
        ("公司余额", tracker.公司余额, expected_company_balance),
        ("个人余额", tracker.个人余额, expected_personal_balance), 
        ("挪用金额", tracker.累计挪用金额, expected_misuse),
        ("垫付金额", tracker.累计垫付金额, expected_advance)
    ]
    
    for name, actual, expected in checks:
        if abs(actual - expected) > 0.01:
            print(f"❌ {name}错误！实际: {actual:,.2f}, 期望: {expected:,.2f}")
            success = False
        else:
            print(f"✅ {name}正确")
    
    return success


def main():
    """主测试函数"""
    print("🔧 差额计算法修复验证测试\n")
    
    tests = [
        ("用户场景（公司应付）", test_user_scenario),
        ("相反场景（个人应付）", test_reverse_scenario),
        ("跨用场景（个人应付超额）", test_cross_usage_scenario)
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n{'='*60}")
        print(f"测试: {test_name}")
        print(f"{'='*60}")
        
        try:
            if test_func():
                print(f"\n✅ {test_name} 测试通过")
                passed += 1
            else:
                print(f"\n❌ {test_name} 测试失败")
        except Exception as e:
            print(f"\n❌ {test_name} 测试异常: {e}")
            import traceback
            traceback.print_exc()
    
    # 测试总结
    print(f"\n{'='*60}")
    print(f"🧪 修复验证总结")
    print(f"{'='*60}")
    print(f"通过: {passed}/{total}")
    
    if passed == total:
        print(f"🎉 所有测试通过！差额计算法修复成功。")
        return True
    else:
        print(f"⚠️ 有 {total - passed} 个测试失败，需要进一步修复。")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)