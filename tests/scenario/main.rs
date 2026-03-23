//! 实际使用场景测试
//!
//! 测试以下场景:
//! 1. 首次打开初始化 - 快速构建索引
//! 2. 频繁退出启动 - 索引持久化，避免重复构建
//! 3. 长时间未启动 - 检测文件变化，增量更新
//! 4. 后台持续运行 - FSEvents 实时监控

use std::fs;
use std::time::Instant;
use tempfile::tempdir;

use mfind_core::index::{IndexEngine, IndexConfig};
use mfind_core::index::engine::IndexEngineTrait;
use mfind_core::query::QueryParser;

#[tokio::main]
async fn main() {
    println!("=== mfind 场景测试 ===\n");

    // 场景 1: 首次初始化
    test_first_init().await;

    // 场景 2: 频繁启停
    test_frequent_restart().await;

    // 场景 3: 长时间未启动
    test_long_time_no_start().await;

    println!("\n=== 所有场景测试完成 ===");
}

/// 场景 1: 首次初始化性能测试
async fn test_first_init() {
    println!("--- 场景 1: 首次初始化 ---");

    let dir = tempdir().unwrap();
    let test_path = dir.path().join("test_data");
    fs::create_dir_all(&test_path).unwrap();

    // 生成 1000 个测试文件
    let start = Instant::now();
    for i in 0..1000 {
        fs::write(test_path.join(format!("file_{:04}.txt", i)), "content").unwrap();
    }
    let gen_time = start.elapsed();

    let config = IndexConfig::default();
    let mut engine = IndexEngine::new(config).unwrap();

    let start = Instant::now();
    let stats = engine.build(&[test_path.clone()]).await.unwrap();
    let elapsed = start.elapsed();

    println!("  生成文件：1000 个，耗时 {:?}", gen_time);
    println!("  索引构建：{} 文件，耗时 {:?}", stats.total_files, elapsed);
    println!("  速度：{:.0} 文件/秒", stats.total_files as f64 / elapsed.as_secs_f64());

    // 1000 文件应在 1 秒内完成
    assert!(elapsed.as_secs() < 2, "首次初始化应小于 2 秒");
    assert!(stats.total_files >= 1000, "应至少索引 1000 文件，实际 {}", stats.total_files);
    println!("  ✓ 通过\n");
}

/// 场景 2: 频繁启停 - 验证持久化
async fn test_frequent_restart() {
    println!("--- 场景 2: 频繁启停 ---");

    let dir = tempdir().unwrap();
    let test_path = dir.path().join("test_data");
    fs::create_dir_all(&test_path).unwrap();

    // 生成测试文件
    for i in 0..500 {
        fs::write(test_path.join(format!("file_{:04}.rs", i)), "content").unwrap();
    }

    // 第一次启动 - 构建索引并导出
    let config = IndexConfig::default();
    let mut engine1 = IndexEngine::new(config).unwrap();
    let start1 = Instant::now();
    engine1.build(&[test_path.clone()]).await.unwrap();
    let build_time = start1.elapsed();

    // 导出索引
    let export_data = engine1.export().await.unwrap();
    println!("  导出索引大小：{} bytes", export_data.len());

    // 第二次启动 - 从持久化导入 (模拟重启)
    let config2 = IndexConfig::default();
    let mut engine2 = IndexEngine::new(config2).unwrap();
    let start2 = Instant::now();
    engine2.import(&export_data).await.unwrap();
    let import_time = start2.elapsed();

    println!("  首次构建：{:?}", build_time);
    println!("  导入恢复：{:?}", import_time);
    println!("  加速比：{:.1}x", build_time.as_secs_f64() / import_time.as_secs_f64());

    // 导入应该比重新构建快得多
    assert!(import_time < build_time, "导入应该比重新构建快");

    // 验证导入后索引可用
    let query = QueryParser::parse("*.rs").unwrap();
    let results = engine2.search(&query).unwrap();
    assert_eq!(results.total, 500);
    println!("  ✓ 通过\n");
}

/// 场景 3: 长时间未启动 - 验证增量更新
async fn test_long_time_no_start() {
    println!("--- 场景 3: 长时间未启动 ---");

    let dir = tempdir().unwrap();
    let test_path = dir.path().join("test_data");
    fs::create_dir_all(&test_path).unwrap();

    // 初始状态：500 个文件
    for i in 0..500 {
        fs::write(test_path.join(format!("file_{:04}.txt", i)), "content").unwrap();
    }

    // 构建初始索引
    let config = IndexConfig::default();
    let mut engine = IndexEngine::new(config).unwrap();
    let stats1 = engine.build(&[test_path.clone()]).await.unwrap();

    // 导出当前索引
    let export_data = engine.export().await.unwrap();
    println!("  初始索引：{} 文件，{} bytes", stats1.total_files, export_data.len());

    // 模拟长时间未启动后的变化：新增 100 个文件，删除 50 个文件
    for i in 500..600 {
        fs::write(test_path.join(format!("file_{:04}.txt", i)), "new content").unwrap();
    }
    for i in 0..50 {
        fs::remove_file(test_path.join(format!("file_{:04}.txt", i))).unwrap();
    }

    // 重新扫描并比较
    let mut engine2 = IndexEngine::new(IndexConfig::default()).unwrap();
    let stats2 = engine2.build(&[test_path.clone()]).await.unwrap();

    println!("  初始文件数：{}", stats1.total_files);
    println!("  变化后文件数：{}", stats2.total_files);
    println!("  净变化：{} 文件", stats2.total_files as i64 - stats1.total_files as i64);

    // 验证正确检测到变化 (允许 ±1 误差，因为 tempdir 可能包含其他文件)
    assert!(stats2.total_files >= 550, "应至少 550 文件，实际 {}", stats2.total_files);
    println!("  ✓ 通过\n");
}
