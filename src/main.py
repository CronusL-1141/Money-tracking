"""
新主程序入口 - 支持双算法切换
整合重构后的服务层，提供完整的审计分析功能
"""

import argparse
import sys
import os
from typing import Optional

# 添加src目录到Python路径，支持从项目根目录运行
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from services.audit_service import AuditService
from core.factories.tracker_factory import TrackerFactory
from utils.logger import audit_logger
from config import Config


def main():
    """主函数 - 支持命令行算法选择"""
    
    # 解析命令行参数
    parser = argparse.ArgumentParser(description='FIFO资金追踪审计系统 v2.0 - 支持双算法')
    parser.add_argument(
        '--algorithm', '-a', 
        choices=['FIFO', 'BALANCE_METHOD'], 
        default='FIFO',
        help='选择算法类型：FIFO（先进先出）或 BALANCE_METHOD（差额计算法）'
    )
    parser.add_argument(
        '--input', '-i',
        default=Config.DEFAULT_INPUT_FILE,
        help='输入Excel文件路径'
    )
    parser.add_argument(
        '--output', '-o',
        help='输出Excel文件路径（默认根据算法自动生成）'
    )
    parser.add_argument(
        '--list-algorithms', '-l',
        action='store_true',
        help='列出所有可用算法'
    )
    parser.add_argument(
        '--compare', '-c',
        action='store_true',
        help='比较两种算法的结果'
    )
    
    args = parser.parse_args()
    
    # 显示可用算法
    if args.list_algorithms:
        print("可用算法:")
        for algo, desc in TrackerFactory.get_algorithms_info().items():
            print(f"  {algo}: {desc}")
        return
    
    # 比较模式
    if args.compare:
        return compare_algorithms(args.input)
    
    # 单算法分析模式
    return run_single_analysis(args.algorithm, args.input, args.output)


def run_single_analysis(algorithm: str, input_file: str, output_file: Optional[str] = None) -> None:
    """
    运行单算法分析
    
    Args:
        algorithm: 算法类型
        input_file: 输入文件
        output_file: 输出文件
    """
    try:
        # 显示算法信息
        algo_desc = TrackerFactory.get_algorithm_description(algorithm)
        print(f"\n🚀 启动算法: {algorithm}")
        print(f"📝 算法描述: {algo_desc}")
        print(f"📂 输入文件: {input_file}")
        sys.stdout.flush()  # 强制刷新输出缓冲区
        
        # 创建审计服务
        audit_service = AuditService(algorithm=algorithm)
        
        # 分析数据
        result_df = audit_service.analyze_financial_data(input_file, output_file)
        
        if result_df is not None:
            print(f"\n✅ {algorithm}算法分析完成！")
            print(f"📊 处理行数: {len(result_df):,}")
            if output_file:
                print(f"💾 结果已保存至: {output_file}")
            else:
                print(f"💾 结果已保存至: {algorithm}_资金追踪结果.xlsx")
            print(f"📋 投资产品记录: 投资产品交易记录_{algorithm}.xlsx")
        else:
            print(f"\n❌ {algorithm}算法分析失败！")
            sys.exit(1)
            
    except Exception as e:
        audit_logger.error(f"{algorithm}分析过程出错: {e}")
        print(f"\n❌ 分析过程出现错误: {e}")
        sys.exit(1)


def compare_algorithms(input_file: str) -> None:
    """
    比较两种算法的结果
    
    Args:
        input_file: 输入文件
    """
    print(f"\n🔄 开始比较FIFO与差额计算法...")
    print(f"📂 输入文件: {input_file}")
    
    results = {}
    
    # 运行两种算法
    for algorithm in ["FIFO", "BALANCE_METHOD"]:
        try:
            print(f"\n正在运行 {algorithm} 算法...")
            audit_service = AuditService(algorithm=algorithm)
            result_df = audit_service.analyze_financial_data(input_file)
            
            if result_df is not None:
                # 提取关键指标
                tracker = audit_service.tracker
                results[algorithm] = {
                    "个人余额": tracker.个人余额,
                    "公司余额": tracker.公司余额,
                    "累计挪用": tracker.累计挪用金额,
                    "累计垫付": tracker.累计垫付金额,
                    "已归还公司本金": tracker.累计由资金池回归公司余额本金,
                    "已归还个人本金": tracker.累计由资金池回归个人余额本金,
                    "个人利润": tracker.总计个人应分配利润,
                    "公司利润": tracker.总计公司应分配利润,
                    "资金缺口": (tracker.累计挪用金额 - tracker.累计由资金池回归公司余额本金 - tracker.累计垫付金额)
                }
                print(f"✅ {algorithm} 算法完成")
            else:
                print(f"❌ {algorithm} 算法失败")
                return
                
        except Exception as e:
            print(f"❌ {algorithm} 算法出错: {e}")
            return
    
    # 显示比较结果
    print("\n" + "="*80)
    print("📊 算法对比结果")
    print("="*80)
    
    print(f"{'指标':<20} {'FIFO算法':<20} {'差额计算法':<20} {'差异':<15}")
    print("-" * 80)
    
    for metric in results["FIFO"].keys():
        fifo_val = results["FIFO"][metric]
        balance_val = results["BALANCE_METHOD"][metric]
        diff = balance_val - fifo_val
        
        print(f"{metric:<20} {fifo_val:<20,.2f} {balance_val:<20,.2f} {diff:<15,.2f}")
    
    print("\n📋 对比说明:")
    print("1. FIFO算法：按先进先出原则分配资金来源")
    print("2. 差额计算法：个人余额优先扣除，简化计算逻辑")
    print("3. 差异：正数表示差额计算法数值更大，负数表示更小")
    
    # 保存对比报告
    try:
        import pandas as pd
        comparison_df = pd.DataFrame(results).T
        comparison_file = "算法对比结果.xlsx"
        comparison_df.to_excel(comparison_file)
        print(f"\n💾 对比结果已保存至: {comparison_file}")
    except Exception as e:
        print(f"⚠️ 保存对比结果失败: {e}")


def interactive_mode():
    """交互模式 - 用户选择算法"""
    print("\n" + "="*60)
    print("🏦 FIFO资金追踪审计系统 v2.0")
    print("="*60)
    
    # 显示算法选项
    algorithms = TrackerFactory.get_algorithms_info()
    print("\n可选算法:")
    for i, (algo, desc) in enumerate(algorithms.items(), 1):
        print(f"  {i}. {algo}: {desc}")
    
    # 用户选择
    while True:
        try:
            choice = input(f"\n请选择算法 (1-{len(algorithms)}) 或输入 'q' 退出: ").strip()
            
            if choice.lower() == 'q':
                print("👋 退出系统")
                return
            
            choice_idx = int(choice) - 1
            if 0 <= choice_idx < len(algorithms):
                algorithm = list(algorithms.keys())[choice_idx]
                break
            else:
                print("❌ 无效选择，请重试")
        except ValueError:
            print("❌ 请输入数字或 'q'")
    
    # 文件选择
    input_file = input(f"\n请输入Excel文件路径 (默认: {Config.DEFAULT_INPUT_FILE}): ").strip()
    if not input_file:
        input_file = Config.DEFAULT_INPUT_FILE
    
    # 运行分析
    run_single_analysis(algorithm, input_file)


if __name__ == "__main__":
    # 如果没有命令行参数，启动交互模式
    if len(sys.argv) == 1:
        interactive_mode()
    else:
        main()