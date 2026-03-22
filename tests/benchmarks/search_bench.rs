//! Search benchmark tests

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mfind_core::index::{FSTIndex, IndexEngine, IndexConfig};
use mfind_core::index::engine::IndexEngineTrait;
use mfind_core::query::{Query, QueryParser};

fn bench_fst_build(c: &mut Criterion) {
    let mut paths: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths.sort();

    c.bench_function("fst_build_100k", |b| {
        b.iter(|| FSTIndex::build(black_box(&paths)).unwrap());
    });
}

fn bench_prefix_search(c: &mut Criterion) {
    let mut paths: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths.sort();

    let index = FSTIndex::build(&paths).unwrap();

    c.bench_function("prefix_search_100k", |b| {
        b.iter(|| index.prefix_search(black_box("/path/to/file_05")).unwrap());
    });
}

fn bench_regex_search(c: &mut Criterion) {
    let mut paths: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{:06}.txt", i).into_bytes())
        .collect();
    paths.sort();

    let index = FSTIndex::build(&paths).unwrap();
    let regex = regex::Regex::new(".*\\.txt$").unwrap();

    c.bench_function("regex_search_100k", |b| {
        b.iter(|| index.regex_search(black_box(&regex)).unwrap());
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
