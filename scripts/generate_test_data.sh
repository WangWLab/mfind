#!/bin/bash
# 测试数据生成脚本
# 用于生成各种类型的测试文件，支持性能测试和功能验证

set -e

TEST_DATA_DIR="${1:-./test_data}"
FILE_COUNT="${2:-1000}"
DEEP_NESTING="${3:-5}"

echo "→ 生成测试数据到：$TEST_DATA_DIR"
echo "  文件数量：$FILE_COUNT"
echo "  最大嵌套深度：$DEEP_NESTING"
echo ""

# 创建基础目录结构
create_base_structure() {
    echo "✓ 创建基础目录结构..."

    mkdir -p "$TEST_DATA_DIR"/{documents,images,code,rust_projects,logs,archives,mixed}

    # 文档目录
    mkdir -p "$TEST_DATA_DIR/documents/reports/2024"
    mkdir -p "$TEST_DATA_DIR/documents/reports/2025"
    mkdir -p "$TEST_DATA_DIR/documents/pdfs"
    mkdir -p "$TEST_DATA_DIR/documents/word"
    mkdir -p "$TEST_DATA_DIR/documents/spreadsheets"

    # 代码目录
    mkdir -p "$TEST_DATA_DIR/code/rust"
    mkdir -p "$TEST_DATA_DIR/code/python"
    mkdir -p "$TEST_DATA_DIR/code/javascript"
    mkdir -p "$TEST_DATA_DIR/code/go"

    # Rust 项目模拟
    mkdir -p "$TEST_DATA_DIR/rust_projects/project_a/src"
    mkdir -p "$TEST_DATA_DIR/rust_projects/project_b/src/bin"
    mkdir -p "$TEST_DATA_DIR/rust_projects/project_c/examples"

    # 日志目录
    mkdir -p "$TEST_DATA_DIR/logs/app"
    mkdir -p "$TEST_DATA_DIR/logs/system"
    mkdir -p "$TEST_DATA_DIR/logs/access"

    # 隐藏文件目录
    mkdir -p "$TEST_DATA_DIR/.hidden"
    mkdir -p "$TEST_DATA_DIR/.git"
}

# 生成文档文件
generate_documents() {
    echo "✓ 生成文档文件..."

    # PDF 文件（模拟）
    for i in $(seq 1 50); do
        echo "%PDF-1.4 Mock PDF content $i" > "$TEST_DATA_DIR/documents/pdfs/report_$i.pdf"
        echo "%PDF-1.4 Mock PDF content $i" > "$TEST_DATA_DIR/documents/reports/2024/report_$i.pdf"
    done

    # Word 文件（模拟）
    for i in $(seq 1 30); do
        echo "Mock Word Document $i" > "$TEST_DATA_DIR/documents/word/doc_$i.docx"
    done

    # Excel 文件（模拟）
    for i in $(seq 1 20); do
        echo "Mock Excel Spreadsheet $i" > "$TEST_DATA_DIR/documents/spreadsheets/data_$i.xlsx"
    done

    # 文本文件
    for i in $(seq 1 100); do
        cat > "$TEST_DATA_DIR/documents/notes_$i.txt" << EOF
Note file #$i
Created: $(date)
Content: This is a test note file with some content.
Tags: test, sample, note-$i
EOF
    done
}

# 生成代码文件
generate_code_files() {
    echo "✓ 生成代码文件..."

    # Rust 文件
    for i in $(seq 1 100); do
        cat > "$TEST_DATA_DIR/code/rust/module_$i.rs" << EOF
//! Module $i - Auto-generated for testing

pub fn process_data_$i(input: &str) -> String {
    format!("Processed: {}", input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_$i() {
        assert_eq!(process_data_$i("test"), "Processed: test");
    }
}
EOF
    done

    # Python 文件
    for i in $(seq 1 50); do
        cat > "$TEST_DATA_DIR/code/python/script_$i.py" << EOF
#!/usr/bin/env python3
\"\"\"Script $i - Auto-generated for testing\"\"\"

def process_data_$i(input_str: str) -> str:
    return f"Processed: {input_str}"

if __name__ == "__main__":
    print(process_data_$i("test"))
EOF
    done

    # JavaScript 文件
    for i in $(seq 1 50); do
        cat > "$TEST_DATA_DIR/code/javascript/app_$i.js" << EOF
// App module $i - Auto-generated for testing

export function processData(input) {
    return \`Processed: \${input}\`;
}

export default processData;
EOF
    done

    # Go 文件
    for i in $(seq 1 30); do
        cat > "$TEST_DATA_DIR/code/go/handler_$i.go" << EOF
package main

// Handler $i - Auto-generated for testing

func ProcessData(input string) string {
    return fmt.Sprintf("Processed: %s", input)
}
EOF
    done
}

# 生成 Rust 项目结构
generate_rust_projects() {
    echo "✓ 生成 Rust 项目结构..."

    # Project A
    cat > "$TEST_DATA_DIR/rust_projects/project_a/Cargo.toml" << EOF
[package]
name = "project_a"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF

    for i in $(seq 1 10); do
        cat > "$TEST_DATA_DIR/rust_projects/project_a/src/module_$i.rs" << EOF
pub mod submodule_$i;

pub fn handler_$i() -> &'static str {
    "handler $i"
}
EOF
        mkdir -p "$TEST_DATA_DIR/rust_projects/project_a/src/submodule_$i"
        echo "pub const VALUE: &str = \"value $i\";" > "$TEST_DATA_DIR/rust_projects/project_a/src/submodule_$i/mod.rs"
    done

    # Project B with binary
    cat > "$TEST_DATA_DIR/rust_projects/project_b/Cargo.toml" << EOF
[package]
name = "project_b"
version = "0.2.0"
edition = "2021"

[[bin]]
name = "cli"
path = "src/bin/cli.rs"

[dependencies]
clap = "4"
EOF

    cat > "$TEST_DATA_DIR/rust_projects/project_b/src/main.rs" << EOF
fn main() {
    println!("Hello from project_b");
}
EOF

    cat > "$TEST_DATA_DIR/rust_projects/project_b/src/bin/cli.rs" << EOF
fn main() {
    println!("CLI tool running");
}
EOF

    # Project C with examples
    cat > "$TEST_DATA_DIR/rust_projects/project_c/Cargo.toml" << EOF
[package]
name = "project_c"
version = "0.3.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
EOF

    for i in $(seq 1 5); do
        cat > "$TEST_DATA_DIR/rust_projects/project_c/examples/example_$i.rs" << EOF
#[tokio::main]
async fn main() {
    println!("Example $i running");
}
EOF
    done
}

# 生成日志文件
generate_logs() {
    echo "✓ 生成日志文件..."

    # 应用日志
    for i in $(seq 1 20); do
        for day in $(seq 1 7); do
            cat > "$TEST_DATA_DIR/logs/app/app_2024010${day}_$i.log" << EOF
2024-01-0${day} 10:00:00 INFO  [app$ i] Application started
2024-01-0${day} 10:00:01 DEBUG [app$ i] Loading configuration
2024-01-0${day} 10:00:02 INFO  [app$ i] Configuration loaded successfully
2024-01-0${day} 10:01:00 WARN  [app$ i] High memory usage detected
2024-01-0${day} 10:02:00 ERROR [app$ i] Connection timeout
2024-01-0${day} 10:03:00 INFO  [app$ i] Reconnected successfully
EOF
        done
    done

    # 访问日志
    for i in $(seq 1 10); do
        for day in $(seq 1 30); do
            printf "192.168.1.%s - - [01/Jan/2024:10:00:00 +0000] \"GET /api/resource%s HTTP/1.1\" 200 1234\n" "$i" "$day" >> "$TEST_DATA_DIR/logs/access/access_202401$(printf '%02d' $day).log"
        done
    done
}

# 生成混合类型文件
generate_mixed_files() {
    echo "✓ 生成混合类型文件..."

    # 各种扩展名
    local extensions=("json" "xml" "yaml" "yml" "toml" "ini" "conf" "cfg" "md" "rst" "html" "css" "sql" "csv" "dat")

    for ext in "${extensions[@]}"; do
        for i in $(seq 1 20); do
            echo "Mock $ext content $i" > "$TEST_DATA_DIR/mixed/file_$i.$ext"
        done
    done

    # 无扩展名文件
    for i in $(seq 1 30); do
        echo "No extension file $i" > "$TEST_DATA_DIR/mixed/noext_$i"
    done

    # 隐藏文件
    for i in $(seq 1 10); do
        echo "Hidden content $i" > "$TEST_DATA_DIR/.hidden/.hidden_file_$i"
    done

    # Git 对象模拟
    for i in $(seq 1 5); do
        echo "mock git object $i" > "$TEST_DATA_DIR/.git/object_$i"
    done
    echo "ref: refs/heads/main" > "$TEST_DATA_DIR/.git/HEAD"
}

# 生成深度嵌套目录
generate_deep_nesting() {
    local depth=$1
    local base_dir="$TEST_DATA_DIR/nested"

    echo "✓ 生成深度嵌套目录 (深度：$depth)..."

    local current_path="$base_dir"
    for i in $(seq 1 $depth); do
        current_path="$current_path/level_$i"
        mkdir -p "$current_path"

        # 每层添加一些文件
        echo "Content at level $i" > "$current_path/file_at_level_$i.txt"
        echo "Data $i" > "$current_path/data_$i.dat"
    done

    # 在最深层添加文件
    for i in $(seq 1 10); do
        echo "Deep nested file $i" > "$current_path/deep_file_$i.txt"
    done
}

# 生成大数据文件（用于性能测试）
generate_large_files() {
    echo "✓ 生成大数据文件..."

    # 1MB 文件
    dd if=/dev/zero of="$TEST_DATA_DIR/mixed/large_1mb.bin" bs=1024 count=1024 2>/dev/null

    # 10MB 文件
    dd if=/dev/zero of="$TEST_DATA_DIR/mixed/large_10mb.bin" bs=1024 count=10240 2>/dev/null
}

# 生成 Cargo.lock 等锁文件
generate_lock_files() {
    echo "✓ 生成锁文件..."

    cat > "$TEST_DATA_DIR/rust_projects/project_a/Cargo.lock" << EOF
# This file is automatically @generated by Cargo.
[[package]]
name = "project_a"
version = "0.1.0"
EOF

    cat > "$TEST_DATA_DIR/code/python/requirements.txt" << EOF
requests==2.31.0
flask==3.0.0
pytest==7.4.0
EOF

    cat > "$TEST_DATA_DIR/code/javascript/package.json" << EOF
{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0"
  }
}
EOF

    cat > "$TEST_DATA_DIR/code/javascript/package-lock.json" << EOF
{
  "name": "test-project",
  "version": "1.0.0",
  "lockfileVersion": 3
}
EOF
}

# 主函数
main() {
    echo "=========================================="
    echo "  测试数据生成脚本"
    echo "=========================================="
    echo ""

    create_base_structure
    generate_documents
    generate_code_files
    generate_rust_projects
    generate_logs
    generate_mixed_files
    generate_deep_nesting $DEEP_NESTING
    generate_large_files
    generate_lock_files

    echo ""
    echo "=========================================="
    echo "  生成完成!"
    echo "=========================================="
    echo ""

    # 统计信息
    echo "统计信息:"
    echo "  总文件数：$(find "$TEST_DATA_DIR" -type f | wc -l | tr -d ' ')"
    echo "  总目录数：$(find "$TEST_DATA_DIR" -type d | wc -l | tr -d ' ')"
    echo "  总大小：$(du -sh "$TEST_DATA_DIR" | cut -f1)"
    echo ""

    # 按类型统计
    echo "按扩展名统计:"
    find "$TEST_DATA_DIR" -type f -name "*.*" | sed 's/.*\.//' | sort | uniq -c | sort -rn | head -10
}

main
