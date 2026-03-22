//! Search benchmark tests

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mfind_core::index::{FSTIndex, IndexEngine, IndexConfig};
use mfind_core::query::{Query, QueryParser};

fn bench_fst_build(c: &mut Criterion) {
    let paths: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{}.txt", i).into_bytes())
        .collect();

    c.bench_function("fst_build_100k", |b| {
        b.iter(|| FSTIndex::build(black_box(&paths)).unwrap());
    });
}

fn bench_prefix_search(c: &mut Criterion) {
    let paths: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{}.txt", i).into_bytes())
        .collect();

    let index = FSTIndex::build(&paths).unwrap();

    c.bench_function("prefix_search_100k", |b| {
        b.iter(|| index.prefix_search(black_box("/path/to/file_5")).unwrap());
    });
}

fn bench_regex_search(c: &mut Criterion) {
    let paths: Vec<Vec<u8>> = (0..100_000)
        .map(|i| format!("/path/to/file_{}.txt", i).into_bytes())
        .collect();

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

criterion_group!(
    benches,
    bench_fst_build,
    bench_prefix_search,
    bench_regex_search,
    bench_query_parse,
);

criterion_main!(benches);
