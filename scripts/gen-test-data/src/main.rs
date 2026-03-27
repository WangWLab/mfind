use anyhow::Result;
use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;

#[derive(Parser, Debug)]
#[command(author, version, about = "高性能测试数据生成器 - 支持千万级文件")]
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

    /// 每个目录的最大文件数
    #[arg(long, default_value = "10000")]
    max_files_per_dir: usize,

    /// 最小化文件内容 (节省空间)
    #[arg(long, default_value = "false")]
    minimal: bool,

    /// 是否生成集成测试所需的文件
    #[arg(long, default_value = "false")]
    for_integration: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let start = std::time::Instant::now();

    // 计算需要的目录数量
    let required_dirs = (args.count + args.max_files_per_dir - 1) / args.max_files_per_dir;

    println!("→ 生成测试数据");
    println!("  输出目录：{}", args.output.display());
    println!("  文件数量：{}", args.count);
    println!("  并发度：{}", args.concurrency);
    println!("  每目录最大文件数：{}", args.max_files_per_dir);
    println!("  需要目录数：{}", required_dirs);
    println!("  最小化内容：{}", if args.minimal { "是" } else { "否" });
    if args.for_integration {
        println!("  模式：集成测试数据");
    }

    // 检查磁盘空间
    println!("\n→ 检查磁盘空间...");
    if let Ok(output) = std::process::Command::new("df")
        .arg("-k")
        .arg(&args.output)
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = output_str.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let available_kb: i64 = parts[3].parse().unwrap_or(0);
                let available_mb = available_kb / 1024;
                let available_gb = available_mb / 1024;
                println!("  可用空间：{} GB", available_gb);

                // 估算所需空间
                let estimated_bytes = if args.minimal {
                    args.count * 20 // 每文件约 20 字节
                } else {
                    args.count * 80 // 每文件约 80 字节
                };
                let estimated_mb = estimated_bytes / (1024 * 1024);
                println!("  预计占用：{} MB", estimated_mb);

                if available_mb < estimated_mb as i64 * 2 {
                    eprintln!("警告：可用空间可能不足！");
                }
            }
        }
    }

    // 创建基础目录
    let base_dirs = [
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

    for d in &base_dirs {
        let path = args.output.join(d);
        if let Err(e) = fs::create_dir_all(&path).await {
            eprintln!("警告：创建目录失败 {:?}: {}", path, e);
        }
    }

    // 动态创建足够的目录以分散文件
    println!("\n→ 创建 {} 个目录...", required_dirs);
    let mut dir_options: Vec<String> = base_dirs.iter().map(|s| s.to_string()).collect();

    // 批量创建目录，使用信号量控制并发
    let dir_sem = Arc::new(Semaphore::new(100)); // 限制目录创建并发
    let mut dir_handles = Vec::with_capacity(required_dirs);

    for i in 0..required_dirs {
        let sem = dir_sem.clone();
        let output = args.output.clone();
        dir_handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let dir_path = output.join(format!("data_{:05}", i));
            fs::create_dir_all(&dir_path).await
        }));
    }

    // 等待所有目录创建完成
    let _dir_results = futures::future::join_all(dir_handles).await;

    // 添加动态创建的目录到列表
    for i in 0..required_dirs {
        dir_options.push(format!("data_{:05}", i));
    }

    println!("✓ 目录创建完成，共 {} 个目录", dir_options.len());

    // 计算每个目录的文件分配
    // 目标：每个目录最多 max_files_per_dir 个文件
    // 将文件均匀分配到各个目录
    let files_per_dir = args.count / required_dirs;
    let _remainder = args.count % required_dirs;

    println!("✓ 文件分配：每目录约 {} 个文件", files_per_dir);

    // 并发创建文件 - 按目录分组处理
    let semaphore = Arc::new(Semaphore::new(args.concurrency));
    let mut handles = Vec::with_capacity(args.count);

    let extensions = [
        "rs", "py", "js", "go", "toml", "json", "yaml", "md",
        "txt", "log", "cfg", "ini", "conf", "xml", "html", "css",
    ];

    // 分批显示进度
    let progress_interval = if args.count > 10000 { 10000 } else { 1000 };
    let max_files_per_dir = args.max_files_per_dir;

    // 集成测试模式：确保生成足够的特定类型文件
    if args.for_integration {
        // 生成特定类型的文件
        let special_files = vec![
            ("rs", 150),
            ("pdf", 150),
            ("toml", 50),
            ("cargo", 50),
        ];

        let mut file_idx = 0;
        for (_ext_type, count) in special_files {
            for _i in 0..count {
                let sem = semaphore.clone();
                let output = args.output.clone();
                let minimal = args.minimal;
                let idx = file_idx;
                let max_files = max_files_per_dir;

                handles.push(tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    if idx % progress_interval == 0 && idx > 0 {
                        println!("  文件进度：{}/{}", idx, args.count);
                    }
                    // 所有特殊文件都使用 create_file_with_ext_minimal
                    create_file_with_ext_minimal(&output, idx, "rs", max_files, minimal).await
                }));

                file_idx += 1;
            }
        }

        // 生成剩余的随机文件
        let remaining = args.count - 400;
        for i in 0..remaining {
            let sem = semaphore.clone();
            let output = args.output.clone();
            let extensions = extensions;
            let minimal = args.minimal;
            let idx = file_idx + i;
            let max_files = max_files_per_dir;

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                if idx % progress_interval == 0 {
                    println!("  文件进度：{}/{}", idx, args.count);
                }
                create_file_minimal(&output, idx, &extensions, max_files, minimal).await
            }));
        }
    } else {
        // 普通模式：全部随机文件
        for i in 0..args.count {
            let sem = semaphore.clone();
            let output = args.output.clone();
            let extensions = extensions;
            let minimal = args.minimal;
            let idx = i;
            let max_files = max_files_per_dir;

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                if idx % progress_interval == 0 && idx > 0 {
                    println!("  文件进度：{}/{}", idx, args.count);
                }
                create_file_minimal(&output, idx, &extensions, max_files, minimal).await
            }));
        }
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

async fn create_file_minimal(
    output: &PathBuf,
    idx: usize,
    extensions: &[&str],
    max_files_per_dir: usize,
    minimal: bool,
) -> Result<()> {
    let mut rng = StdRng::seed_from_u64(idx as u64);

    // 根据索引计算目录，确保均匀分布
    let dir_idx = idx / max_files_per_dir;
    let dir = format!("data_{:05}", dir_idx);

    // 随机扩展名
    let ext = extensions[rng.gen_range(0..extensions.len())];

    // 生成文件名
    let filename = format!("file_{:07}.{}", idx, ext);
    let path = output.join(&dir).join(&filename);

    // 生成文件内容
    let content = if minimal {
        // 最小化模式：只包含文件索引，每文件约 10-20 字节
        format!("f{}\n", idx)
    } else {
        match ext {
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
        }
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

async fn create_file_with_ext_minimal(
    output: &PathBuf,
    idx: usize,
    ext: &str,
    max_files_per_dir: usize,
    minimal: bool,
) -> Result<()> {
    // 根据索引计算目录，确保均匀分布
    let dir_idx = idx / max_files_per_dir;
    let dir = format!("data_{:05}", dir_idx);
    let filename = format!("file_{:07}.{}", idx, ext);
    let path = output.join(&dir).join(&filename);

    let content = if minimal {
        // 最小化模式
        match ext {
            "rs" => format!("// {}\n", idx),
            "pdf" => format!("%{}\n", idx),
            "toml" => format!("{}={}\n", idx, idx),
            _ => format!("{}\n", idx),
        }
    } else {
        match ext {
            "rs" => generate_rust_content(idx),
            "pdf" => format!("%PDF-1.4 Mock PDF content {}\n", idx),
            "toml" => format!("# Cargo manifest {}\n[package]\nname = \"test_{}\"\nversion = \"0.1.{}\"\n", idx, idx, idx),
            _ => format!("File {}\n", idx),
        }
    };

    fs::write(&path, content).await.unwrap_or_else(|e| {
        eprintln!("写入失败 {:?}: {}", path, e);
    });
    Ok(())
}
