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
        print("可用算法:", file=sys.stderr)
        for algo, desc in TrackerFactory.get_algorithms_info().items():
            print(f"  {algo}: {desc}", file=sys.stderr)
        return
    
    # 创建查询服务
    query_service = TimePointQueryService(algorithm=args.algorithm)
    
    # 加载数据
    print(f"🔄 加载数据文件: {args.file}", file=sys.stderr)
    load_result = query_service.load_data(args.file)
    
    if not load_result["success"]:
        print(f"❌ {load_result['message']}", file=sys.stderr)
        sys.exit(1)
    
    print(f"✅ {load_result['message']}", file=sys.stderr)
    
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
        print("请指定目标行数 (-r) 或启动交互模式 (-i)", file=sys.stderr)
        sys.exit(1)
    
    # 执行查询
    print(f"🔍 查询第 {args.row} 行状态 (使用 {args.algorithm} 算法)", file=sys.stderr)
    sys.stderr.flush()
    sys.stdout.flush()
    print("💰 计算初始余额...", file=sys.stderr)
    sys.stderr.flush()
    sys.stdout.flush()
    
    # 获取初始余额（第一行的余额）
    if hasattr(query_service, 'data') and query_service.data is not None and len(query_service.data) > 0:
        initial_balance = query_service.data.iloc[0]['余额'] if '余额' in query_service.data.columns else 0.0
        print(f"📊 初始余额: {initial_balance:,.2f} 元", file=sys.stderr)
    sys.stderr.flush()
    sys.stdout.flush()
    
    print(f"🚀 开始时点查询分析... (目标行: {args.row})", file=sys.stderr)
    sys.stderr.flush()
    sys.stdout.flush()
    print(f"📋 需要处理到第 {args.row} 行交易记录", file=sys.stderr)
    sys.stderr.flush()
    sys.stdout.flush()
    
    # 显示查询进度
    total_rows = query_service.total_rows if hasattr(query_service, 'total_rows') else args.row
    percentage = (args.row / total_rows * 100) if total_rows > 0 else 0
    print(f"⏳ 查询进度: {args.row}/{total_rows} ({percentage:.1f}%)", file=sys.stderr)
    sys.stderr.flush()
    sys.stdout.flush()
    
    query_result = query_service.query_time_point(args.row)
    
    if query_result["success"]:
        print(f"✅ 所有 {args.row} 条交易记录处理完成", file=sys.stderr)
        sys.stderr.flush()
        sys.stdout.flush()
        print(f"📈 生成查询结果...", file=sys.stderr)
        sys.stderr.flush()
        sys.stdout.flush()
        
        # 显示处理统计
        if 'processing_stats' in query_result:
            stats = query_result['processing_stats']
            if stats.get('error_count', 0) > 0:
                print(f"⚠️ 发现 {stats['error_count']} 个处理问题", file=sys.stderr)
                sys.stderr.flush()
                sys.stdout.flush()
        
        print(f"✅ 时点查询分析完成！", file=sys.stderr)
        sys.stderr.flush()
        sys.stdout.flush()
        print(f"📊 处理行数: {args.row}", file=sys.stderr)
        sys.stderr.flush()
        sys.stdout.flush()
        
        # 显示查询的关键信息
        if 'tracker_state' in query_result and query_result['tracker_state']:
            state = query_result['tracker_state']
            total_balance = state.get('total_balance', 0)
            print(f"💰 查询时点余额: {total_balance:,.2f} 元", file=sys.stderr)
            sys.stderr.flush()
            sys.stdout.flush()
            
        print("===== 时点查询结束 =====", file=sys.stderr)
        sys.stderr.flush()
        sys.stdout.flush()
        
        print_query_result(query_result)
        
        # 导出结果
        if args.output:
            print(f"\n💾 导出结果到: {args.output}", file=sys.stderr)
            export_result = query_service.export_query_result(query_result, args.output)
            if export_result["success"]:
                print(f"✅ {export_result['message']}", file=sys.stderr)
            else:
                print(f"❌ {export_result['message']}", file=sys.stderr)
    else:
        print(f"❌ 查询失败: {query_result['message']}", file=sys.stderr)
        sys.exit(1)


def run_interactive_mode(query_service: TimePointQueryService):
    """运行交互模式"""
    print("\n" + "="*60, file=sys.stderr)
    print("🔍 时点查询交互模式", file=sys.stderr)
    print(f"算法: {query_service.algorithm}", file=sys.stderr)
    print(f"数据行数: {query_service.total_rows}", file=sys.stderr)
    print("="*60, file=sys.stderr)
    
    print("\n💡 可用命令:", file=sys.stderr)
    print("  query <行数>      - 查询指定行状态", file=sys.stderr)
    print("  history [n]       - 显示查询历史(默认10条)", file=sys.stderr)
    print("  export <文件路径>  - 导出最近查询结果", file=sys.stderr)
    print("  status            - 显示服务状态", file=sys.stderr)
    print("  switch <算法>     - 切换算法 (FIFO|BALANCE_METHOD)", file=sys.stderr)
    print("  clear             - 清除历史记录", file=sys.stderr)
    print("  quit              - 退出", file=sys.stderr)
    
    last_query_result = None
    
    while True:
        try:
            user_input = input(f"\n[{query_service.algorithm}] > ").strip()
            
            if not user_input:
                continue
            
            parts = user_input.split()
            command = parts[0].lower()
            
            if command == 'quit':
                print("👋 退出时点查询工具", file=sys.stderr)
                break
                
            elif command == 'query':
                if len(parts) < 2:
                    print("用法: query <行数>", file=sys.stderr)
                    continue
                try:
                    row_num = int(parts[1])
                    print(f"🔍 查询第 {row_num} 行...", file=sys.stderr)
                    query_result = query_service.query_time_point(row_num)
                    
                    if query_result["success"]:
                        print_query_result(query_result)
                        last_query_result = query_result
                    else:
                        print(f"❌ {query_result['message']}", file=sys.stderr)
                        
                except ValueError:
                    print("请输入有效的行数", file=sys.stderr)
                    
            elif command == 'history':
                limit = 10
                if len(parts) > 1:
                    try:
                        limit = int(parts[1])
                    except ValueError:
                        print("请输入有效的数量", file=sys.stderr)
                        continue
                show_history(query_service, limit)
                
            elif command == 'export':
                if len(parts) < 2:
                    print("用法: export <文件路径>", file=sys.stderr)
                    continue
                if last_query_result is None:
                    print("没有可导出的查询结果，请先执行查询", file=sys.stderr)
                    continue
                
                file_path = parts[1]
                export_result = query_service.export_query_result(last_query_result, file_path)
                if export_result["success"]:
                    print(f"✅ {export_result['message']}", file=sys.stderr)
                else:
                    print(f"❌ {export_result['message']}", file=sys.stderr)
                    
            elif command == 'status':
                status = query_service.get_service_status()
                print("\n📊 服务状态:", file=sys.stderr)
                for key, value in status.items():
                    print(f"  {key}: {value}", file=sys.stderr)
                    
            elif command == 'switch':
                if len(parts) < 2:
                    print("用法: switch <算法> (FIFO|BALANCE_METHOD)", file=sys.stderr)
                    continue
                new_algorithm = parts[1].upper()
                if new_algorithm not in ['FIFO', 'BALANCE_METHOD']:
                    print("无效算法，可用: FIFO, BALANCE_METHOD", file=sys.stderr)
                    continue
                
                # 创建新的查询服务
                old_algorithm = query_service.algorithm
                query_service = TimePointQueryService(algorithm=new_algorithm)
                
                # 重新加载数据
                # 这里假设原数据文件路径存储在某处，实际应该优化
                print(f"🔄 切换算法: {old_algorithm} -> {new_algorithm}", file=sys.stderr)
                print("⚠️ 请重新加载数据文件", file=sys.stderr)
                
            elif command == 'clear':
                result = query_service.clear_history()
                print(f"✅ {result['message']}", file=sys.stderr)
                
            else:
                print("❌ 未知命令。输入命令: query, history, export, status, switch, clear, quit", file=sys.stderr)
                
        except KeyboardInterrupt:
            print("\n\n👋 退出...", file=sys.stderr)
            break
        except Exception as e:
            print(f"❌ 发生错误: {e}", file=sys.stderr)
            audit_logger.error(f"CLI工具出错: {str(e)}")


def print_query_result(result: dict):
    """打印查询结果（JSON输出到stdout，不再向stderr输出详细摘要）"""
    print(f"✅ 查询成功 (耗时: {result['processing_time']:.2f}s)", file=sys.stderr)
    sys.stderr.flush()
    sys.stdout.flush()
    
    # 不再在日志中显示详细信息摘要，这些信息已包含在JSON结果中
    # 只显示成功信息和耗时
    
    # 错误记录
    if 'errors' in result and result['errors']:
        print(f"\n⚠️ 错误记录:", file=sys.stderr)
        for error in result['errors'][-3:]:  # 显示最近3个错误
            print(f"  第{error['row']}行: {error.get('error', '余额不匹配')}", file=sys.stderr)
    
    # 输出JSON格式结果供Rust后端解析（保持stdout纯净）
    print("JSON_RESULT_START")
    sys.stdout.flush()
    print(json.dumps(result, ensure_ascii=False, default=str))
    sys.stdout.flush()
    print("JSON_RESULT_END")
    sys.stdout.flush()


def show_history(query_service: TimePointQueryService, limit: int = 10):
    """显示查询历史"""
    history = query_service.get_query_history(limit)
    
    if not history:
        print("📜 暂无查询历史", file=sys.stderr)
        return
    
    print(f"\n📜 查询历史 (最近 {len(history)} 条):", file=sys.stderr)
    print("-" * 80, file=sys.stderr)
    print(f"{'ID':<4} {'算法':<12} {'行数':<8} {'查询时间':<20} {'耗时(s)':<8} {'状态'}", file=sys.stderr)
    print("-" * 80, file=sys.stderr)
    
    for item in history:
        status = "✅" if item['success'] else "❌"
        error_info = f" ({item['error_count']}个错误)" if item.get('error_count', 0) > 0 else ""
        
        print(f"{item['id']:<4} {item['algorithm']:<12} {item['target_row']:<8} "
              f"{item['query_time'][:19]:<20} {item['processing_time']:<8.2f} {status}{error_info}", file=sys.stderr)


if __name__ == "__main__":
    main()