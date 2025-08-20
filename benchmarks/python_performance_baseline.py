"""
Python性能基准测试脚本
为Rust迁移提供准确的性能对比基准

功能:
1. 测量FIFO和差额计算法的处理时间
2. 监控内存使用量变化
3. 生成输出结果哈希值用于一致性验证
4. 支持多轮测试和统计分析
5. 生成详细的基准报告
"""

import os
import sys
import time
import hashlib
import psutil
import json
import pandas as pd
from datetime import datetime
from typing import Dict, List, Tuple, Optional
import statistics
import tracemalloc
import gc

# 添加src目录到路径
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(__file__)), 'src'))

try:
    from services.audit_service import AuditService
    from core.factories.tracker_factory import TrackerFactory
except ImportError as e:
    print(f"❌ 导入失败: {e}")
    print("请确保在项目根目录下运行此脚本")
    sys.exit(1)

class PerformanceBenchmark:
    """Python性能基准测试器"""
    
    def __init__(self):
        self.test_data_dir = "rust-backend/test_data"
        self.output_dir = "benchmarks/results"
        self.algorithms = ["FIFO", "BALANCE_METHOD"]
        
        # 测试数据集配置
        self.test_datasets = {
            "minimal": {
                "file": "test_data_minimal.xlsx",
                "description": "最小数据集(50行) - 基础功能测试",
                "expected_rows": 50
            },
            "standard": {
                "file": "test_data_standard.xlsx", 
                "description": "标准数据集(1000行) - 常规性能基准",
                "expected_rows": 1000
            },
            "investment": {
                "file": "test_data_investment.xlsx",
                "description": "投资产品数据集 - 复杂业务逻辑测试", 
                "expected_rows": 500
            },
            "complex": {
                "file": "test_data_complex.xlsx",
                "description": "复杂数据集(10000行) - 大数据量性能测试",
                "expected_rows": 10000
            }
        }
        
        # 确保输出目录存在
        os.makedirs(self.output_dir, exist_ok=True)
        
        print("🚀 Python性能基准测试器初始化完成")
    
    def run_full_benchmark(self, rounds: int = 3) -> Dict:
        """运行完整的基准测试"""
        print(f"\n📊 开始完整基准测试 (共{rounds}轮)")
        
        results = {
            "timestamp": datetime.now().isoformat(),
            "system_info": self._get_system_info(),
            "test_rounds": rounds,
            "algorithms": {},
            "summary": {}
        }
        
        for algorithm in self.algorithms:
            print(f"\n🔄 测试算法: {algorithm}")
            results["algorithms"][algorithm] = self._benchmark_algorithm(algorithm, rounds)
        
        # 生成总结
        results["summary"] = self._generate_summary(results["algorithms"])
        
        # 保存结果
        self._save_results(results)
        
        # 显示报告
        self._display_report(results)
        
        return results
    
    def _benchmark_algorithm(self, algorithm: str, rounds: int) -> Dict:
        """对单个算法进行基准测试"""
        algorithm_results = {
            "algorithm": algorithm,
            "datasets": {},
            "overall_stats": {}
        }
        
        for dataset_name, dataset_info in self.test_datasets.items():
            print(f"  📁 测试数据集: {dataset_name}")
            
            dataset_results = []
            file_path = os.path.join(self.test_data_dir, dataset_info["file"])
            
            if not os.path.exists(file_path):
                print(f"    ⚠️ 文件不存在: {file_path}")
                continue
            
            for round_num in range(rounds):
                print(f"    🔄 第{round_num + 1}轮测试...")
                try:
                    result = self._run_single_test(algorithm, file_path, dataset_info)
                    result["round"] = round_num + 1
                    dataset_results.append(result)
                    
                    # 显示实时结果
                    print(f"      ⏱️ 处理时间: {result['processing_time']:.3f}s")
                    print(f"      💾 峰值内存: {result['peak_memory_mb']:.1f}MB")
                    print(f"      📊 处理行数: {result['processed_rows']:,}")
                    
                except Exception as e:
                    print(f"    ❌ 测试失败: {e}")
                    continue
            
            if dataset_results:
                algorithm_results["datasets"][dataset_name] = {
                    "info": dataset_info,
                    "results": dataset_results,
                    "stats": self._calculate_stats(dataset_results)
                }
        
        return algorithm_results
    
    def _run_single_test(self, algorithm: str, file_path: str, dataset_info: Dict) -> Dict:
        """运行单次测试"""
        # 清理内存
        gc.collect()
        
        # 开始内存追踪
        tracemalloc.start()
        process = psutil.Process()
        start_memory = process.memory_info().rss / 1024 / 1024  # MB
        
        # 记录开始时间
        start_time = time.time()
        
        try:
            # 执行算法
            audit_service = AuditService(algorithm=algorithm)
            result_df = audit_service.analyze_financial_data(file_path, suppress_output=True)
            
            # 记录结束时间
            end_time = time.time()
            processing_time = end_time - start_time
            
            # 计算内存使用
            current_memory = process.memory_info().rss / 1024 / 1024  # MB
            memory_usage = current_memory - start_memory
            
            # 获取内存追踪信息
            current, peak = tracemalloc.get_traced_memory()
            tracemalloc.stop()
            peak_memory_mb = peak / 1024 / 1024
            
            # 生成结果哈希
            result_hash = self._generate_result_hash(result_df, audit_service.tracker)
            
            # 获取追踪器状态
            tracker_state = audit_service.tracker.获取状态摘要() if hasattr(audit_service.tracker, '获取状态摘要') else {}
            
            return {
                "processing_time": processing_time,
                "memory_usage_mb": memory_usage,
                "peak_memory_mb": peak_memory_mb,
                "processed_rows": len(result_df) if result_df is not None else 0,
                "result_hash": result_hash,
                "tracker_state": {
                    "个人余额": getattr(audit_service.tracker, '个人余额', 0),
                    "公司余额": getattr(audit_service.tracker, '公司余额', 0),
                    "累计挪用金额": getattr(audit_service.tracker, '累计挪用金额', 0),
                    "累计垫付金额": getattr(audit_service.tracker, '累计垫付金额', 0)
                },
                "success": True
            }
            
        except Exception as e:
            tracemalloc.stop()
            return {
                "processing_time": 0,
                "memory_usage_mb": 0,
                "peak_memory_mb": 0,
                "processed_rows": 0,
                "result_hash": "",
                "tracker_state": {},
                "success": False,
                "error": str(e)
            }
    
    def _generate_result_hash(self, result_df: pd.DataFrame, tracker) -> str:
        """生成结果哈希值用于一致性验证"""
        if result_df is None:
            return ""
        
        # 选择关键字段生成哈希
        key_columns = ['个人资金占比', '公司资金占比', '行为性质', '累计挪用', '累计垫付', '余额']
        available_columns = [col for col in key_columns if col in result_df.columns]
        
        if not available_columns:
            return ""
        
        # 提取关键数据
        hash_data = []
        
        # DataFrame关键字段
        for col in available_columns:
            if result_df[col].dtype in ['float64', 'int64']:
                # 数值字段保留2位小数
                hash_data.extend([f"{x:.2f}" for x in result_df[col].fillna(0)])
            else:
                # 字符串字段
                hash_data.extend([str(x) for x in result_df[col].fillna("")])
        
        # 追踪器状态
        if hasattr(tracker, '个人余额'):
            hash_data.append(f"{tracker.个人余额:.2f}")
        if hasattr(tracker, '公司余额'):
            hash_data.append(f"{tracker.公司余额:.2f}")
        if hasattr(tracker, '累计挪用金额'):
            hash_data.append(f"{tracker.累计挪用金额:.2f}")
        if hasattr(tracker, '累计垫付金额'):
            hash_data.append(f"{tracker.累计垫付金额:.2f}")
        
        # 生成MD5哈希
        hash_string = "|".join(hash_data)
        return hashlib.md5(hash_string.encode('utf-8')).hexdigest()[:16]  # 取前16位
    
    def _calculate_stats(self, results: List[Dict]) -> Dict:
        """计算统计数据"""
        if not results or not any(r.get("success", False) for r in results):
            return {"error": "无有效结果"}
        
        valid_results = [r for r in results if r.get("success", False)]
        
        times = [r["processing_time"] for r in valid_results]
        memories = [r["peak_memory_mb"] for r in valid_results]
        
        return {
            "count": len(valid_results),
            "processing_time": {
                "mean": statistics.mean(times),
                "median": statistics.median(times),
                "min": min(times),
                "max": max(times),
                "stdev": statistics.stdev(times) if len(times) > 1 else 0
            },
            "memory_usage": {
                "mean": statistics.mean(memories),
                "median": statistics.median(memories),
                "min": min(memories),
                "max": max(memories),
                "stdev": statistics.stdev(memories) if len(memories) > 1 else 0
            },
            "result_hashes": list(set(r["result_hash"] for r in valid_results)),
            "consistency": len(set(r["result_hash"] for r in valid_results)) == 1
        }
    
    def _generate_summary(self, algorithm_results: Dict) -> Dict:
        """生成测试总结"""
        summary = {
            "algorithms_tested": len(algorithm_results),
            "algorithm_comparison": {}
        }
        
        # 算法对比
        if "FIFO" in algorithm_results and "BALANCE_METHOD" in algorithm_results:
            for dataset in self.test_datasets.keys():
                if (dataset in algorithm_results["FIFO"].get("datasets", {}) and 
                    dataset in algorithm_results["BALANCE_METHOD"].get("datasets", {})):
                    
                    fifo_stats = algorithm_results["FIFO"]["datasets"][dataset]["stats"]
                    balance_stats = algorithm_results["BALANCE_METHOD"]["datasets"][dataset]["stats"]
                    
                    if "error" not in fifo_stats and "error" not in balance_stats:
                        fifo_time = fifo_stats["processing_time"]["mean"]
                        balance_time = balance_stats["processing_time"]["mean"]
                        
                        summary["algorithm_comparison"][dataset] = {
                            "fifo_time_avg": fifo_time,
                            "balance_time_avg": balance_time,
                            "speed_ratio": balance_time / fifo_time if fifo_time > 0 else 0,
                            "faster_algorithm": "FIFO" if fifo_time < balance_time else "BALANCE_METHOD"
                        }
        
        return summary
    
    def _get_system_info(self) -> Dict:
        """获取系统信息"""
        return {
            "python_version": sys.version,
            "cpu_count": psutil.cpu_count(),
            "memory_total_gb": psutil.virtual_memory().total / (1024**3),
            "platform": sys.platform,
            "timestamp": datetime.now().isoformat()
        }
    
    def _save_results(self, results: Dict):
        """保存测试结果"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"python_baseline_{timestamp}.json"
        filepath = os.path.join(self.output_dir, filename)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(results, f, ensure_ascii=False, indent=2)
        
        # 同时保存一个latest版本
        latest_filepath = os.path.join(self.output_dir, "python_baseline_latest.json")
        with open(latest_filepath, 'w', encoding='utf-8') as f:
            json.dump(results, f, ensure_ascii=False, indent=2)
        
        print(f"\n💾 基准测试结果已保存:")
        print(f"  📄 详细结果: {filepath}")
        print(f"  🔗 最新结果: {latest_filepath}")
    
    def _display_report(self, results: Dict):
        """显示测试报告"""
        print(f"\n" + "="*80)
        print(f"📊 PYTHON性能基准测试报告")
        print(f"="*80)
        print(f"🕐 测试时间: {results['timestamp']}")
        print(f"🔄 测试轮数: {results['test_rounds']}")
        print(f"💻 系统信息: Python {results['system_info']['python_version'].split()[0]}, {results['system_info']['cpu_count']}核CPU")
        
        for algorithm, algo_results in results["algorithms"].items():
            print(f"\n🧮 算法: {algorithm}")
            print(f"-" * 60)
            
            for dataset_name, dataset_results in algo_results.get("datasets", {}).items():
                stats = dataset_results["stats"]
                if "error" in stats:
                    print(f"  ❌ {dataset_name}: {stats['error']}")
                    continue
                
                info = dataset_results["info"]
                print(f"  📁 {dataset_name} ({info['description']})")
                print(f"    ⏱️  平均时间: {stats['processing_time']['mean']:.3f}s (±{stats['processing_time']['stdev']:.3f}s)")
                print(f"    💾 平均内存: {stats['memory_usage']['mean']:.1f}MB (±{stats['memory_usage']['stdev']:.1f}MB)")
                print(f"    🏃‍♂️ 处理速度: {info['expected_rows']/stats['processing_time']['mean']:,.0f} 行/秒")
                print(f"    🔒 结果一致性: {'✅' if stats['consistency'] else '❌'}")
        
        # 算法对比
        if "algorithm_comparison" in results["summary"] and results["summary"]["algorithm_comparison"]:
            print(f"\n⚖️ 算法性能对比")
            print(f"-" * 60)
            for dataset, comparison in results["summary"]["algorithm_comparison"].items():
                print(f"  📊 {dataset}:")
                print(f"    FIFO: {comparison['fifo_time_avg']:.3f}s")
                print(f"    差额法: {comparison['balance_time_avg']:.3f}s") 
                print(f"    更快算法: {comparison['faster_algorithm']} (比率: {comparison['speed_ratio']:.2f}x)")
        
        print(f"\n🎯 基准数据已建立，可用于Rust版本性能对比！")

def main():
    """主程序入口"""
    print("🚀 Python性能基准测试工具")
    print("为Rust迁移建立准确的性能对比基准")
    
    # 检查测试数据
    test_data_dir = "rust-backend/test_data"
    if not os.path.exists(test_data_dir):
        print(f"❌ 测试数据目录不存在: {test_data_dir}")
        print("请先运行 analyze_data_format.py 创建测试数据集")
        return
    
    benchmark = PerformanceBenchmark()
    
    try:
        # 运行基准测试
        results = benchmark.run_full_benchmark(rounds=3)
        
        print(f"\n✅ Python基准测试完成！")
        print(f"📈 后续可使用这些基准数据验证Rust版本性能提升")
        
    except KeyboardInterrupt:
        print(f"\n⚠️ 用户中断测试")
    except Exception as e:
        print(f"❌ 测试过程出错: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()
