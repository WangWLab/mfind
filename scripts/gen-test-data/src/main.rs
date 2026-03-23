use anyhow::Result;
use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;

#[derive(Parser, Debug)]
#[command(author, version, about = "高性能测试数据生成器")]
struct Args {
    /// 输出目录
    #[arg(short, long, default_value = "./test_data")]
    output: PathBuf,

    /// 文件总数
    #[arg(short, long, default_value = "1000")]
    count: usize,

    /// 并发度 (同时创建的文件数)
    #[arg(short, long, default_value = "200")]
    concurrency: usize,

    /// 最大嵌套深度
    #[arg(short, long, default_value = "5")]
    depth: usize,

    /// 是否生成集成测试所需的文件
    #[arg(long, default_value = "false")]
    for_integration: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let start = std::time::Instant::now();

    println!("→ 生成测试数据");
    println!("  输出目录：{}", args.output.display());
    println!("  文件数量：{}", args.count);
    println!("  并发度：{}", args.concurrency);
    println!("  最大深度：{}", args.depth);
    if args.for_integration {
        println!("  模式：集成测试数据");
    }

    // 创建基础目录
    let dirs = [
        "documents/reports/2024",
        "documents/reports/2025",
        "documents/pdfs",
        "documents/word",
        "code/rust",
        "code/python",
        "code/javascript",
        "code/go",
        "rust_projects/proj_a/src",
        "rust_projects/proj_b/src/bin",
        "logs/app",
        "logs/system",
        "mixed",
        ".hidden",
    ];

    for d in &dirs {
        let path = args.output.join(d);
        if let Err(e) = fs::create_dir_all(&path).await {
            eprintln!("警告：创建目录失败 {:?}: {}", path, e);
        }
    }

    // 创建深度嵌套目录
    for i in 1..=args.depth {
        let nested = args.output.join("nested").join(format!("level_{}", i));
        fs::create_dir_all(&nested).await.expect("创建嵌套目录失败");
    }

    // 并发创建文件
    let semaphore = Arc::new(Semaphore::new(args.concurrency));
    let mut handles = Vec::with_capacity(args.count);

    // 动态生成目录列表（包含所有嵌套层级）
    let mut dir_options = Vec::new();
    for i in 1..=args.depth.min(10) {
        dir_options.push(format!("nested/level_{}", i));
    }
    dir_options.extend_from_slice(&[
        "documents/reports/2024".to_string(),
        "documents/reports/2025".to_string(),
        "documents/pdfs".to_string(),
        "documents/word".to_string(),
        "code/rust".to_string(),
        "code/python".to_string(),
        "code/javascript".to_string(),
        "code/go".to_string(),
        "rust_projects/proj_a/src".to_string(),
        "rust_projects/proj_b/src/bin".to_string(),
        "logs/app".to_string(),
        "logs/system".to_string(),
        "mixed".to_string(),
    ]);

    let extensions = [
        "rs", "py", "js", "go", "toml", "json", "yaml", "md",
        "txt", "log", "cfg", "ini", "conf", "xml", "html", "css",
    ];

    // 集成测试模式：确保生成足够的特定类型文件
    if args.for_integration {
        // 生成 150 个 .rs 文件
        for i in 0..150 {
            let sem = semaphore.clone();
            let output = args.output.clone();
            let dirs = dir_options.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                create_file_with_ext(&output, i, "rs", &dirs).await
            }));
        }
        // 生成 150 个 .pdf 文件
        for i in 0..150 {
            let sem = semaphore.clone();
            let output = args.output.clone();
            let dirs = dir_options.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                create_file_with_ext(&output, i, "pdf", &dirs).await
            }));
        }
        // 生成 50 个 .toml 文件
        for i in 0..50 {
            let sem = semaphore.clone();
            let output = args.output.clone();
            let dirs = dir_options.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                create_file_with_ext(&output, i, "toml", &dirs).await
            }));
        }
        // 生成 50 个 Cargo 开头的文件
        for i in 0..50 {
            let sem = semaphore.clone();
            let output = args.output.clone();
            let dirs = dir_options.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                create_cargo_file(&output, i, &dirs).await
            }));
        }
    }

    // 生成剩余的随机文件
    let remaining = if args.for_integration { 400 } else { args.count };
    for i in 0..remaining {
        let sem = semaphore.clone();
        let output = args.output.clone();
        let dirs = dir_options.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            create_file(&output, i, &extensions, &dirs).await
        }));
    }

    // 等待所有任务完成
    let results = futures::future::join_all(handles).await;
    let mut success = 0;
    let mut failed = 0;

    for r in results {
        match r {
            Ok(Ok(())) => success += 1,
            _ => failed += 1,
        }
    }

    let elapsed = start.elapsed();
    println!("\n✓ 完成!");
    println!("  成功：{} 文件", success);
    println!("  失败：{} 文件", failed);
    println!("  耗时：{:.2}s", elapsed.as_secs_f64());
    println!("  速度：{:.0} 文件/秒", success as f64 / elapsed.as_secs_f64());

    Ok(())
}

async fn create_file(output: &PathBuf, idx: usize, extensions: &[&str], dirs: &[String]) -> Result<()> {
    let mut rng = StdRng::seed_from_u64(idx as u64);

    // 随机选择目录
    let dir = &dirs[rng.gen_range(0..dirs.len())];

    // 随机扩展名
    let ext = extensions[rng.gen_range(0..extensions.len())];

    // 生成文件名
    let filename = format!("file_{:06}.{}", idx, ext);
    let path = output.join(dir).join(&filename);

    // 生成文件内容
    let content = match ext {
        "rs" => generate_rust_content(idx),
        "py" => generate_python_content(idx),
        "js" => generate_js_content(idx),
        "go" => generate_go_content(idx),
        "toml" => format!("# Config {}\nname = \"test_{}\"\n", idx, idx),
        "json" => format!("{{\"id\": {}, \"name\": \"test_{}\"}}", idx, idx),
        "yaml" => format!("id: {}\nname: test_{}\n", idx, idx),
        "md" => format!("# Note {}\n\nContent for note {}.\n", idx, idx),
        "txt" => format!("Text file {}\nCreated for testing.\n", idx),
        "log" => format!("2024-01-01 10:00:00 INFO  [app] Log entry {}\n", idx),
        "cfg" => format!("[section]\nkey = value_{}\n", idx),
        "ini" => format!("[settings]\nitem = {}\n", idx),
        "conf" => format!("# Config\noption={}\n", idx),
        "xml" => format!("<root><item id=\"{}\">Test</item></root>\n", idx),
        "html" => format!("<!DOCTYPE html><html><body>Page {}</body></html>\n", idx),
        "css" => format!("/* Stylesheet {} */\nbody {{ margin: 0; }}\n", idx),
        _ => format!("File {}\n", idx),
    };

    fs::write(&path, content).await.unwrap_or_else(|e| {
        eprintln!("写入失败 {:?}: {}", path, e);
    });
    Ok(())
}

fn generate_rust_content(idx: usize) -> String {
    format!(
        r#"//! Module {0}

pub fn process_{0}(input: &str) -> String {{
    format!("Processed: {{}}", input)
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_process_{0}() {{
        assert_eq!(process_{0}("test"), "Processed: test");
    }}
}}
"#,
        idx
    )
}

fn generate_python_content(idx: usize) -> String {
    format!(
        r#"#!/usr/bin/env python3
"""Module {0}"""

def process_{0}(s: str) -> str:
    return f"Processed: {{s}}"

if __name__ == "__main__":
    print(process_{0}("test"))
"#,
        idx
    )
}

fn generate_js_content(idx: usize) -> String {
    format!(
        r#"// Module {0}
export function process_{0}(input) {{
    return `Processed: ${{input}}`;
}}

export default process_{0};
"#,
        idx
    )
}

fn generate_go_content(idx: usize) -> String {
    format!(
        r#"package main

// Handler {0}
fn Process_{0}(input string) string {{
    return "Processed: " + input
}}
"#,
        idx
    )
}

async fn create_file_with_ext(output: &PathBuf, idx: usize, ext: &str, dirs: &[String]) -> Result<()> {
    let mut rng = StdRng::seed_from_u64(idx as u64);
    let dir = &dirs[rng.gen_range(0..dirs.len())];
    let filename = format!("file_{:06}.{}", idx, ext);
    let path = output.join(dir).join(&filename);

    let content = match ext {
        "rs" => generate_rust_content(idx),
        "pdf" => format!("%PDF-1.4 Mock PDF content {}\n", idx),
        "toml" => format!("# Cargo manifest {}\n[package]\nname = \"test_{}\"\nversion = \"0.1.{}\"\n", idx, idx, idx),
        _ => format!("File {}\n", idx),
    };

    fs::write(&path, content).await.unwrap_or_else(|e| {
        eprintln!("写入失败 {:?}: {}", path, e);
    });
    Ok(())
}

async fn create_cargo_file(output: &PathBuf, idx: usize, dirs: &[String]) -> Result<()> {
    let mut rng = StdRng::seed_from_u64(idx as u64);
    let dir = &dirs[rng.gen_range(0..dirs.len())];
    let filename = format!("Cargo_{:06}.toml", idx);
    let path = output.join(dir).join(&filename);

    let content = format!(
        r#"# Cargo manifest file {}
[package]
name = "cargo_test_{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        idx, idx
    );

    fs::write(&path, content).await.unwrap_or_else(|e| {
        eprintln!("写入失败 {:?}: {}", path, e);
    });
    Ok(())
}
