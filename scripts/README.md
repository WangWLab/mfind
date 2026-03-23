# CI/CD 配置说明

## GitHub Actions 工作流

### 构建和发布流程

1. **构建触发**
   - Push 到 main 分支
   - 创建新的 Git 标签

2. **构建步骤**
   - Rust 项目构建
   - 运行测试
   - 创建 macOS 应用 Bundle
   - 代码签名（如果有证书）
   - 创建 DMG 安装包
   - 上传到 GitHub Releases

### 环境变量配置

在 GitHub Secrets 中配置以下变量：

```bash
# Apple 代码签名
MACOS_INSTALLER_IDENTITY    # Apple 分发证书
MACOS_INSTALLER_KEYCHAIN_PASSWORD  # Keychain 密码
APPLE_ID                    # Apple ID
APPLE_TEAM_ID               # Apple Team ID
```

### 本地构建

```bash
# 1. 构建 Release 版本
./scripts/build-release.sh

# 2. 代码签名
./scripts/sign-macos.sh

# 3. 测试应用
open crates/mfind-gui/target/release/bundle/macos/mfind.app
```

## 手动发布步骤

1. 更新版本号
2. 运行构建脚本
3. 运行签名脚本
4. 创建 GitHub Release
5. 上传构建产物
