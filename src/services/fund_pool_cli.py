#!/usr/bin/env python3
"""
资金池查询CLI
用于通过命令行查询特定资金池的详细信息
"""

import sys
import json
import argparse
from pathlib import Path

# 添加项目根目录到Python路径
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

from services.time_point_query_service import TimePointQueryService
from utils.logger import audit_logger


def main():
    parser = argparse.ArgumentParser(description='资金池详细信息查询')
    parser.add_argument('--file', required=True, help='数据文件路径')
    parser.add_argument('--row', type=int, required=True, help='查询行号')
    parser.add_argument('--algorithm', required=True, choices=['FIFO', 'BALANCE_METHOD'], help='分析算法')
    parser.add_argument('--pool', required=True, help='资金池名称')
    
    args = parser.parse_args()
    
    try:
        # 创建时点查询服务
        service = TimePointQueryService(
            file_path=args.file, 
            algorithm=args.algorithm
        )
        
        # 首先执行时点查询到指定行，以确保追踪器状态正确
        time_point_result = service.query(args.row, save_to_history=False)
        
        if not time_point_result.get("success", False):
            print(json.dumps({
                "success": False,
                "message": f"时点查询失败: {time_point_result.get('message', '未知错误')}"
            }), file=sys.stdout)
            return
        
        # 查询资金池详情
        fund_pool_result = service.query_fund_pool(args.pool)
        
        # 输出结果
        print(json.dumps(fund_pool_result, ensure_ascii=False, indent=2), file=sys.stdout)
        sys.stdout.flush()
        
    except Exception as e:
        error_result = {
            "success": False,
            "message": f"资金池查询异常: {str(e)}",
            "error_details": str(e)
        }
        print(json.dumps(error_result, ensure_ascii=False), file=sys.stdout)
        audit_logger.error(f"资金池查询CLI异常: {e}")


if __name__ == "__main__":
    main()
