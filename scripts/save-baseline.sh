#!/bin/bash
# 保存性能基线
# 用法：./scripts/save-baseline.sh [基线名称]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCHMARK_DIR="$PROJECT_ROOT/.benchmarks"

NAME="${1:-latest}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BENCHMARK_DIR"

echo "保存性能基线：$NAME"
echo "时间：$TIMESTAMP"
echo ""

cd "$PROJECT_ROOT"
cargo bench --bench search_bench -- --save-baseline "$BENCHMARK_DIR/$NAME"

echo ""
echo "基线已保存：$BENCHMARK_DIR/$NAME"
