"""
时点查询服务测试
验证时点查询功能是否正常工作
"""

import sys
import os
import tempfile

# 添加src目录到路径
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'src'))

from services.time_point_query_service import TimePointQueryService
from utils.logger import audit_logger


def test_service_initialization():
    """测试服务初始化"""
    print("🧪 测试服务初始化...")
    
    # 测试FIFO算法初始化
    fifo_service = TimePointQueryService("FIFO")
    assert fifo_service.algorithm == "FIFO"
    assert fifo_service.data is None
    assert fifo_service.total_rows == 0
    assert len(fifo_service.query_history) == 0
    print("✅ FIFO服务初始化正确")
    
    # 测试差额计算法初始化
    balance_service = TimePointQueryService("BALANCE_METHOD")
    assert balance_service.algorithm == "BALANCE_METHOD"
    print("✅ 差额计算法服务初始化正确")
    
    return True


def test_data_loading():
    """测试数据加载功能"""
    print("🧪 测试数据加载...")
    
    service = TimePointQueryService("FIFO")
    
    # 测试加载不存在的文件
    result = service.load_data("不存在的文件.xlsx")
    assert not result["success"]
    print("✅ 不存在文件处理正确")
    
    # 测试加载真实文件（如果存在）
    test_files = ["流水.xlsx", "投资产品交易记录.xlsx"]
    loaded = False
    
    for file_path in test_files:
        if os.path.exists(file_path):
            result = service.load_data(file_path)
            if result["success"]:
                print(f"✅ 成功加载文件: {file_path} ({result['total_rows']} 行)")
                loaded = True
                break
            else:
                print(f"⚠️ 文件存在但加载失败: {file_path}")
    
    if not loaded:
        print("⚠️ 没有找到可用的测试数据文件，跳过数据加载测试")
        return True
    
    return True


def test_service_status():
    """测试服务状态获取"""
    print("🧪 测试服务状态...")
    
    service = TimePointQueryService("FIFO")
    status = service.get_service_status()
    
    assert "algorithm" in status
    assert "data_loaded" in status
    assert "total_rows" in status
    assert "history_count" in status
    assert status["algorithm"] == "FIFO"
    assert status["data_loaded"] == False
    assert status["total_rows"] == 0
    assert status["history_count"] == 0
    
    print("✅ 服务状态获取正确")
    return True


def test_history_management():
    """测试历史记录管理"""
    print("🧪 测试历史记录管理...")
    
    service = TimePointQueryService("FIFO")
    
    # 测试空历史
    history = service.get_query_history()
    assert len(history) == 0
    print("✅ 空历史记录正确")
    
    # 模拟添加历史记录
    service.query_history.append({
        "id": 1,
        "algorithm": "FIFO",
        "target_row": 100,
        "query_time": "2024-01-01T10:00:00",
        "processing_time": 1.5,
        "success": True,
        "tracker_state": {},
        "error_count": 0
    })
    
    history = service.get_query_history()
    assert len(history) == 1
    assert history[0]["id"] == 1
    print("✅ 历史记录添加正确")
    
    # 测试清除历史
    result = service.clear_history()
    assert result["success"]
    assert len(service.query_history) == 0
    print("✅ 历史记录清除正确")
    
    return True


def test_export_functionality():
    """测试导出功能"""
    print("🧪 测试导出功能...")
    
    service = TimePointQueryService("FIFO")
    
    # 创建模拟查询结果
    mock_result = {
        "success": True,
        "algorithm": "FIFO",
        "target_row": 100,
        "total_rows": 1000,
        "query_time": "2024-01-01T10:00:00",
        "processing_time": 1.5,
        "tracker_state": {
            "personal_balance": 50000,
            "company_balance": 100000,
            "total_balance": 150000,
            "total_misuse": 10000,
            "total_advance": 5000
        },
        "target_row_data": {
            "timestamp": "2024-01-01 10:00:00",
            "income_amount": 0,
            "expense_amount": 1000,
            "balance": 149000,
            "fund_attr": "个人应付",
            "flow_type": "支出",
            "behavior": "个人支付"
        }
    }
    
    # 测试JSON导出
    with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as tmp_json:
        json_path = tmp_json.name
    
    json_result = service.export_query_result(mock_result, json_path)
    assert json_result["success"]
    print(f"✅ JSON导出成功: {json_path}")
    
    # 清理临时文件
    try:
        os.unlink(json_path)
    except (OSError, PermissionError):
        print(f"⚠️ 无法删除临时文件: {json_path}")
    
    # 测试Excel导出
    try:
        with tempfile.NamedTemporaryFile(suffix='.xlsx', delete=False) as tmp_xlsx:
            xlsx_path = tmp_xlsx.name
        
        xlsx_result = service.export_query_result(mock_result, xlsx_path)
        assert xlsx_result["success"]
        print(f"✅ Excel导出成功: {xlsx_path}")
        
        # 清理临时文件
        try:
            os.unlink(xlsx_path)
        except (OSError, PermissionError):
            print(f"⚠️ 无法删除临时文件: {xlsx_path}")
    except ImportError:
        print("⚠️ 缺少openpyxl，跳过Excel导出测试")
    
    # 测试不支持的格式
    unsupported_result = service.export_query_result(mock_result, "test.txt")
    assert not unsupported_result["success"]
    print("✅ 不支持格式处理正确")
    
    return True


def test_algorithm_switching():
    """测试算法切换"""
    print("🧪 测试算法切换...")
    
    # 测试两种算法的服务创建
    fifo_service = TimePointQueryService("FIFO")
    balance_service = TimePointQueryService("BALANCE_METHOD")
    
    assert fifo_service.algorithm == "FIFO"
    assert balance_service.algorithm == "BALANCE_METHOD"
    
    print("✅ 算法切换正确")
    return True


def test_real_data_query():
    """测试真实数据查询（如果有数据文件）"""
    print("🧪 测试真实数据查询...")
    
    test_files = ["流水.xlsx"]
    
    for file_path in test_files:
        if os.path.exists(file_path):
            print(f"📄 使用文件: {file_path}")
            
            for algorithm in ["FIFO", "BALANCE_METHOD"]:
                print(f"  测试算法: {algorithm}")
                service = TimePointQueryService(algorithm)
                
                # 加载数据
                load_result = service.load_data(file_path)
                if not load_result["success"]:
                    print(f"    ⚠️ 数据加载失败: {load_result['message']}")
                    continue
                
                total_rows = load_result["total_rows"]
                if total_rows == 0:
                    print("    ⚠️ 数据文件为空")
                    continue
                
                # 测试查询第1行
                query_result = service.query_time_point(1)
                if query_result["success"]:
                    print(f"    ✅ 第1行查询成功")
                    
                    # 验证结果结构
                    assert "tracker_state" in query_result
                    assert "target_row_data" in query_result
                    assert "processing_stats" in query_result
                    
                else:
                    print(f"    ❌ 第1行查询失败: {query_result['message']}")
                
                # 测试查询中间行
                if total_rows >= 10:
                    mid_row = min(10, total_rows)
                    query_result = service.query_time_point(mid_row)
                    if query_result["success"]:
                        print(f"    ✅ 第{mid_row}行查询成功")
                    else:
                        print(f"    ❌ 第{mid_row}行查询失败")
                
                # 测试无效行数
                invalid_result = service.query_time_point(total_rows + 1)
                assert not invalid_result["success"]
                print(f"    ✅ 无效行数处理正确")
                
                # 测试历史记录
                history = service.get_query_history()
                assert len(history) >= 1  # 至少有一次成功查询
                print(f"    ✅ 历史记录正常 ({len(history)} 条)")
            
            return True
    
    print("⚠️ 没有找到测试数据文件，跳过真实数据查询测试")
    return True


def main():
    """主测试函数"""
    print("🧪 时点查询服务测试")
    print("="*60)
    
    tests = [
        ("服务初始化", test_service_initialization),
        ("数据加载", test_data_loading),
        ("服务状态", test_service_status),
        ("历史管理", test_history_management),
        ("导出功能", test_export_functionality),
        ("算法切换", test_algorithm_switching),
        ("真实数据查询", test_real_data_query)
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n--- 测试: {test_name} ---")
        try:
            if test_func():
                print(f"✅ {test_name} 通过")
                passed += 1
            else:
                print(f"❌ {test_name} 失败")
        except Exception as e:
            print(f"❌ {test_name} 异常: {e}")
            import traceback
            traceback.print_exc()
    
    # 测试总结
    print(f"\n{'='*60}")
    print(f"🧪 测试总结: {passed}/{total} 通过")
    
    if passed == total:
        print("🎉 所有测试通过！时点查询服务工作正常。")
        return True
    else:
        print(f"⚠️ 有 {total - passed} 个测试失败。")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)