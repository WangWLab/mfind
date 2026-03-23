#!/bin/bash
# mfind 性能测试自动化脚本
# 用于日常性能测试和回归检测

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCHMARK_DIR="$PROJECT_ROOT/.benchmarks"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 性能阈值（毫秒）
THRESHOLD_FST_BUILD_100K=20
THRESHOLD_PREFIX_SEARCH_100K=10
THRESHOLD_REGEX_SEARCH_100K=15

usage() {
    cat << EOF
mfind 性能测试工具

用法：$0 [选项] [命令]

命令:
    run         运行完整基准测试（默认）
    quick       快速测试（仅 100/1k 规模）
    compare     与之前基线对比
    save        保存当前基线
    report      生成性能报告

选项:
    -s, --scale <scale>   测试规模：100|1k|10k|100k (默认：all)
    -t, --target <name>   特定测试目标（如：fst_build）
    -o, --output <file>   输出结果到文件
    -v, --verbose         详细输出
    -h, --help            显示帮助

示例:
    $0 run                        # 运行完整测试
    $0 quick                      # 快速测试
    $0 run -s 10k                 # 运行 10k 规模测试
    $0 run -t fst_build           # 只测试 FST 构建
    $0 compare                    # 与基线对比
    $0 save baseline_v1           # 保存基线

性能阈值（100k 规模）:
    FST 构建：    < ${THRESHOLD_FST_BUILD_100K}ms
    前缀搜索：    < ${THRESHOLD_PREFIX_SEARCH_100K}ms
    正则搜索：    < ${THRESHOLD_REGEX_SEARCH_100K}ms
EOF
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

check_test_data() {
    if [ ! -d "$PROJECT_ROOT/test_data" ]; then
        log_info "测试数据不存在，正在生成..."
        ./scripts/gen-test-data.sh --count 1000
    fi
}

run_benchmarks() {
    local scale="$1"
    local target="$2"
    local verbose="$3"

    local args=""

    # 构建测试过滤参数
    if [ -n "$target" ]; then
        args="$args $target"
    fi

    # 详细输出
    if [ "$verbose" = "true" ]; then
        args="$args --verbose"
    fi

    log_info "运行基准测试..."
    echo ""
    echo "========================================"
    echo "  mfind 性能基准测试"
    echo "  时间：$(date '+%Y-%m-%d %H:%M:%S')"
    echo "========================================"
    echo ""

    check_test_data

    # 运行基准测试
    if [ "$scale" = "quick" ]; then
        log_info "快速测试模式（100/1k 规模）"
        cargo bench --bench search_bench -- 'fst_build_100$'
        cargo bench --bench search_bench -- 'fst_build_1k$'
        cargo bench --bench search_bench -- 'prefix_search_100$'
        cargo bench --bench search_bench -- 'prefix_search_1k$'
    else
        cargo bench --bench search_bench $args
    fi

    echo ""
    log_success "基准测试完成!"
}

compare_baseline() {
    if [ ! -d "$BENCHMARK_DIR" ]; then
        log_error "没有找到基线数据"
        log_info "请先运行：$0 save"
        exit 1
    fi

    log_info "与基线对比..."
    cargo bench --bench search_bench -- --baseline latest
}

save_baseline() {
    local name="${1:-latest}"

    mkdir -p "$BENCHMARK_DIR"

    log_info "保存基线：$name"
    cargo bench --bench search_bench -- --save-baseline "$BENCHMARK_DIR/$name"

    log_success "基线已保存：$BENCHMARK_DIR/$name"
}

check_thresholds() {
    log_info "检查性能阈值..."

    # 这里可以解析 benchmark 输出
    # 简化版本：直接运行并目视检查

    echo ""
    echo "请检查以下关键指标:"
    echo "  - fst_build_100k:    < ${THRESHOLD_FST_BUILD_100K}ms"
    echo "  - prefix_search_100k: < ${THRESHOLD_PREFIX_SEARCH_100K}ms"
    echo "  - regex_search_100k:  < ${THRESHOLD_REGEX_SEARCH_100K}ms"
    echo ""
}

generate_report() {
    local output="${1:-$BENCHMARK_DIR/report_$TIMESTAMP.md}"

    mkdir -p "$(dirname "$output")"

    log_info "生成性能报告：$output"

    cat > "$output" << EOF
# mfind 性能测试报告

**生成时间:** $(date '+%Y-%m-%d %H:%M:%S')
**测试环境:** $(uname -a)

## 测试结果

运行 \`cargo bench\` 后查看详细输出。

## 关键指标

| 测试项 | 目标值 | 实测值 | 状态 |
|--------|--------|--------|------|
| FST 构建 (100k) | < 20ms | - | - |
| 前缀搜索 (100k) | < 10ms | - | - |
| 正则搜索 (100k) | < 15ms | - | - |

## 详细数据

请查看 \`cargo bench\` 输出或 criterion 报告。
EOF

    log_success "报告已生成：$output"
}

# 主程序
main() {
    cd "$PROJECT_ROOT"

    local command="${1:-run}"
    shift || true

    local scale=""
    local target=""
    local output=""
    local verbose="false"

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -s|--scale)
                scale="$2"
                shift 2
                ;;
            -t|--target)
                target="$2"
                shift 2
                ;;
            -o|--output)
                output="$2"
                shift 2
                ;;
            -v|--verbose)
                verbose="true"
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                if [ -z "$command" ]; then
                    command="$1"
                fi
                shift
                ;;
        esac
    done

    case "$command" in
        run)
            run_benchmarks "$scale" "$target" "$verbose"
            ;;
        quick)
            run_benchmarks "quick" "" "false"
            ;;
        compare)
            compare_baseline
            ;;
        save)
            save_baseline "$1"
            ;;
        report)
            generate_report "$output"
            ;;
        check)
            check_thresholds
            ;;
        -h|--help|help)
            usage
            exit 0
            ;;
        *)
            log_error "未知命令：$command"
            usage
            exit 1
            ;;
    esac
}

main "$@"
