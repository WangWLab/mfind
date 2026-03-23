#!/bin/bash
# mfind 发布构建脚本
# 使用方法：./scripts/build-release.sh

set -e

echo "=== mfind 发布构建脚本 ==="

# 清理之前的构建
echo "🧹 清理之前的构建..."
cargo clean -p mfind-gui

# 构建 Release 版本
echo "🔨 构建 Release 版本..."
cargo build --release -p mfind-gui

if [ $? -eq 0 ]; then
    echo "✅ 构建成功"
else
    echo "❌ 构建失败"
    exit 1
fi

# 显示构建产物位置
echo ""
echo "=== 构建产物 ==="
echo "应用位置：crates/mfind-gui/target/release/mfind"
echo "Bundle 位置：crates/mfind-gui/target/release/bundle/macos/"
echo ""

# 列出构建产物
if [ -d "crates/mfind-gui/target/release/bundle/macos" ]; then
    echo "📦 Bundle 内容:"
    ls -la crates/mfind-gui/target/release/bundle/macos/
fi

echo ""
echo "=== 构建完成 ==="
echo ""
echo "下一步:"
echo "1. 运行代码签名：./scripts/sign-macos.sh"
echo "2. 测试应用功能"
echo "3. 创建 GitHub Release"
