"""
时点查询CLI接口
提供命令行方式的时点查询功能
"""

import argparse
import sys
import json
import os
from typing import Optional

# 添加src目录到路径
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from services.time_point_query_service import TimePointQueryService
from core.factories.tracker_factory import TrackerFactory
from utils.logger import audit_logger
from config import Config


def main():
    """主CLI函数"""
    parser = argparse.ArgumentParser(description='时点查询工具 - 查询特定交易行的系统状态')
    parser.add_argument(
        '--file', '-f',
        required=True,
        help='Excel数据文件路径'
    )
    parser.add_argument(
        '--row', '-r',
        type=int,
        help='目标行数 (1-based)'
    )
    parser.add_argument(
        '--algorithm', '-a',
        choices=['FIFO', 'BALANCE_METHOD'],
        default='FIFO',
        help='算法类型 (默认: FIFO)'
    )
    parser.add_argument(
        '--output', '-o',
        help='导出结果文件路径 (.json 或 .xlsx)'
    )
    parser.add_argument(
        '--history',
        action='store_true',
        help='显示查询历史记录'
    )
    parser.add_argument(
        '--interactive', '-i',
        action='store_true',
        help='启动交互模式'
    )
    parser.add_argument(
        '--list-algorithms',
        action='store_true',
        help='列出所有可用算法'
    )
    
    args = parser.parse_args()
    
    # 显示可用算法
    if args.list_algorithms:
        print("可用算法:")
        for algo, desc in TrackerFactory.get_algorithms_info().items():
            print(f"  {algo}: {desc}")
        return
    
    # 创建查询服务
    query_service = TimePointQueryService(algorithm=args.algorithm)
    
    # 加载数据
    print(f"🔄 加载数据文件: {args.file}")
    load_result = query_service.load_data(args.file)
    
    if not load_result["success"]:
        print(f"❌ {load_result['message']}")
        sys.exit(1)
    
    print(f"✅ {load_result['message']}")
    
    # 交互模式
    if args.interactive:
        run_interactive_mode(query_service)
        return
    
    # 显示历史记录
    if args.history:
        show_history(query_service)
        return
    
    # 单次查询模式
    if args.row is None:
        print("请指定目标行数 (-r) 或启动交互模式 (-i)")
        sys.exit(1)
    
    # 执行查询
    print(f"🔍 查询第 {args.row} 行状态 (使用 {args.algorithm} 算法)")
    query_result = query_service.query_time_point(args.row)
    
    if query_result["success"]:
        print_query_result(query_result)
        
        # 导出结果
        if args.output:
            print(f"\n💾 导出结果到: {args.output}")
            export_result = query_service.export_query_result(query_result, args.output)
            if export_result["success"]:
                print(f"✅ {export_result['message']}")
            else:
                print(f"❌ {export_result['message']}")
    else:
        print(f"❌ 查询失败: {query_result['message']}")
        sys.exit(1)


def run_interactive_mode(query_service: TimePointQueryService):
    """运行交互模式"""
    print("\n" + "="*60)
    print("🔍 时点查询交互模式")
    print(f"算法: {query_service.algorithm}")
    print(f"数据行数: {query_service.total_rows}")
    print("="*60)
    
    print("\n💡 可用命令:")
    print("  query <行数>      - 查询指定行状态")
    print("  history [n]       - 显示查询历史(默认10条)")
    print("  export <文件路径>  - 导出最近查询结果")
    print("  status            - 显示服务状态")
    print("  switch <算法>     - 切换算法 (FIFO|BALANCE_METHOD)")
    print("  clear             - 清除历史记录")
    print("  quit              - 退出")
    
    last_query_result = None
    
    while True:
        try:
            user_input = input(f"\n[{query_service.algorithm}] > ").strip()
            
            if not user_input:
                continue
            
            parts = user_input.split()
            command = parts[0].lower()
            
            if command == 'quit':
                print("👋 退出时点查询工具")
                break
                
            elif command == 'query':
                if len(parts) < 2:
                    print("用法: query <行数>")
                    continue
                try:
                    row_num = int(parts[1])
                    print(f"🔍 查询第 {row_num} 行...")
                    query_result = query_service.query_time_point(row_num)
                    
                    if query_result["success"]:
                        print_query_result(query_result)
                        last_query_result = query_result
                    else:
                        print(f"❌ {query_result['message']}")
                        
                except ValueError:
                    print("请输入有效的行数")
                    
            elif command == 'history':
                limit = 10
                if len(parts) > 1:
                    try:
                        limit = int(parts[1])
                    except ValueError:
                        print("请输入有效的数量")
                        continue
                show_history(query_service, limit)
                
            elif command == 'export':
                if len(parts) < 2:
                    print("用法: export <文件路径>")
                    continue
                if last_query_result is None:
                    print("没有可导出的查询结果，请先执行查询")
                    continue
                
                file_path = parts[1]
                export_result = query_service.export_query_result(last_query_result, file_path)
                if export_result["success"]:
                    print(f"✅ {export_result['message']}")
                else:
                    print(f"❌ {export_result['message']}")
                    
            elif command == 'status':
                status = query_service.get_service_status()
                print("\n📊 服务状态:")
                for key, value in status.items():
                    print(f"  {key}: {value}")
                    
            elif command == 'switch':
                if len(parts) < 2:
                    print("用法: switch <算法> (FIFO|BALANCE_METHOD)")
                    continue
                new_algorithm = parts[1].upper()
                if new_algorithm not in ['FIFO', 'BALANCE_METHOD']:
                    print("无效算法，可用: FIFO, BALANCE_METHOD")
                    continue
                
                # 创建新的查询服务
                old_algorithm = query_service.algorithm
                query_service = TimePointQueryService(algorithm=new_algorithm)
                
                # 重新加载数据
                # 这里假设原数据文件路径存储在某处，实际应该优化
                print(f"🔄 切换算法: {old_algorithm} -> {new_algorithm}")
                print("⚠️ 请重新加载数据文件")
                
            elif command == 'clear':
                result = query_service.clear_history()
                print(f"✅ {result['message']}")
                
            else:
                print("❌ 未知命令。输入命令: query, history, export, status, switch, clear, quit")
                
        except KeyboardInterrupt:
            print("\n\n👋 退出...")
            break
        except Exception as e:
            print(f"❌ 发生错误: {e}")
            audit_logger.error(f"CLI工具出错: {str(e)}")


def print_query_result(result: dict):
    """打印查询结果"""
    print(f"\n✅ 查询成功 (耗时: {result['processing_time']:.2f}s)")
    print("="*50)
    
    # 基本信息
    print(f"📋 基本信息:")
    print(f"  算法: {result['algorithm']}")
    print(f"  目标行: {result['target_row']}/{result['total_rows']}")
    print(f"  查询时间: {result['query_time']}")
    
    # 追踪器状态
    if 'tracker_state' in result:
        state = result['tracker_state']
        print(f"\n💰 资金状态:")
        print(f"  个人余额: {state.get('personal_balance', 0):,.2f}")
        print(f"  公司余额: {state.get('company_balance', 0):,.2f}")
        print(f"  总余额: {state.get('total_balance', 0):,.2f}")
        print(f"  累计挪用: {state.get('total_misuse', 0):,.2f}")
        print(f"  累计垫付: {state.get('total_advance', 0):,.2f}")
        print(f"  已归还本金: {state.get('total_returned', 0):,.2f}")
        print(f"  个人利润: {state.get('personal_profit', 0):,.2f}")
        print(f"  公司利润: {state.get('company_profit', 0):,.2f}")
    
    # 目标行数据
    if 'target_row_data' in result:
        row_data = result['target_row_data']
        print(f"\n📄 第{result['target_row']}行数据:")
        print(f"  时间: {row_data.get('timestamp', 'N/A')}")
        print(f"  收入金额: {row_data.get('income_amount', 0):,.2f}")
        print(f"  支出金额: {row_data.get('expense_amount', 0):,.2f}")
        print(f"  余额: {row_data.get('balance', 0):,.2f}")
        print(f"  资金属性: {row_data.get('fund_attr', 'N/A')}")
        print(f"  流向类型: {row_data.get('flow_type', 'N/A')}")
        print(f"  行为性质: {row_data.get('behavior', 'N/A')}")
    
    # 处理统计
    if 'processing_stats' in result:
        stats = result['processing_stats']
        print(f"\n📊 处理统计:")
        print(f"  处理步骤: {stats.get('total_steps', 0)}")
        print(f"  错误数量: {stats.get('error_count', 0)}")
    
    # 错误记录
    if 'errors' in result and result['errors']:
        print(f"\n⚠️ 错误记录:")
        for error in result['errors'][-3:]:  # 显示最近3个错误
            print(f"  第{error['row']}行: {error.get('error', '余额不匹配')}")


def show_history(query_service: TimePointQueryService, limit: int = 10):
    """显示查询历史"""
    history = query_service.get_query_history(limit)
    
    if not history:
        print("📜 暂无查询历史")
        return
    
    print(f"\n📜 查询历史 (最近 {len(history)} 条):")
    print("-" * 80)
    print(f"{'ID':<4} {'算法':<12} {'行数':<8} {'查询时间':<20} {'耗时(s)':<8} {'状态'}")
    print("-" * 80)
    
    for item in history:
        status = "✅" if item['success'] else "❌"
        error_info = f" ({item['error_count']}个错误)" if item.get('error_count', 0) > 0 else ""
        
        print(f"{item['id']:<4} {item['algorithm']:<12} {item['target_row']:<8} "
              f"{item['query_time'][:19]:<20} {item['processing_time']:<8.2f} {status}{error_info}")


if __name__ == "__main__":
    main()