#!/bin/bash
# 高性能测试数据生成器包装脚本
# 使用预编译的 gen-test-data 工具
# 测试数据生成在项目根目录的 test_data 目录

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
GEN_TOOL="$SCRIPT_DIR/gen-test-data/target/release/gen-test-data"

if [ ! -f "$GEN_TOOL" ]; then
    echo "正在编译测试数据生成器..."
    cd "$SCRIPT_DIR/gen-test-data"
    cargo build --release
    GEN_TOOL="$SCRIPT_DIR/gen-test-data/target/release/gen-test-data"
fi

# 默认输出到项目根目录的 test_data
exec "$GEN_TOOL" --output "$PROJECT_ROOT/test_data" "$@"
