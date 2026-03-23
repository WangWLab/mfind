//! Search benchmark tests

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mfind_core::index::{FSTIndex, IndexEngine, IndexConfig};
use mfind_core::index::engine::IndexEngineTrait;
use mfind_core::query::{Query, QueryParser};

/// Benchmark FST build with different scales
fn bench_fst_build(c: &mut Criterion) {
    // 100 files
    let mut paths_100: Vec<Vec<u8>> = (0..100)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_100.sort();
    c.bench_function("fst_build_100", |b| {
        b.iter(|| FSTIndex::build(black_box(&paths_100)).unwrap());
    });

    // 1000 files
    let mut paths_1k: Vec<Vec<u8>> = (0..1_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_1k.sort();
    c.bench_function("fst_build_1k", |b| {
        b.iter(|| FSTIndex::build(black_box(&paths_1k)).unwrap());
    });

    // 10k files
    let mut paths_10k: Vec<Vec<u8>> = (0..10_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_10k.sort();
    c.bench_function("fst_build_10k", |b| {
        b.iter(|| FSTIndex::build(black_box(&paths_10k)).unwrap());
    });

    // 100k files
    let mut paths_100k: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_100k.sort();
    c.bench_function("fst_build_100k", |b| {
        b.iter(|| FSTIndex::build(black_box(&paths_100k)).unwrap());
    });
}

fn bench_prefix_search(c: &mut Criterion) {
    // 100 files
    let mut paths_100: Vec<Vec<u8>> = (0..100)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_100.sort();
    let index_100 = FSTIndex::build(&paths_100).unwrap();
    c.bench_function("prefix_search_100", |b| {
        b.iter(|| index_100.prefix_search(black_box("/path/to/file_05")).unwrap());
    });

    // 1k files
    let mut paths_1k: Vec<Vec<u8>> = (0..1_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_1k.sort();
    let index_1k = FSTIndex::build(&paths_1k).unwrap();
    c.bench_function("prefix_search_1k", |b| {
        b.iter(|| index_1k.prefix_search(black_box("/path/to/file_05")).unwrap());
    });

    // 10k files
    let mut paths_10k: Vec<Vec<u8>> = (0..10_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_10k.sort();
    let index_10k = FSTIndex::build(&paths_10k).unwrap();
    c.bench_function("prefix_search_10k", |b| {
        b.iter(|| index_10k.prefix_search(black_box("/path/to/file_05")).unwrap());
    });

    // 100k files
    let mut paths_100k: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_100k.sort();
    let index_100k = FSTIndex::build(&paths_100k).unwrap();
    c.bench_function("prefix_search_100k", |b| {
        b.iter(|| index_100k.prefix_search(black_box("/path/to/file_05")).unwrap());
    });
}

fn bench_regex_search(c: &mut Criterion) {
    let regex = regex::Regex::new(".*\\.txt$").unwrap();

    // 100 files
    let mut paths_100: Vec<Vec<u8>> = (0..100)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_100.sort();
    let index_100 = FSTIndex::build(&paths_100).unwrap();
    c.bench_function("regex_search_100", |b| {
        b.iter(|| index_100.regex_search(black_box(&regex)).unwrap());
    });

    // 1k files
    let mut paths_1k: Vec<Vec<u8>> = (0..1_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_1k.sort();
    let index_1k = FSTIndex::build(&paths_1k).unwrap();
    c.bench_function("regex_search_1k", |b| {
        b.iter(|| index_1k.regex_search(black_box(&regex)).unwrap());
    });

    // 10k files
    let mut paths_10k: Vec<Vec<u8>> = (0..10_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_10k.sort();
    let index_10k = FSTIndex::build(&paths_10k).unwrap();
    c.bench_function("regex_search_10k", |b| {
        b.iter(|| index_10k.regex_search(black_box(&regex)).unwrap());
    });

    // 100k files
    let mut paths_100k: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths_100k.sort();
    let index_100k = FSTIndex::build(&paths_100k).unwrap();
    c.bench_function("regex_search_100k", |b| {
        b.iter(|| index_100k.regex_search(black_box(&regex)).unwrap());
    });
}

fn bench_query_parse(c: &mut Criterion) {
    c.bench_function("query_parse_simple", |b| {
        b.iter(|| QueryParser::parse(black_box("test_pattern")).unwrap());
    });

    c.bench_function("query_parse_regex", |b| {
        b.iter(|| QueryParser::parse(black_box("regex:.*\\.txt$")).unwrap());
    });

    c.bench_function("query_parse_wildcard", |b| {
        b.iter(|| QueryParser::parse(black_box("*.txt")).unwrap());
    });
}

/// Benchmark index engine build with realistic data
fn bench_index_engine_build(c: &mut Criterion) {
    let roots = vec![std::path::PathBuf::from("./test_data")];

    c.bench_function("index_engine_build_test_data", |b| {
        b.iter(|| {
            let config = IndexConfig::default();
            let mut engine = IndexEngine::new(config).unwrap();
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(engine.build(&roots));
        })
    });
}

/// Benchmark search with various patterns
fn bench_search_patterns(c: &mut Criterion) {
    // Build an index first
    let config = IndexConfig::default();
    let mut engine = IndexEngine::new(config).unwrap();
    let roots = vec![std::path::PathBuf::from("./test_data")];

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(engine.build(&roots));

    c.bench_function("search_prefix_cargo", |b| {
        let query = Query::prefix("Cargo".to_string());
        b.iter(|| engine.search(black_box(&query)).unwrap())
    });

    c.bench_function("search_wildcard_rs", |b| {
        let query = Query::wildcard("*.rs".to_string());
        b.iter(|| engine.search(black_box(&query)).unwrap())
    });

    c.bench_function("search_regex_rs", |b| {
        let query = Query::regex(".*\\.rs$".to_string()).unwrap();
        b.iter(|| engine.search(black_box(&query)).unwrap())
    });
}

criterion_group!(
    benches,
    bench_fst_build,
    bench_prefix_search,
    bench_regex_search,
    bench_query_parse,
    bench_index_engine_build,
    bench_search_patterns,
);

criterion_main!(benches);
