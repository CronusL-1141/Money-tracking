"""
双算法架构测试脚本
验证FIFO和差额计算法是否正常工作
"""

import sys
import os

# 添加src目录到路径
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'src'))

from core.factories.tracker_factory import TrackerFactory
from services.audit_service import AuditService
from utils.logger import audit_logger


def test_tracker_factory():
    """测试追踪器工厂"""
    print("🔧 测试追踪器工厂...")
    
    # 测试获取可用算法
    algorithms = TrackerFactory.get_available_algorithms()
    print(f"✅ 可用算法: {algorithms}")
    
    # 测试创建FIFO追踪器
    try:
        fifo_tracker = TrackerFactory.create_tracker("FIFO")
        print(f"✅ FIFO追踪器创建成功: {type(fifo_tracker).__name__}")
    except Exception as e:
        print(f"❌ FIFO追踪器创建失败: {e}")
        return False
    
    # 测试创建差额计算法追踪器
    try:
        balance_tracker = TrackerFactory.create_tracker("BALANCE_METHOD")
        print(f"✅ 差额计算法追踪器创建成功: {type(balance_tracker).__name__}")
    except Exception as e:
        print(f"❌ 差额计算法追踪器创建失败: {e}")
        return False
    
    # 测试无效算法
    try:
        invalid_tracker = TrackerFactory.create_tracker("INVALID")
        print(f"❌ 应该抛出异常，但没有")
        return False
    except ValueError:
        print(f"✅ 无效算法正确抛出异常")
    
    return True


def test_tracker_interfaces():
    """测试追踪器接口"""
    print("\n🔧 测试追踪器接口...")
    
    algorithms = ["FIFO", "BALANCE_METHOD"]
    
    for algorithm in algorithms:
        print(f"\n--- 测试 {algorithm} 追踪器 ---")
        try:
            tracker = TrackerFactory.create_tracker(algorithm)
            
            # 测试初始化
            tracker.初始化余额(100000, '公司')
            print(f"✅ 初始化余额成功")
            
            # 测试基本属性访问
            print(f"个人余额: {tracker.个人余额:,.2f}")
            print(f"公司余额: {tracker.公司余额:,.2f}")
            print(f"已初始化: {tracker.已初始化}")
            
            # 测试收入处理
            个人占比, 公司占比, 行为性质 = tracker.处理资金流入(50000, "个人应收", None)
            print(f"收入处理: 个人占比{个人占比:.1%}, 公司占比{公司占比:.1%}")
            
            # 测试支出处理
            个人占比, 公司占比, 行为性质 = tracker.处理资金流出(30000, "个人应付", None)
            print(f"支出处理: 个人占比{个人占比:.1%}, 公司占比{公司占比:.1%}")
            
            # 测试状态获取
            status = tracker.获取状态摘要()
            print(f"✅ 状态获取成功，包含 {len(status)} 项信息")
            
        except Exception as e:
            print(f"❌ {algorithm} 追踪器测试失败: {e}")
            import traceback
            traceback.print_exc()
            return False
    
    return True


def test_audit_service():
    """测试审计服务"""
    print("\n🔧 测试审计服务...")
    
    algorithms = ["FIFO", "BALANCE_METHOD"]
    
    for algorithm in algorithms:
        print(f"\n--- 测试 {algorithm} 审计服务 ---")
        try:
            service = AuditService(algorithm)
            print(f"✅ {algorithm} 审计服务创建成功")
            
            # 测试算法信息获取
            info = service.get_algorithm_info()
            print(f"算法信息: {info}")
            
            # 测试算法切换
            other_algo = "BALANCE_METHOD" if algorithm == "FIFO" else "FIFO"
            success = service.switch_algorithm(other_algo)
            if success:
                print(f"✅ 算法切换成功: {algorithm} -> {other_algo}")
                # 切换回原算法
                service.switch_algorithm(algorithm)
            else:
                print(f"❌ 算法切换失败")
                return False
                
        except Exception as e:
            print(f"❌ {algorithm} 审计服务测试失败: {e}")
            import traceback
            traceback.print_exc()
            return False
    
    return True


def test_simple_scenario():
    """测试简单场景，比较两种算法结果"""
    print("\n🧮 测试简单场景比较...")
    
    # 模拟交易场景
    transactions = [
        ("收入", 100000, "公司应收"),  # 公司收入10万
        ("收入", 50000, "个人应收"),   # 个人收入5万  
        ("支出", 30000, "个人应付"),   # 个人支出3万
        ("支出", 80000, "个人应付"),   # 个人支出8万（会产生挪用）
    ]
    
    results = {}
    
    for algorithm in ["FIFO", "BALANCE_METHOD"]:
        print(f"\n--- {algorithm} 算法模拟 ---")
        tracker = TrackerFactory.create_tracker(algorithm)
        
        # 初始余额0（从交易中累积）
        
        for transaction_type, amount, fund_attr in transactions:
            if transaction_type == "收入":
                个人占比, 公司占比, 行为性质 = tracker.处理资金流入(amount, fund_attr, None)
            else:
                个人占比, 公司占比, 行为性质 = tracker.处理资金流出(amount, fund_attr, None)
            
            print(f"{transaction_type} {amount:,} ({fund_attr}): {行为性质}")
        
        # 记录最终结果
        results[algorithm] = {
            "个人余额": tracker.个人余额,
            "公司余额": tracker.公司余额,
            "累计挪用": tracker.累计挪用金额,
            "累计垫付": tracker.累计垫付金额
        }
        
        print(f"最终状态:")
        for key, value in results[algorithm].items():
            print(f"  {key}: {value:,.2f}")
    
    # 比较结果
    print(f"\n📊 算法对比:")
    print(f"{'指标':<12} {'FIFO':<12} {'差额计算法':<12} {'差异':<12}")
    print("-" * 50)
    
    for metric in results["FIFO"].keys():
        fifo_val = results["FIFO"][metric]
        balance_val = results["BALANCE_METHOD"][metric]
        diff = balance_val - fifo_val
        print(f"{metric:<12} {fifo_val:<12,.2f} {balance_val:<12,.2f} {diff:<12,.2f}")
    
    return True


def main():
    """主测试函数"""
    print("🧪 开始双算法架构测试...\n")
    
    tests = [
        ("追踪器工厂", test_tracker_factory),
        ("追踪器接口", test_tracker_interfaces), 
        ("审计服务", test_audit_service),
        ("简单场景", test_simple_scenario),
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
    print(f"🧪 测试总结")
    print(f"{'='*60}")
    print(f"通过: {passed}/{total}")
    
    if passed == total:
        print(f"🎉 所有测试通过！双算法架构工作正常。")
        return True
    else:
        print(f"⚠️ 有 {total - passed} 个测试失败，需要修复。")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)