#!/bin/bash
# mfind 功能测试脚本
# 执行各种搜索场景的测试验证

set -e

MFIND="./target/release/mfind"
TEST_DATA="./test_data"
PASS=0
FAIL=0

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "  mfind 功能测试"
echo "=========================================="
echo ""

# 检查 mfind 是否已编译
if [ ! -f "$MFIND" ]; then
    echo -e "${YELLOW}Release 版本未找到，尝试构建...${NC}"
    cargo build --release 2>&1 | tail -3
fi

if [ ! -f "$MFIND" ]; then
    echo -e "${RED}错误：mfind 未编译${NC}"
    exit 1
fi

# 测试函数
run_test() {
    local name="$1"
    local cmd="$2"
    local expected_min="$3"
    local expected_max="$4"

    echo -n "测试：$name ... "

    # 执行命令并计时
    local start_time=$(date +%s%N)
    local result=$(eval "$cmd" 2>/dev/null | grep -v "^→" | grep -v "^✓" | grep -v "^ℹ" | wc -l | tr -d ' ')
    local end_time=$(date +%s%N)
    local elapsed=$(( (end_time - start_time) / 1000000 )) # ms

    # 检查结果
    if [ -n "$expected_min" ] && [ "$result" -ge "$expected_min" ]; then
        if [ -n "$expected_max" ] && [ "$result" -le "$expected_max" ]; then
            echo -e "${GREEN}✓ 通过${NC} (找到 $result 个文件，${elapsed}ms)"
            ((PASS++))
        elif [ -z "$expected_max" ]; then
            echo -e "${GREEN}✓ 通过${NC} (找到 $result 个文件，${elapsed}ms)"
            ((PASS++))
        else
            echo -e "${RED}✗ 失败${NC} (找到 $result 个，期望 <= $expected_max)"
            ((FAIL++))
        fi
    elif [ -n "$expected_min" ]; then
        echo -e "${RED}✗ 失败${NC} (找到 $result 个，期望 >= $expected_min)"
        ((FAIL++))
    else
        echo -e "${GREEN}✓ 通过${NC} (${elapsed}ms)"
        ((PASS++))
    fi
}

# 测试前生成统计信息
echo ""
echo "测试数据集:"
echo "  路径：$TEST_DATA"
echo "  总文件数：$(find "$TEST_DATA" -type f | wc -l | tr -d ' ')"
echo "  总目录数：$(find "$TEST_DATA" -type d | wc -l | tr -d ' ')"
echo "  总大小：$(du -sh "$TEST_DATA" | cut -f1)"
echo ""

echo "=========================================="
echo "  功能测试"
echo "=========================================="
echo ""

# 1. 前缀搜索测试
echo -e "${BLUE}[1] 前缀搜索测试${NC}"
run_test "前缀 'report' (reports 目录)" "$MFIND search 'report' -p $TEST_DATA" 50
run_test "前缀 'app' (应用日志)" "$MFIND search 'app' -p $TEST_DATA" 100
run_test "前缀 'module' (模块文件)" "$MFIND search 'module' -p $TEST_DATA" 50

echo ""

# 2. 通配符搜索测试
echo -e "${BLUE}[2] 通配符搜索测试${NC}"
run_test "通配符 '*.rs' (Rust 文件)" "$MFIND search '*.rs' -p $TEST_DATA" 100
run_test "通配符 '*.pdf' (PDF 文件)" "$MFIND search '*.pdf' -p $TEST_DATA" 100
run_test "通配符 '*.log' (日志文件)" "$MFIND search '*.log' -p $TEST_DATA" 150
run_test "通配符 '*.toml' (TOML 配置)" "$MFIND search '*.toml' -p $TEST_DATA" 20
run_test "通配符 'Cargo.*'" "$MFIND search 'Cargo.*' -p $TEST_DATA" 5

echo ""

# 3. 扩展名过滤测试
echo -e "${BLUE}[3] 扩展名过滤测试${NC}"
run_test "扩展名 '-e rs'" "$MFIND search -e rs -p $TEST_DATA" 100
run_test "扩展名 '-e py'" "$MFIND search -e py -p $TEST_DATA" 50
run_test "扩展名 '-e js'" "$MFIND search -e js -p $TEST_DATA" 50
run_test "扩展名 '-e pdf'" "$MFIND search -e pdf -p $TEST_DATA" 100

echo ""

# 4. 正则表达式搜索测试
echo -e "${BLUE}[4] 正则表达式搜索测试${NC}"
run_test "正则 '.*\.rs$'" "$MFIND search -r '.*\\.rs$' -p $TEST_DATA" 100
run_test "正则 '.*\\.pdf$'" "$MFIND search -r '.*\\.pdf$' -p $TEST_DATA" 100
run_test "正则 '.*log.*'" "$MFIND search -r '.*log.*' -p $TEST_DATA" 150

echo ""

# 5. 路径过滤测试
echo -e "${BLUE}[5] 路径过滤测试${NC}"
run_test "限定路径 'code/rust'" "$MFIND search '*.rs' -p $TEST_DATA/code/rust" 100
run_test "限定路径 'documents'" "$MFIND search '*.pdf' -p $TEST_DATA/documents" 100
run_test "限定路径 'logs/app'" "$MFIND search '*.log' -p $TEST_DATA/logs/app" 100

echo ""

# 6. 输出格式测试
echo -e "${BLUE}[6] 输出格式测试${NC}"
run_test "JSON 输出" "$MFIND search '*.rs' -p $TEST_DATA -o json" 1
run_test "列表输出" "$MFIND search '*.rs' -p $TEST_DATA -o list" 100

echo ""

# 7. 隐藏文件测试
echo -e "${BLUE}[7] 隐藏文件测试${NC}"
run_test "包含隐藏文件 (默认)" "$MFIND search '.hidden' -p $TEST_DATA" 1
run_test "显式包含隐藏文件" "$MFIND search '.hidden' -p $TEST_DATA --hidden" 1

echo ""

# 8. 限制结果数量测试
echo -e "${BLUE}[8] 限制结果数量测试${NC}"
run_test "限制 10 个结果" "$MFIND search '*' -p $TEST_DATA -n 10" 10 10
run_test "限制 50 个结果" "$MFIND search '*' -p $TEST_DATA -n 50" 50 50

echo ""

# 9. 深度嵌套测试
echo -e "${BLUE}[9] 深度嵌套测试${NC}"
run_test "搜索嵌套目录" "$MFIND search 'deep_file' -p $TEST_DATA" 1
run_test "搜索嵌套层级" "$MFIND search 'level_' -p $TEST_DATA" 5

echo ""

# 10. 性能测试
echo ""
echo "=========================================="
echo "  性能测试"
echo "=========================================="
echo ""

# 全量搜索性能
echo -e "${YELLOW}全量搜索性能测试:${NC}"
echo "扫描全部测试数据..."

# 预热
$MFIND search '*' -p $TEST_DATA > /dev/null 2>&1

# 正式测试
local start=$(date +%s%N)
$MFIND search '*' -p $TEST_DATA > /dev/null 2>&1
local end=$(date +%s%N)
local total_ms=$(( (end - start) / 1000000 ))

echo "  全量搜索时间：${total_ms}ms"
echo "  索引文件数：$(find $TEST_DATA -type f | wc -l | tr -d ' ')"

# 特定模式性能
start=$(date +%s%N)
$MFIND search '*.rs' -p $TEST_DATA > /dev/null 2>&1
end=$(date +%s%N)
local rs_ms=$(( (end - start) / 1000000 ))
echo "  搜索 *.rs 时间：${rs_ms}ms"

start=$(date +%s%N)
$MFIND search '*.pdf' -p $TEST_DATA > /dev/null 2>&1
end=$(date +%s%N)
local pdf_ms=$(( (end - start) / 1000000 ))
echo "  搜索 *.pdf 时间：${pdf_ms}ms"

echo ""

# 总结
echo "=========================================="
echo "  测试总结"
echo "=========================================="
echo ""
echo -e "  ${GREEN}通过：$PASS${NC}"
echo -e "  ${RED}失败：$FAIL${NC}"
echo "  总计：$((PASS + FAIL))"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}所有测试通过!${NC}"
    exit 0
else
    echo -e "${RED}部分测试失败${NC}"
    exit 1
fi
