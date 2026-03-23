#!/bin/bash
# 生成 100 万文件用于性能测试
# 使用硬链接和简短内容来减少磁盘占用

set -e

BASE_DIR="${1:-./test_data/large_scale}"
TARGET_COUNT="${2:-1000000}"

echo "=========================================="
echo "  百万级文件生成脚本"
echo "=========================================="
echo ""
echo "目标目录：$BASE_DIR"
echo "目标文件数：$TARGET_COUNT"
echo ""

# 检查磁盘空间
AVAILABLE_KB=$(df -k "$BASE_DIR" 2>/dev/null | tail -1 | awk '{print $4}')
echo "可用磁盘空间：$((AVAILABLE_KB / 1024 / 1024)) GB"
echo ""

# 创建基础目录
echo "✓ 创建目录结构..."
mkdir -p "$BASE_DIR"

# 计算目录和文件分布
# 1000 个目录，每个目录约 1000 个文件 = 100 万文件
NUM_DIRS=1000
FILES_PER_DIR=$((TARGET_COUNT / NUM_DIRS))

echo "目录数量：$NUM_DIRS"
echo "每目录文件数：$FILES_PER_DIR"
echo ""

# 批量创建目录
echo "✓ 创建 $NUM_DIRS 个目录..."
START_TIME=$(date +%s)

for i in $(seq 1 $NUM_DIRS); do
    mkdir -p "$BASE_DIR/dir_$(printf '%04d' $i)"
done

END_TIME=$(date +%s)
echo "  目录创建完成：$((END_TIME - START_TIME)) 秒"
echo ""

# 批量创建文件
echo "✓ 创建 $TARGET_COUNT 个文件..."
START_TIME=$(date +%s)

# 文件扩展名列表
EXTENSIONS=(".rs" ".py" ".js" ".go" ".java" ".txt" ".md" ".json" ".toml" ".yaml")
NUM_EXTS=${#EXTENSIONS[@]}

for dir_idx in $(seq 1 $NUM_DIRS); do
    DIR="$BASE_DIR/dir_$(printf '%04d' $dir_idx)"

    for file_idx in $(seq 1 $FILES_PER_DIR); do
        # 循环使用扩展名
        EXT_IDX=$(( (dir_idx + file_idx) % NUM_EXTS ))
        EXT="${EXTENSIONS[$EXT_IDX]}"

        # 创建简短内容的文件
        echo "f$dir_idx_$file_idx" > "$DIR/file_$(printf '%06d' $file_idx)$EXT"
    done

    # 每 100 个目录显示进度
    if [ $((dir_idx % 100)) -eq 0 ]; then
        CURRENT_COUNT=$((dir_idx * FILES_PER_DIR))
        PERCENT=$((dir_idx * 100 / NUM_DIRS))
        echo "  进度：$CURRENT_COUNT / $TARGET_COUNT ($PERCENT%)"
    fi
done

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
echo ""
echo "  文件创建完成：$ELAPSED 秒"
echo "  平均速度：$((TARGET_COUNT / ELAPSED)) 文件/秒"
echo ""

# 创建一些特殊文件
echo "✓ 创建特殊文件..."

# Cargo.lock 类型的大文件
for i in $(seq 1 10); do
    DIR="$BASE_DIR/dir_$(printf '%04d' $i)"
    for j in $(seq 1 100); do
        echo "# Lock file entry $j" >> "$DIR/Cargo.lock"
    done
done

# 深度嵌套目录
echo "✓ 创建深度嵌套目录..."
DEEP_DIR="$BASE_DIR/nested"
for i in $(seq 1 20); do
    DEEP_DIR="$DEEP_DIR/level_$i"
    mkdir -p "$DEEP_DIR"
    for j in $(seq 1 50); do
        echo "deep_$j" > "$DEEP_DIR/deep_file_$j.txt"
    done
done

# 统计信息
echo ""
echo "=========================================="
echo "  生成完成!"
echo "=========================================="
echo ""

TOTAL_FILES=$(find "$BASE_DIR" -type f | wc -l | tr -d ' ')
TOTAL_DIRS=$(find "$BASE_DIR" -type d | wc -l | tr -d ' ')
TOTAL_SIZE=$(du -sh "$BASE_DIR" | cut -f1)

echo "统计信息:"
echo "  总文件数：$TOTAL_FILES"
echo "  总目录数：$TOTAL_DIRS"
echo "  总大小：$TOTAL_SIZE"
echo ""

# 按扩展名统计
echo "按扩展名统计 (Top 10):"
find "$BASE_DIR" -type f -name "*.*" | sed 's/.*\.//' | sort | uniq -c | sort -rn | head -10
echo ""

echo "✓ 完成!"
