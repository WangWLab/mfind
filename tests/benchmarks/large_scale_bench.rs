//! 百万级文件性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use mfind_core::index::{FSTIndex, IndexEngine, IndexConfig};
use mfind_core::index::engine::IndexEngineTrait;
use mfind_core::query::{Query, QueryParser};
use std::path::PathBuf;

/// Benchmark FST build with 1 million entries
fn bench_fst_build_1m(c: &mut Criterion) {
    let mut paths: Vec<Vec<u8>> = (0..1_000_000)
        .map(|i| format!("/path/to/file_{:07}.txt", i).into_bytes())
        .collect();
    paths.sort();

    let mut group = c.benchmark_group("fst_build");
    group.throughput(Throughput::Elements(1_000_000));
    group.bench_function("fst_build_1m", |b| {
        b.iter(|| FSTIndex::build(black_box(&paths)).unwrap());
    });
    group.finish();
}

/// Benchmark prefix search with 1 million entries
fn bench_prefix_search_1m(c: &mut Criterion) {
    let mut paths: Vec<Vec<u8>> = (0..1_000_000)
        .map(|i| format!("/path/to/file_{:07}.txt", i).into_bytes())
        .collect();
    paths.sort();

    let index = FSTIndex::build(&paths).unwrap();

    let mut group = c.benchmark_group("prefix_search");
    group.throughput(Throughput::Elements(1_000_000));
    group.bench_function("prefix_search_1m_pattern", |b| {
        b.iter(|| index.prefix_search(black_box("/path/to/file_05")).unwrap());
    });
    group.finish();
}

/// Benchmark regex search with 1 million entries
fn bench_regex_search_1m(c: &mut Criterion) {
    let mut paths: Vec<Vec<u8>> = (0..1_000_000)
        .map(|i| format!("/path/to/file_{:07}.txt", i).into_bytes())
        .collect();
    paths.sort();

    let index = FSTIndex::build(&paths).unwrap();
    let regex = regex::Regex::new(".*\\.txt$").unwrap();

    let mut group = c.benchmark_group("regex_search");
    group.throughput(Throughput::Elements(1_000_000));
    group.bench_function("regex_search_1m", |b| {
        b.iter(|| index.regex_search(black_box(&regex)).unwrap());
    });
    group.finish();
}

/// Benchmark IndexEngine build with real 1M files
fn bench_index_engine_build_1m(c: &mut Criterion) {
    let roots = vec![PathBuf::from("./test_data/large_scale")];

    let mut group = c.benchmark_group("index_engine_build");
    group.throughput(Throughput::Elements(1_000_000));
    group.sample_size(10); // Reduce samples for long-running test
    group.bench_function("index_engine_build_1m_files", |b| {
        b.iter(|| {
            let config = IndexConfig::default();
            let mut engine = IndexEngine::new(config).unwrap();
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(engine.build(&roots));
        })
    });
    group.finish();
}

/// Benchmark search on 1M file index
fn bench_search_1m_files(c: &mut Criterion) {
    let config = IndexConfig::default();
    let mut engine = IndexEngine::new(config).unwrap();
    let roots = vec![PathBuf::from("./test_data/large_scale")];

    println!("\n  Building index for 1M files...");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let build_result = rt.block_on(engine.build(&roots));
    println!("  Index build result: {:?}", build_result.is_ok());

    let mut group = c.benchmark_group("search_1m_files");
    group.sample_size(10);

    // Prefix search
    group.bench_function("search_prefix_cargo", |b| {
        let query = Query::prefix("Cargo".to_string());
        b.iter(|| engine.search(black_box(&query)).unwrap())
    });

    // Wildcard search - *.rs files
    group.bench_function("search_wildcard_rs", |b| {
        let query = Query::wildcard("*.rs".to_string());
        b.iter(|| engine.search(black_box(&query)).unwrap())
    });

    // Wildcard search - *.txt files
    group.bench_function("search_wildcard_txt", |b| {
        let query = Query::wildcard("*.txt".to_string());
        b.iter(|| engine.search(black_box(&query)).unwrap())
    });

    // Regex search
    group.bench_function("search_regex_rs", |b| {
        let query = Query::regex(".*\\.rs$".to_string()).unwrap();
        b.iter(|| engine.search(black_box(&query)).unwrap())
    });

    // Prefix search for specific pattern
    group.bench_function("search_prefix_file_500", |b| {
        let query = Query::prefix("/test_data/large_scale/dir_0500/file_".to_string());
        b.iter(|| engine.search(black_box(&query)).unwrap())
    });

    group.finish();
}

/// Benchmark query parsing
fn bench_query_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_parsing");

    group.bench_function("parse_simple_pattern", |b| {
        b.iter(|| QueryParser::parse(black_box("test_pattern")).unwrap())
    });

    group.bench_function("parse_wildcard", |b| {
        b.iter(|| QueryParser::parse(black_box("*.rs")).unwrap())
    });

    group.bench_function("parse_regex", |b| {
        b.iter(|| QueryParser::parse(black_box(".*\\.txt$")).unwrap())
    });

    group.bench_function("parse_prefix", |b| {
        b.iter(|| QueryParser::parse(black_box("prefix:Cargo")).unwrap())
    });

    group.finish();
}

/// Memory footprint estimation
fn bench_memory_footprint(c: &mut Criterion) {
    let mut paths: Vec<Vec<u8>> = (0..1_000_000)
        .map(|i| format!("/path/to/file_{:07}.txt", i).into_bytes())
        .collect();
    paths.sort();

    let mut group = c.benchmark_group("memory_footprint");
    group.sample_size(10);
    group.bench_function("fst_size_1m", |b| {
        b.iter(|| {
            let index = FSTIndex::build(&paths).unwrap();
            let data = index.to_bytes().unwrap();
            data.len() // Size in bytes
        })
    });
    group.finish();
}

criterion_group!(
    benches_1m,
    bench_fst_build_1m,
    bench_prefix_search_1m,
    bench_regex_search_1m,
    bench_query_parsing,
    bench_memory_footprint,
);

criterion_group!(
    benches_real_1m,
    bench_index_engine_build_1m,
    bench_search_1m_files,
);

criterion_main!(benches_1m, benches_real_1m);
