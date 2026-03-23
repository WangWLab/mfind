#!/bin/bash
# mfind macOS 代码签名和发布脚本
# 使用方法：./scripts/sign-macos.sh

set -e

echo "=== mfind macOS 代码签名脚本 ==="

# 配置变量
APP_NAME="mfind"
APP_BUNDLE="crates/mfind-gui/target/release/bundle/macos/${APP_NAME}.app"
IDENTITY_KEYCHAIN="${KEYCHAIN:-$HOME/Library/Keychains/login.keychain-db}"
INSTALLER_IDENTITY="${MACOS_INSTALLER_IDENTITY:-}"

# 检查是否设置了签名身份
if [ -z "$MACOS_INSTALLER_IDENTITY" ]; then
    echo "⚠️  未设置 MACOS_INSTALLER_IDENTITY 环境变量"
    echo "   将使用 ad-hoc 签名（仅用于本地测试）"
    SIGN_IDENTITY="--force --sign -"
else
    echo "✅ 使用签名身份：$MACOS_INSTALLER_IDENTITY"
    SIGN_IDENTITY="--force --sign \"$MACOS_INSTALLER_IDENTITY\""
fi

# 检查应用是否存在
if [ ! -d "$APP_BUNDLE" ]; then
    echo "❌ 应用不存在：$APP_BUNDLE"
    echo "   请先运行：cargo build --release -p mfind-gui"
    exit 1
fi

echo "📦 签名应用：$APP_BUNDLE"

# 签名应用
codesign $SIGN_IDENTITY \
    --deep \
    --strict \
    --preserve-metadata=entitlements,requirements,flags,runtime \
    "$APP_BUNDLE"

if [ $? -eq 0 ]; then
    echo "✅ 应用签名完成"

    # 验证签名
    echo "🔍 验证签名..."
    codesign --verify --verbose=4 "$APP_BUNDLE"

    if [ $? -eq 0 ]; then
        echo "✅ 签名验证通过"
    else
        echo "❌ 签名验证失败"
        exit 1
    fi
else
    echo "❌ 应用签名失败"
    exit 1
fi

echo ""
echo "=== 签名完成 ==="
echo "应用位置：$APP_BUNDLE"
echo ""
echo "下一步:"
echo "1. 测试运行应用"
echo "2. 创建 DMG 安装包"
echo "3. 提交到 GitHub Releases"
