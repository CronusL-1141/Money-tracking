"""
测试流水完整性验证器
验证新的FlowIntegrityValidator是否正常工作
"""

import pandas as pd
from utils.flow_integrity_validator import FlowIntegrityValidator
from utils.logger import audit_logger
from config import Config

def test_flow_integrity_validator():
    """测试流水完整性验证器"""
    
    print("="*60)
    print("测试流水完整性验证器")
    print("="*60)
    
    try:
        # 读取原始数据
        print("正在读取原始数据...")
        df = pd.read_excel(Config.DEFAULT_INPUT_FILE)
        
        # 基本预处理（模拟DataProcessor的部分功能）
        print("进行基本预处理...")
        df['交易日期'] = pd.to_datetime(df['交易日期'])
        
        # 简单的时间处理
        df['完整时间戳'] = df['交易日期']
        
        # 显示前几行数据
        print("\n前5行数据预览:")
        print(df[['交易日期', '交易收入金额', '交易支出金额', '余额']].head())
        
        # 创建验证器并执行验证
        print("\n开始流水完整性验证...")
        validator = FlowIntegrityValidator()
        result = validator.validate_flow_integrity(df)
        
        # 显示验证结果
        print("\n验证结果摘要:")
        print(f"总行数: {result['total_rows']}")
        print(f"发现错误: {result['errors_count']}个")
        print(f"成功修复: {result['optimizations_count']}个")
        print(f"验证结果: {'✅ 通过' if result['is_valid'] else '❌ 失败'}")
        
        if result['errors']:
            print("\n错误详情:")
            for i, error in enumerate(result['errors'][:10]):  # 最多显示10个错误
                print(f"  {i+1}. 第{error['row']}行: {error['message']}")
                if i >= 9 and len(result['errors']) > 10:
                    print(f"  ... 还有{len(result['errors']) - 10}个错误")
                    break
        
        return result
        
    except Exception as e:
        print(f"测试过程中出现错误: {e}")
        import traceback
        traceback.print_exc()
        return None

def test_simple_balance_continuity():
    """测试简单的余额连贯性检查"""
    
    print("\n" + "="*60)
    print("测试简单余额连贯性检查")
    print("="*60)
    
    # 创建测试数据
    test_data = {
        '交易日期': pd.to_datetime(['2024-01-01', '2024-01-02', '2024-01-03', '2024-01-04']),
        '交易收入金额': [0, 1000, 0, 500],
        '交易支出金额': [0, 0, 300, 0],
        '余额': [5000, 6000, 5700, 6200],  # 5000 + 1000 = 6000, 6000 - 300 = 5700, 5700 + 500 = 6200
        '完整时间戳': pd.to_datetime(['2024-01-01', '2024-01-02', '2024-01-03', '2024-01-04'])
    }
    
    df = pd.DataFrame(test_data)
    print("测试数据:")
    print(df)
    
    validator = FlowIntegrityValidator()
    result = validator.validate_flow_integrity(df)
    
    print(f"\n测试结果: {'✅ 通过' if result['is_valid'] else '❌ 失败'}")
    return result

def test_problematic_balance():
    """测试有问题的余额数据"""
    
    print("\n" + "="*60)
    print("测试有问题的余额数据")
    print("="*60)
    
    # 创建有问题的测试数据
    test_data = {
        '交易日期': pd.to_datetime(['2024-01-01', '2024-01-02', '2024-01-03']),
        '交易收入金额': [0, 1000, 0],
        '交易支出金额': [0, 0, 300],
        '余额': [5000, 6000, 5800],  # 最后一个余额错误：应该是5700，但写成了5800
        '完整时间戳': pd.to_datetime(['2024-01-01', '2024-01-02', '2024-01-03'])
    }
    
    df = pd.DataFrame(test_data)
    print("有问题的测试数据:")
    print(df)
    
    validator = FlowIntegrityValidator()
    result = validator.validate_flow_integrity(df)
    
    print(f"\n测试结果: {'✅ 通过' if result['is_valid'] else '❌ 失败'}")
    if result['errors']:
        print("发现的错误:")
        for error in result['errors']:
            print(f"  第{error['row']}行: {error['message']}")
    
    return result

if __name__ == "__main__":
    print("开始测试流水完整性验证器...")
    
    # 测试1: 简单的正确数据
    test_simple_balance_continuity()
    
    # 测试2: 有问题的数据
    test_problematic_balance()
    
    # 测试3: 真实数据
    print("\n开始测试真实数据...")
    test_flow_integrity_validator()
    
    print("\n所有测试完成！") 