//! # 数据处理性能基准测试
//! 
//! 测试Excel读写、数据验证、序列化等数据处理操作的性能。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// 模拟Excel数据行
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockExcelRow {
    pub 交易日期: String,
    pub 交易时间: String,
    pub 交易收入金额: Option<f64>,
    pub 交易支出金额: Option<f64>,
    pub 余额: f64,
    pub 资金属性: String,
    pub 备注: Option<String>,
}

/// 生成模拟Excel数据
fn generate_mock_excel_data(rows: usize) -> Vec<MockExcelRow> {
    (0..rows)
        .map(|i| MockExcelRow {
            交易日期: format!("2023-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1),
            交易时间: format!("{:02}:{:02}:{:02}", (i % 24), (i % 60), (i % 60)),
            交易收入金额: if i % 3 == 0 { Some(1000.0 + i as f64) } else { None },
            交易支出金额: if i % 3 == 1 { Some(500.0 + i as f64 * 0.5) } else { None },
            余额: 10000.0 + i as f64 * 10.0,
            资金属性: if i % 2 == 0 { "个人".to_string() } else { "公司".to_string() },
            备注: if i % 10 == 0 { Some(format!("备注{}", i)) } else { None },
        })
        .collect()
}

/// JSON序列化性能测试
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");
    
    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        let test_data = generate_mock_excel_data(*size);
        
        group.bench_with_input(
            BenchmarkId::new("serialize_to_json", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    black_box(serde_json::to_string(data).unwrap())
                });
            },
        );
        
        // 测试反序列化
        let serialized = serde_json::to_string(&test_data).unwrap();
        group.bench_with_input(
            BenchmarkId::new("deserialize_from_json", size),
            &serialized,
            |b, json_str| {
                b.iter(|| {
                    black_box(
                        serde_json::from_str::<Vec<MockExcelRow>>(json_str).unwrap()
                    )
                });
            },
        );
    }
    
    group.finish();
}

/// 数据验证性能测试
fn bench_data_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_validation");
    
    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        let test_data = generate_mock_excel_data(*size);
        
        group.bench_with_input(
            BenchmarkId::new("validate_all_fields", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    black_box(mock_validate_data(data))
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("validate_required_only", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    black_box(mock_validate_required_fields(data))
                });
            },
        );
    }
    
    group.finish();
}

/// 数据转换性能测试
fn bench_data_transformation(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_transformation");
    
    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        let test_data = generate_mock_excel_data(*size);
        
        // 测试单线程转换
        group.bench_with_input(
            BenchmarkId::new("transform_sequential", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    black_box(mock_transform_data_sequential(data))
                });
            },
        );
        
        // 测试并行转换
        group.bench_with_input(
            BenchmarkId::new("transform_parallel", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    black_box(mock_transform_data_parallel(data))
                });
            },
        );
    }
    
    group.finish();
}

/// 哈希表性能测试
fn bench_hashmap_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_operations");
    
    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        let test_data = generate_mock_excel_data(*size);
        
        group.bench_with_input(
            BenchmarkId::new("build_hashmap", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    let mut map = HashMap::with_capacity(data.len());
                    for (i, row) in data.iter().enumerate() {
                        map.insert(i, black_box(&row.资金属性));
                    }
                    black_box(map)
                });
            },
        );
        
        // 构建用于查询测试的HashMap
        let mut lookup_map = HashMap::new();
        for (i, row) in test_data.iter().enumerate() {
            lookup_map.insert(format!("key_{}", i), row);
        }
        
        group.bench_with_input(
            BenchmarkId::new("lookup_operations", size),
            &lookup_map,
            |b, map| {
                b.iter(|| {
                    let mut sum = 0.0;
                    for i in 0..*size {
                        if let Some(row) = map.get(&format!("key_{}", i)) {
                            sum += black_box(row.余额);
                        }
                    }
                    black_box(sum)
                });
            },
        );
    }
    
    group.finish();
}

/// 内存分配模式测试
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // 测试Vec预分配 vs 动态增长
        group.bench_with_input(
            BenchmarkId::new("vec_with_capacity", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = Vec::with_capacity(size);
                    for i in 0..size {
                        vec.push(black_box(i));
                    }
                    black_box(vec)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("vec_without_capacity", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = Vec::new();
                    for i in 0..size {
                        vec.push(black_box(i));
                    }
                    black_box(vec)
                });
            },
        );
        
        // 测试String操作
        group.bench_with_input(
            BenchmarkId::new("string_concatenation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut result = String::with_capacity(size * 10);
                    for i in 0..size {
                        result.push_str(&format!("item_{},", black_box(i)));
                    }
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

/// 字符串处理性能测试
fn bench_string_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_processing");
    
    let test_strings: Vec<String> = (0..10000)
        .map(|i| format!("测试字符串_{}_with_mixed_中文_and_english_{}", i, i * 2))
        .collect();
    
    group.bench_function("string_parsing", |b| {
        b.iter(|| {
            let mut count = 0;
            for s in &test_strings {
                if black_box(s.contains("中文")) {
                    count += 1;
                }
            }
            black_box(count)
        });
    });
    
    group.bench_function("regex_matching", |b| {
        let re = regex::Regex::new(r"\d+").unwrap();
        b.iter(|| {
            let mut matches = 0;
            for s in &test_strings {
                matches += black_box(re.find_iter(s).count());
            }
            black_box(matches)
        });
    });
    
    group.finish();
}

// =============================================================================
// 模拟实现函数
// =============================================================================

/// 模拟数据验证
fn mock_validate_data(data: &[MockExcelRow]) -> usize {
    data.iter()
        .filter(|row| {
            !row.交易日期.is_empty() 
                && !row.交易时间.is_empty()
                && !row.资金属性.is_empty()
                && row.余额.is_finite()
        })
        .count()
}

/// 模拟必需字段验证
fn mock_validate_required_fields(data: &[MockExcelRow]) -> usize {
    data.iter()
        .filter(|row| !row.交易日期.is_empty() && !row.资金属性.is_empty())
        .count()
}

/// 模拟单线程数据转换
fn mock_transform_data_sequential(data: &[MockExcelRow]) -> Vec<f64> {
    data.iter()
        .map(|row| {
            let inflow = row.交易收入金额.unwrap_or(0.0);
            let outflow = row.交易支出金额.unwrap_or(0.0);
            inflow - outflow
        })
        .collect()
}

/// 模拟并行数据转换
fn mock_transform_data_parallel(data: &[MockExcelRow]) -> Vec<f64> {
    use rayon::prelude::*;
    
    data.par_iter()
        .map(|row| {
            let inflow = row.交易收入金额.unwrap_or(0.0);
            let outflow = row.交易支出金额.unwrap_or(0.0);
            inflow - outflow
        })
        .collect()
}

// 配置基准测试组
criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(8))
        .sample_size(50);
    targets = 
        bench_json_serialization,
        bench_data_validation,
        bench_data_transformation,
        bench_hashmap_operations,
        bench_memory_patterns,
        bench_string_processing
);

criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mock_data() {
        let data = generate_mock_excel_data(100);
        assert_eq!(data.len(), 100);
        
        // 检查数据完整性
        for row in &data {
            assert!(!row.交易日期.is_empty());
            assert!(!row.交易时间.is_empty());
            assert!(!row.资金属性.is_empty());
            assert!(row.余额.is_finite());
        }
    }

    #[test]
    fn test_json_serialization() {
        let data = generate_mock_excel_data(10);
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: Vec<MockExcelRow> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(data.len(), deserialized.len());
        assert_eq!(data[0].交易日期, deserialized[0].交易日期);
    }

    #[test]
    fn test_validation_functions() {
        let data = generate_mock_excel_data(10);
        
        let all_valid = mock_validate_data(&data);
        let required_valid = mock_validate_required_fields(&data);
        
        assert!(all_valid <= data.len());
        assert!(required_valid <= data.len());
        assert!(required_valid >= all_valid);
    }

    #[test]
    fn test_transformation_functions() {
        let data = generate_mock_excel_data(100);
        
        let sequential_result = mock_transform_data_sequential(&data);
        let parallel_result = mock_transform_data_parallel(&data);
        
        assert_eq!(sequential_result.len(), data.len());
        assert_eq!(parallel_result.len(), data.len());
        
        // 结果应该相同
        for (seq, par) in sequential_result.iter().zip(parallel_result.iter()) {
            assert!((seq - par).abs() < f64::EPSILON);
        }
    }
}
