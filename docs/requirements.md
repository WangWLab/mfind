# macOS 高性能文件搜索工具 - 需求文档与技术实现方案 (v2.0)

> **文档状态：** 已根据市场调研更新
> **更新日期：** 2026-03-23

---

## 后台持续运行场景解决方案

### 问题描述

当前 FSEvents 实现使用轮询方式，存在以下问题：
- 事件延迟高 (100-1000ms)
- 可能遗漏快速连续的文件变化
- 资源浪费 (持续轮询消耗 CPU)
- 不递归 (只监控顶层目录)

### 解决方案：原生 FSEvents API ✅ 已完成

使用 notify crate 实现原生 FSEvents API（底层调用 macOS FSEvents）：

**关键技术点:**
1. 使用 `notify::RecommendedWatcher` 创建事件流
2. 事件驱动，零轮询开销
3. 事件类型转换 (Create/Delete/Modify)
4. 事件去重和批处理

**实际收益:**
| 指标 | 轮询实现 | 原生 FSEvents | 改进 |
|------|----------|---------------|------|
| 事件延迟 | 100-1000ms | 12.27ms | 8-80x |
| CPU 占用 | 持续轮询 | 事件驱动 | 99% 降低 |
| 事件准确性 | 可能遗漏 | 内核保证 | 100% 可靠 |
| 递归监控 | ❌ 不支持 | ✅ 支持 | - |

**详细方案文档:** [docs/fsevents_solution.md](./fsevents_solution.md)

### 里程碑更新

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| M6: FSEvents 监控 | 🟢 已完成 | 轮询实现已工作 |
| M6b: 原生 FSEvents | 🟢 已完成 | 使用 notify crate 实现 |

---

## 1. 项目概述

### 1.1 项目目标

在 macOS 平台开发一款性能媲美甚至超越 Windows Everything 的文件搜索工具，提供：
- **毫秒级搜索响应** - 数百万文件索引下 sub-second 搜索完成
- **实时增量更新** - 文件系统变更秒级同步
- **低资源占用** - 内存和 CPU 使用最小化
- **CLI 优先，GUI 就绪** - 初期提供命令行界面，架构设计支持后续 GUI 扩展
- **Spotlight 独立** - 不依赖系统 Spotlight 服务

### 1.2 市场定位

> **macOS 上唯一不依赖 Spotlight 的高性能文件搜索工具**
>
> 兼具 fd 的速度、HoudahSpot 的功能、以及 Everything 的可靠性

### 1.3 目标用户

| 用户类型 | 占比 | 核心需求 | 优先级 |
|----------|------|----------|--------|
| **开发者** | 35% | CLI、快速、可脚本化、.gitignore 感知 | P0 |
| **设计师/创意工作者** | 25% | GUI、按类型搜索、预览、元数据过滤 | P1 |
| **研究人员/学生** | 20% | 内容搜索、元数据过滤、保存搜索 | P2 |
| **普通用户** | 20% | 简单、快速、可靠、系统托盘 | P1 |

### 1.4 对标 Everything 的核心指标

| 指标 | Everything (NTFS) | 本项目目标 (macOS APFS) | 测量方法 |
|------|-------------------|------------------------|----------|
| 初始索引时间 (100 万文件) | 5-10 秒 | ≤ 10 秒 | 基准测试 |
| 搜索响应时间 | < 50ms | < 50ms | P99 延迟 |
| 内存占用 (100 万文件) | ~150MB | ≤ 200MB | RSS |
| 实时同步延迟 | < 1 秒 | < 500ms | 事件到可见延迟 |
| CPU 空闲占用 | < 1% | < 1% | 后台监控 |
| 索引持久化恢复 | < 1 秒 | < 2 秒 | 冷启动时间 |

---

## 2. 文件系统兼容性设计

### 2.1 支持的文件系统

| 文件系统 | macOS 支持 | 索引策略 | 实时同步 | 优先级 |
|----------|------------|----------|----------|--------|
| **APFS** | ✅ 原生 | 遍历 + FSEvents | ✅ FSEvents | P0 |
| **HFS+** | ✅ 原生 | 遍历 + FSEvents | ✅ FSEvents | P0 |
| **exFAT** | ✅ 原生 | 遍历 (无事件) | ❌ 轮询 | P1 |
| **FAT32** | ✅ 原生 | 遍历 (无事件) | ❌ 轮询 | P1 |
| **NTFS** | ⚠️ 只读 | 遍历 (无事件) | ❌ 轮询 | P2 |
| **SMB/CIFS** | ✅ 网络 | 遍历 + 轮询 | ⚠️ 有限支持 | P2 |
| **NFS** | ✅ 网络 | 遍历 + 轮询 | ⚠️ 有限支持 | P2 |
| **加密卷 (DMG)** | ✅ 原生 | 挂载后处理 | ✅ FSEvents | P1 |

### 2.2 文件系统抽象层设计

```rust
/// 文件系统能力特征
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileSystemCapability {
    /// 支持实时事件通知
    SupportsEvents,
    /// 支持硬链接
    SupportsHardLinks,
    /// 支持符号链接
    SupportsSymlinks,
    /// 支持扩展属性
    SupportsExtendedAttributes,
    /// 支持资源分叉 (HFS+)
    SupportsResourceFork,
    /// 区分大小写
    CaseSensitive,
}

/// 文件系统类型
#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemType {
    Apfs,
    HfsPlus,
    ExFat,
    Fat32,
    Ntfs,
    Smb,
    Nfs,
    Unknown(String),
}

/// 文件系统信息
pub struct FileSystemInfo {
    pub fs_type: FileSystemType,
    pub capabilities: HashSet<FileSystemCapability>,
    pub mount_point: PathBuf,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub is_network: bool,
    pub is_removable: bool,
}

/// 文件系统监控策略
pub enum MonitorStrategy {
    /// FSEvents 实时监控 (APFS/HFS+)
    FSEvents,
    /// 定时轮询 (exFAT/FAT32/网络卷)
    Polling { interval: Duration },
    /// 混合模式
    Hybrid,
}

/// 文件系统抽象 trait
pub trait FileSystemBackend: Send + Sync {
    /// 获取文件系统信息
    fn get_info(&self, path: &Path) -> Result<FileSystemInfo>;

    /// 创建监控器
    fn create_monitor(&self, paths: &[PathBuf]) -> Result<Box<dyn FileSystemMonitor>>;

    /// 创建扫描器
    fn create_scanner(&self, config: ScannerConfig) -> Result<Box<dyn FileSystemScanner>>;

    /// 获取文件系统类型
    fn get_fs_type(&self, path: &Path) -> Result<FileSystemType>;
}
```

### 2.3 多文件系统处理策略

```
┌─────────────────────────────────────────────────────────────────┐
│                    文件系统抽象层                                │
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │
│  │  LocalFS      │  │  NetworkFS    │  │  VirtualFS    │        │
│  │  (APFS/HFS+)  │  │  (SMB/NFS)    │  │  (DMG/归档)   │        │
│  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘        │
│          │                  │                  │                 │
│          └──────────────────┼──────────────────┘                 │
│                             ↓                                     │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │              FileSystemAbstractionLayer                  │     │
│  │  - 统一路径表示                                          │     │
│  │  - 能力检测                                              │     │
│  │  - 策略路由                                              │     │
│  └─────────────────────────────────────────────────────────┘     │
│                             ↓                                     │
│          ┌──────────────────┼──────────────────┐                 │
│          │                  │                  │                 │
│  ┌───────┴───────┐  ┌───────┴───────┐  ┌───────┴───────┐        │
│  │ FSEvents      │  │ Polling       │  │ Hybrid        │        │
│  │ Monitor       │  │ Monitor       │  │ Monitor       │        │
│  └───────────────┘  └───────────────┘  └───────────────┘        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 核心架构设计 (优化版)

### 3.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           接口层 (Interface Layer)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  CLI        │  │  TUI        │  │  GUI (未来)  │  │  RPC/API    │     │
│  │  (clap)     │  │  (ratatui)  │  │  (Tauri)    │  │  (gRPC)     │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         └─────────────────┴─────────────────┴─────────────────┘          │
│                                   ↓                                      │
│  ┌─────────────────────────────────────────────────────────────────┐     │
│  │                      应用服务层 (Service Layer)                  │     │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │     │
│  │  │  SearchService│  │  IndexService │  │  MonitorService│       │     │
│  │  └───────────────┘  └───────────────┘  └───────────────┘        │     │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │     │
│  │  │  ConfigService│  │  PluginService│  │  Telemetry    │        │     │
│  │  └───────────────┘  └───────────────┘  └───────────────┘        │     │
│  └─────────────────────────────────────────────────────────────────┘     │
│                                   ↓                                      │
│  ┌─────────────────────────────────────────────────────────────────┐     │
│  │                      核心引擎层 (Core Engine)                    │     │
│  │  ┌─────────────────────────────────────────────────────────┐    │     │
│  │  │                   IndexEngine                           │    │     │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │    │     │
│  │  │  │ FST Index   │  │ Inode Map   │  │ Meta Cache   │      │    │     │
│  │  │  └─────────────┘  └─────────────┘  └─────────────┘      │    │     │
│  │  └─────────────────────────────────────────────────────────┘    │     │
│  │  ┌─────────────────────────────────────────────────────────┐    │     │
│  │  │                   QueryEngine                           │    │     │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │    │     │
│  │  │  │ QueryParser │  │ Optimizer   │  │ Executor     │      │    │     │
│  │  │  └─────────────┘  └─────────────┘  └─────────────┘      │    │     │
│  │  └─────────────────────────────────────────────────────────┘    │     │
│  └─────────────────────────────────────────────────────────────────┘     │
│                                   ↓                                      │
│  ┌─────────────────────────────────────────────────────────────────┐     │
│  │                    文件系统抽象层 (FSAL)                         │     │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │     │
│  │  │  ScanBackend  │  │ MonitorBackend│  │ StorageBackend│        │     │
│  │  └───────────────┘  └───────────────┘  └───────────────┘        │     │
│  └─────────────────────────────────────────────────────────────────┘     │
│                                   ↓                                      │
│  ┌─────────────────────────────────────────────────────────────────┐     │
│  │                      平台适配层 (Platform)                       │     │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │     │
│  │  │  macOS        │  │  Linux        │  │  Windows      │        │     │
│  │  │  (FSEvents)   │  │  (inotify)    │  │  (USN/RW)     │        │     │
│  │  └───────────────┘  └───────────────┘  └───────────────┘        │     │
│  └─────────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 分层架构原则

| 层级 | 职责 | 依赖方向 | 可测试性 |
|------|------|----------|----------|
| **接口层** | 用户交互、命令解析 | ↓ 依赖服务层 | 中 |
| **应用服务层** | 业务逻辑、状态管理 | ↓ 依赖核心引擎 | 高 |
| **核心引擎层** | 索引/搜索核心算法 | ↓ 依赖 FSAL | 高 |
| **文件系统抽象层** | 统一文件系统接口 | ↓ 依赖平台层 | 中 |
| **平台适配层** | OS 特定 API 封装 | ↓ 无依赖 | 低 |

### 3.3 模块职责与接口

#### 3.3.1 索引引擎 (IndexEngine)

```rust
/// 索引引擎 trait
#[async_trait]
pub trait IndexEngine: Send + Sync {
    /// 构建索引
    async fn build(&mut self, roots: &[PathBuf], config: &BuildConfig) -> Result<IndexStats>;

    /// 增量更新
    async fn update(&mut self, events: &[FSEvent]) -> Result<UpdateStats>;

    /// 搜索
    fn search(&self, query: &Query) -> Result<SearchResult>;

    /// 异步搜索 (流式返回)
    fn search_stream(&self, query: &Query) -> Result<BoxStream<'_, Result<SearchResultItem>>>;

    /// 导出索引
    async fn export(&self, writer: &mut dyn Write) -> Result<()>;

    /// 导入索引
    async fn import(&mut self, reader: &mut dyn Read) -> Result<()>;

    /// 获取索引统计
    fn stats(&self) -> IndexStats;

    /// 检查索引健康
    fn health_check(&self) -> IndexHealth;
}

/// 索引配置
pub struct IndexConfig {
    /// 内存限制
    pub memory_limit: Option<usize>,
    /// 并发度
    pub parallelism: usize,
    /// 排除模式
    pub exclude_patterns: Vec<Pattern>,
    /// 包含隐藏文件
    pub include_hidden: bool,
    /// .gitignore 感知
    pub gitignore_ignore: bool,
    /// 跟随符号链接
    pub follow_symlinks: bool,
    /// 索引元数据
    pub index_metadata: bool,
    /// 索引扩展属性
    pub index_xattr: bool,
}

/// 索引统计
pub struct IndexStats {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_symlinks: u64,
    pub total_bytes: u64,
    pub index_size_bytes: u64,
    pub build_time: Duration,
    pub last_update: Option<SystemTime>,
    pub health: IndexHealth,
}
```

#### 3.3.2 查询引擎 (QueryEngine)

```rust
/// 查询引擎 trait
pub trait QueryEngine: Send + Sync {
    /// 解析查询
    fn parse(&self, input: &str) -> Result<Query>;

    /// 执行查询
    fn execute(&self, query: &Query, index: &dyn IndexEngine) -> Result<SearchResult>;

    /// 优化查询
    fn optimize(&self, query: &Query) -> Result<Query>;
}

/// 查询 AST
#[derive(Debug, Clone)]
pub enum QueryNode {
    /// 文件名匹配
    Filename { pattern: Pattern, case_sensitive: bool },
    /// 路径匹配
    Path { pattern: Pattern },
    /// 扩展名匹配
    Extension { ext: String },
    /// 大小范围
    Size { min: Option<u64>, max: Option<u64> },
    /// 时间范围
    Modified { after: Option<SystemTime>, before: Option<SystemTime> },
    /// 文件类型
    FileType { kind: FileKind },
    /// 布尔 AND
    And { left: Box<QueryNode>, right: Box<QueryNode> },
    /// 布尔 OR
    Or { left: Box<QueryNode>, right: Box<QueryNode> },
    /// 布尔 NOT
    Not { inner: Box<QueryNode> },
    /// 模糊匹配
    Fuzzy { term: String, threshold: f64 },
}

/// 搜索选项
pub struct SearchOptions {
    /// 最大结果数
    pub limit: Option<usize>,
    /// 偏移量
    pub offset: Option<usize>,
    /// 排序字段
    pub sort_by: SortField,
    /// 排序顺序
    pub order: SortOrder,
    /// 分组方式
    pub group_by: Option<GroupBy>,
    /// 高亮匹配
    pub highlight: bool,
    /// 包含分数
    pub include_score: bool,
}
```

#### 3.3.3 监控服务 (MonitorService)

```rust
/// 监控服务 trait
#[async_trait]
pub trait MonitorService: Send + Sync {
    /// 开始监控
    async fn start(&mut self, roots: &[PathBuf], config: &MonitorConfig) -> Result<()>;

    /// 停止监控
    async fn stop(&mut self) -> Result<()>;

    /// 暂停监控
    async fn pause(&mut self) -> Result<()>;

    /// 恢复监控
    async fn resume(&mut self) -> Result<()>;

    /// 获取事件流
    fn event_stream(&self) -> BoxStream<'_, FSEvent>;

    /// 获取监控状态
    fn status(&self) -> MonitorStatus;
}

/// 监控配置
pub struct MonitorConfig {
    /// 批处理窗口
    pub batch_window: Duration,
    /// 批处理大小
    pub batch_size: usize,
    /// 去重时间窗口
    pub dedup_window: Duration,
    /// 事件缓冲区大小
    pub buffer_size: usize,
    /// 轮询间隔 (用于不支持事件的 FS)
    pub polling_interval: Option<Duration>,
    /// 递归监控
    pub recursive: bool,
}
```

---

## 4. 关键功能需求

### 4.1 核心功能 (P0)

| ID | 功能 | 描述 | 优先级 | 复杂度 |
|----|------|------|--------|--------|
| F001 | 文件名搜索 | 支持前缀、通配符、正则匹配 | P0 | 低 |
| F002 | 实时索引 | FSEvents 实时监控文件系统变更 | P0 | 中 |
| F003 | 增量更新 | 基于事件的索引增量更新 | P0 | 中 |
| F004 | 索引持久化 | 索引导出/导入，快速恢复 | P0 | 中 |
| F005 | CLI 界面 | 完整的命令行界面 | P0 | 低 |
| F006 | .gitignore 感知 | 自动跳过 gitignore 的文件 | P0 | 低 |
| F007 | 元数据过滤 | 按大小、时间、类型过滤 | P0 | 低 |
| F008 | 布尔搜索 | AND/OR/NOT 组合查询 | P0 | 中 |

### 4.2 重要功能 (P1)

| ID | 功能 | 描述 | 优先级 | 复杂度 |
|----|------|------|--------|--------|
| F101 | TUI 界面 | 终端用户界面 (ratatui) | P1 | 中 | ✅ 已完成 |
| F102 | 多卷支持 | 支持多个挂载点/卷 | P1 | 中 |
| F103 | 排除目录 | 配置排除目录列表 | P1 | 低 |
| F104 | 搜索历史 | 保存和重用搜索历史 | P1 | 低 |
| F105 | 结果分组 | 按类型/目录分组结果 | P1 | 低 |
| F106 | 模糊搜索 | 支持模糊匹配 (similarity) | P1 | 中 |
| F107 | 索引健康检查 | 检测索引过期/损坏 | P1 | 中 |
| F108 | 后台服务 | launchd 集成，开机启动 | P1 | 中 | ✅ 已完成 |

### 4.3 高级功能 (P2)

| ID | 功能 | 描述 | 优先级 | 复杂度 |
|----|------|------|--------|--------|
| F201 | GUI 界面 | macOS 原生 GUI (SwiftUI/Tauri) | P2 | 高 |
| F202 | 内容搜索 | 文件内容全文检索 | P2 | 高 |
| F203 | 扩展属性 | 索引和搜索 xattr | P2 | 中 |
| F204 | 保存搜索 | 保存常用搜索为智能文件夹 | P2 | 中 |
| F205 | 插件系统 | 支持第三方插件扩展 | P2 | 高 |
| F206 | 网络卷 | SMB/NFS 网络卷支持 | P2 | 中 |
| F207 | 云存储 | iCloud/ Dropbox 集成 | P2 | 高 |
| F208 | API/RPC | gRPC/REST API 暴露 | P2 | 中 |
| F209 | 跨平台 | Linux/Windows 支持 | P2 | 高 |

### 4.4 功能详细说明

#### F001: 文件名搜索

```bash
# 前缀匹配 (最快)
mfind "app"           # 文件名以"app"开头

# 通配符匹配
mfind "*.app"         # 所有.app 文件
mfind "test_??.rs"    # test_后跟两个字符

# 正则匹配
mfind --regex ".*\.(jpg|png|gif)$"

# 子串匹配
mfind --contains "config"

# 模糊匹配
mfind --fuzzy "readme"  # 匹配 README.md, ReadMe.txt 等
```

#### F006: .gitignore 感知

```bash
# 默认启用.gitignore 感知
mfind "Cargo.toml"      # 自动跳过.gitignore 中的文件

# 禁用.gitignore 感知
mfind "Cargo.toml" --no-gitignore

# 支持全局.gitignore
# 读取 ~/.config/git/ignore
```

#### F108: 后台服务

```bash
# 安装后台服务
mfind service install

# 启动服务
mfind service start

# 查看状态
mfind service status

# 停止服务
mfind service stop

# 卸载服务
mfind service uninstall
```

launchd plist 配置:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.mfind.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/mfind</string>
        <string>serve</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

---

## 5. 技术选型 (更新版)

### 5.1 编程语言

**主语言：Rust**

| 模块 | 语言 | 理由 |
|------|------|------|
| 核心引擎 | Rust | 性能、内存安全 |
| CLI | Rust | 生态好 (clap) |
| TUI | Rust | ratatui 成熟 |
| GUI (未来) | Rust + TypeScript | Tauri 跨平台 |
| 绑定 | Rust + Swift | 原生 macOS 集成 |

### 5.2 核心依赖库 (更新)

```toml
[workspace]
members = [
    "crates/mfind-core",
    "crates/mfind-cli",
    "crates/mfind-tui",
    "crates/mfind-api",
]

[workspace.dependencies]
# 核心
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "1"
anyhow = "1"

# 文件系统
notify = "6"                    # FSEvents 封装
walkdir = "2"                   # 目录遍历
ignore = "0.4"                  # .gitignore 解析
path-absolutize = "3"           # 路径处理
dunce = "1"                     # Windows 路径规范化

# 索引数据结构
fst = "0.4"                     # FST
dashmap = "5"                   # 并发 HashMap
rustc-hash = "1"                # 快速哈希
xxhash-rust = "0.8"             # 快速哈希

# 搜索
regex = "1"                     # 正则表达式
fuzzy-matcher = "0.3"           # 模糊匹配
levenshtein_automata = "0.2"    # 编辑距离

# 并发
rayon = "1"                     # 数据并行
crossbeam = "0.8"               # 无锁并发
flume = "0.11"                  # 通道

# 持久化
lmdb-rkv = "0.14"               # LMDB
# 备选
# rocksdb = "0.21"
# sled = "0.34"

# 序列化
serde = { version = "1", features = ["derive"] }
bincode = "1"                   # 二进制序列化
rmp-serde = "1"                 # MessagePack

# CLI
clap = { version = "4", features = ["derive"] }
clap_complete = "4"             # Shell 补全
dialoguer = "0.11"              # 交互式 CLI
indicatif = "0.17"              # 进度条

# TUI
ratatui = "0.24"
crossterm = "0.27"
ansi-term = "0.12"

# 日志/追踪
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# 配置
config = "0.14"
directories = "5"               # 平台特定目录

# 网络 (未来)
# tonic = "0.10"               # gRPC
# axum = "0.7"                 # HTTP 服务器

# 测试
criterion = "0.5"               # 基准测试
proptest = "1"                  # 属性测试
tempfile = "3"                  # 临时文件

# GUI (未来)
# tauri = "1"                  # Tauri GUI
```

### 5.3 索引数据结构最终选型

| 用途 | 数据结构 | 库 | 理由 |
|------|----------|-----|------|
| 文件名索引 | FST | `fst` | 极高压缩率，O(m) 查找 |
| 路径映射 | DashMap | `dashmap` | 并发安全，O(1) 查找 |
| 元数据缓存 | LRU Cache + DashMap | `lru` + `dashmap` | 热点数据缓存 |
| 范围查询 | B-Tree | 内置 | 大小/时间范围 |
| 模糊搜索 | Levenshtein Automaton | `levenshtein_automata` | 编辑距离搜索 |
| 持久化 | LMDB | `lmdb-rkv` | 极快读取，ACID |

---

## 6. 项目结构 (优化版)

```
mfind/
├── Cargo.toml                    # Workspace 配置
├── Cargo.lock
├── README.md
├── README-zh.md                  # 中文文档
├── LICENSE                       # MIT/Apache 2.0
├── .gitignore
├── rust-toolchain.toml           # Rust 版本
├── .rustfmt.toml                 # 代码格式
├── .clippy.toml                  # Lint 配置
├── CHANGELOG.md
├── CONTRIBUTING.md
│
├── crates/
│   │
│   ├── mfind-core/               # 核心引擎 (库)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # 公共 API
│   │       ├── index/            # 索引模块
│   │       │   ├── mod.rs
│   │       │   ├── engine.rs     # IndexEngine trait
│   │       │   ├── fst_index.rs  # FST 实现
│   │       │   ├── inode_map.rs  # inode 映射
│   │       │   ├── meta_cache.rs # 元数据缓存
│   │       │   └── stats.rs      # 统计信息
│   │       │
│   │       ├── query/            # 查询模块
│   │       │   ├── mod.rs
│   │       │   ├── ast.rs        # 查询 AST
│   │       │   ├── parser.rs     # 查询解析
│   │       │   ├── optimizer.rs  # 查询优化
│   │       │   ├── executor.rs   # 查询执行
│   │       │   └── pattern.rs    # 模式匹配
│   │       │
│   │       ├── fs/               # 文件系统模块
│   │       │   ├── mod.rs
│   │       │   ├── backend.rs    # FS backend trait
│   │       │   ├── scanner.rs    # 扫描器
│   │       │   ├── monitor.rs    # 监控器
│   │       │   ├── watcher.rs    # FSEvents 封装
│   │       │   └── info.rs       # 文件系统信息
│   │       │
│   │       ├── storage/          # 存储模块
│   │       │   ├── mod.rs
│   │       │   ├── trait.rs      # Storage trait
│   │       │   ├── lmdb.rs       # LMDB 实现
│   │       │   └── memory.rs     # 内存实现
│   │       │
│   │       ├── event/            # 事件模块
│   │       │   ├── mod.rs
│   │       │   ├── fs_event.rs   # FSEvent 定义
│   │       │   ├── batch.rs      # 事件批处理
│   │       │   └── dedup.rs      # 事件去重
│   │       │
│   │       └── util/             # 工具模块
│   │           ├── mod.rs
│   │           ├── path.rs       # 路径工具
│   │           ├── time.rs       # 时间工具
│   │           └── format.rs     # 格式工具
│   │
│   ├── mfind-cli/                # CLI 可执行文件
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs            # 应用入口
│   │       ├── commands/         # 命令实现
│   │       │   ├── mod.rs
│   │       │   ├── search.rs     # 搜索命令
│   │       │   ├── index.rs      # 索引命令
│   │       │   ├── config.rs     # 配置命令
│   │       │   └── service.rs    # 服务命令
│   │       ├── output/           # 输出格式化
│   │       │   ├── mod.rs
│   │       │   ├── table.rs      # 表格输出
│   │       │   ├── json.rs       # JSON 输出
│   │       │   └── list.rs       # 列表输出
│   │       └── config/           # CLI 配置
│   │           └── mod.rs
│   │
│   ├── mfind-tui/                # TUI 界面
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs            # TUI 应用
│   │       ├── ui/               # UI 组件
│   │       │   ├── mod.rs
│   │       │   ├── search.rs     # 搜索界面
│   │       │   ├── result.rs     # 结果列表
│   │       │   └── help.rs       # 帮助界面
│   │       └── state.rs          # 状态管理
│   │
│   └── mfind-api/                # API/RPC (未来)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── grpc/             # gRPC 服务
│           └── http/             # HTTP API
│
├── bindings/                     # 语言绑定
│   ├── swift/                    # Swift 绑定
│   │   ├── Sources/
│   │   └── Package.swift
│   └── python/                   # Python 绑定 (未来)
│       ├── pyproject.toml
│       └── mfind/
│
├── tests/
│   ├── integration/              # 集成测试
│   │   ├── search_test.rs
│   │   ├── index_test.rs
│   │   └── fs_test.rs
│   └── benchmarks/               # 基准测试
│       ├── search_bench.rs
│       └── index_bench.rs
│
├── docs/                         # 文档
│   ├── architecture.md           # 架构文档
│   ├── api.md                    # API 文档
│   ├── development.md            # 开发指南
│   └── user-guide/               # 用户指南
│       ├── installation.md
│       ├── usage.md
│       └── config.md
│
└── scripts/                      # 脚本
    ├── benchmark.sh              # 基准测试脚本
    ├── release.sh                # 发布脚本
    └── generate-completions.sh   # 生成 Shell 补全
```

---

## 7. 开发路线图 (更新版)

### 阶段 1: MVP (4-6 周)

**目标：** 可用的 CLI 工具，核心功能可用

**周次分解：**

| 周次 | 任务 | 交付物 |
|------|------|--------|
| W1 | 项目脚手架、核心 trait 定义 | crates 结构、IndexEngine trait |
| W2 | FST 索引实现、基本扫描器 | fst_index.rs、scanner.rs |
| W3 | 查询解析器、执行器 | parser.rs、executor.rs |
| W4 | CLI 框架、搜索命令 | mfind CLI、search 命令 |
| W5 | .gitignore 感知、排除配置 | ignore 集成、config |
| W6 | 测试、文档、基准测试 | 集成测试、README |

**交付物：**
- `mfind <pattern>` 基本搜索
- `mfind index build <path>` 索引构建
- `.gitignore` 感知
- 基础文档

### 阶段 2: 完善 CLI (4-6 周)

| 周次 | 任务 | 交付物 |
|------|------|--------|
| W1-2 | FSEvents 监控 | watcher.rs、MonitorService |
| W3 | 增量更新逻辑 | 索引增量更新 |
| W4 | 复杂查询语法 | 布尔搜索、正则 |
| W5 | 索引持久化 | LMDB 存储 |
| W6 | TUI 界面、性能优化 | mfind-tui、基准报告 | ✅ 已完成 |

**交付物：**
- 实时索引更新
- 复杂搜索语法
- TUI 界面 ✅ 已完成
- 性能对标 fd

### 阶段 3: 服务化 (4 周) 🟢

| 周次 | 任务 | 交付物 |
|------|------|--------|
| W1 | 后台服务架构 | ✅ serve 命令、守护进程 |
| W2 | launchd 集成 | ✅ plist 配置、service 命令 |
| W3 | HTTP/REST API | ✅ axum 服务器、/health /stats /search 端点 |
| W4 | 配置系统 | ⚪ 配置文件、环境变量 |

**交付物：**
- ✅ 后台常驻服务
- ✅ 开机自启动
- ✅ HTTP/REST API

### 阶段 4: GUI 开发 (6-8 周) 🟢

| 周次 | 任务 | 交付物 |
|------|------|--------|
| W1-2 | Tauri 框架搭建 | ✅ mfind-gui crate、Tauri v2 配置 |
| W3-4 | 搜索界面 | ✅ 搜索框、结果列表、预览面板 |
| W5 | 高级功能 | ✅ 搜索历史、高级搜索选项 |
| W6-7 | 系统集成 | ✅ 菜单栏、单实例运行、Spotlight 式启动 |
| W8 | 测试发布 | ✅ GUI 应用、代码签名、GitHub Release |

**交付物：**
- ✅ Tauri 框架 (M14 完成)
- ✅ 搜索界面增强 (M15 完成)
  - ✅ 高级搜索选项（正则、通配符、区分大小写）
  - ✅ 搜索历史记录（localStorage，最多 20 条）
  - ✅ 文件预览（文本和图片格式）
  - ✅ 预览面板（可关闭）
- ✅ 系统集成 (M16 完成)
  - ✅ 菜单栏图标（系统托盘）
  - ✅ 右键菜单（显示/隐藏窗口、退出）
  - ✅ 单实例运行
  - ✅ Spotlight 式启动（Esc 键隐藏）
  - ✅ Cmd+K 快捷键聚焦搜索框
- ✅ GUI 发布 (M17 完成)
  - ✅ macOS 应用 Bundle (app, dmg 格式)
  - ✅ 代码签名脚本
  - ✅ GitHub Actions CI/CD 工作流
  - ✅ 构建和发布脚本

### 阶段 5: 跨平台 (持续)

| 阶段 | 平台 | 任务 |
|------|------|------|
| P1 | macOS | 优化完善 |
| P2 | Linux | inotify 支持 |
| P3 | Windows | USN Journal 支持 |

---

## 8. 配置系统设计

### 8.1 配置文件结构

```toml
# ~/.config/mfind/config.toml

# 全局设置
[global]
# 最大内存使用 (MB)
memory_limit = 512
# 并行度
parallelism = 4
# 日志级别
log_level = "info"

# 索引设置
[index]
# 默认索引路径
roots = ["~/Documents", "~/Downloads", "~/Desktop"]
# 排除目录
exclude_dirs = [
    "node_modules",
    ".git",
    "target",
    "__pycache__",
    ".cache",
]
# 排除模式
exclude_patterns = ["*.log", "*.tmp", "*.bak"]
# 包含隐藏文件
include_hidden = false
# .gitignore 感知
gitignore = true
# 跟随符号链接
follow_symlinks = false
# 索引元数据
index_metadata = true
# 索引扩展属性
index_xattr = false

# 搜索设置
[search]
# 默认结果限制
default_limit = 1000
# 高亮匹配
highlight = true
# 模糊搜索阈值
fuzzy_threshold = 0.6

# 监控设置
[monitor]
# 启用实时监控
enabled = true
# 批处理窗口 (ms)
batch_window_ms = 100
# 轮询间隔 (用于不支持事件的 FS)
polling_interval_s = 60

# UI 设置
[ui]
# 颜色主题
theme = "dark"
# 日期格式
date_format = "%Y-%m-%d %H:%M"
# 大小格式 (si/iec)
size_format = "iec"
```

### 8.2 环境变量

```bash
MFIND_CONFIG_HOME=~/.config/mfind
MFIND_DATA_HOME=~/.local/share/mfind
MFIND_CACHE_HOME=~/Library/Caches/mfind
MFIND_LOG_LEVEL=info
MFIND_PARALLELISM=4
MFIND_MEMORY_LIMIT=512
```

---

## 9. 性能优化策略 (更新版)

### 9.1 索引构建优化

| 优化点 | 策略 | 实现 | 预期收益 |
|--------|------|------|----------|
| 并行遍历 | rayon 多线程 | `par_bridge()` | 4-8x 加速 |
| 批量 FST 构建 | 一次性构建 | `fst::SetBuilder` | 10x 加速 |
| 跳过系统目录 | 预定义排除列表 | 内置排除 | 减少 30% 文件 |
| .gitignore 感知 | ignore crate | 提前剪枝 | 减少 50% 文件 |
| 索引缓存 | 增量更新 | LMDB 持久化 | 冷启动 10x 加速 |
| 内存池 | 复用分配 | `bumpalo` | 减少 GC 压力 |

### 9.2 搜索优化

| 优化点 | 策略 | 实现 | 预期收益 |
|--------|------|------|----------|
| FST 前缀扫描 | O(m) 复杂度 | `fst::Stream` | 亚毫秒级 |
| 查询优化 | 重写规则 | 优化器 | 2-4x 加速 |
| 结果缓存 | LRU 缓存 | `lru` crate | 重复查询 100x |
| 并发执行 | 并行搜索 | `rayon` | 2x 加速 |
| SIMD 加速 | 向量化 | `std::arch` | 2x 加速 |
| 分支预测 | 提示优化 | `likely/unlikely` | 10-20% |

### 9.3 内存优化

| 优化点 | 策略 | 实现 | 预期收益 |
|--------|------|------|----------|
| FST 压缩 | 状态合并 | fst 自动 | 5-10x 压缩 |
| 字符串驻留 | 共享前缀 | `Arc<str>` | 减少 30% |
| 惰性加载 | 按需加载 | 元数据缓存 | 减少 50% |
| LMDB 溢出 | 大索引磁盘 | lmdb-rkv | 无上限 |
| 零拷贝 | 避免复制 | `&[u8]` | 减少分配 |

### 9.4 基准测试目标

```rust
// benchmarks/search_bench.rs

#[bench]
fn bench_prefix_search_100k(b: &mut Bencher) {
    // 10 万文件，前缀搜索 < 10ms
}

#[bench]
fn bench_regex_search_100k(b: &mut Bencher) {
    // 10 万文件，正则搜索 < 50ms
}

#[bench]
fn bench_build_index_1m(b: &mut Bencher) {
    // 100 万文件，索引构建 < 10 秒
}

#[bench]
fn bench_memory_1m(b: &mut Bencher) {
    // 100 万文件，内存占用 < 200MB
}
```

---

## 10. 差异化竞争策略

### 10.1 核心差异化

| 维度 | 本项目 | Spotlight | fd | HoudahSpot |
|------|--------|-----------|----|------------|
| 索引独立性 | ✅ 完全独立 | ❌ 系统索引 | ❌ 无索引 | ❌ 依赖 Spotlight |
| 实时同步 | ✅ FSEvents | ✅ | ❌ | ❌ |
| CLI 支持 | ✅ 完整 | ⚠️ 有限 | ✅ | ❌ |
| GUI 计划 | ✅ Tauri | ✅ | ❌ | ✅ |
| 开源免费 | ✅ | ❌ | ✅ | ❌ |
| .gitignore | ✅ | ❌ | ✅ | ❌ |
| 可脚本化 | ✅ gRPC | ⚠️ | ⚠️ | ❌ |

### 10.2 独特卖点 (USP)

1. **唯一独立的专业搜索工具** - macOS 上唯一不依赖 Spotlight 的高性能搜索
2. **开发者优先** - .gitignore 感知、JSON 输出、API 接口
3. **双重形态** - CLI 和 GUI 同等重要
4. **开源透明** - 建立信任，社区共建

### 10.3 护城河

| 护城河 | 说明 | 可持续性 |
|--------|------|----------|
| 技术架构 | FST + FSEvents + Rust | 高 |
| 用户习惯 | CLI 工作流集成 | 中 |
| 开源社区 | 贡献者生态 | 高 |
| 性能优势 | 基准测试领先 | 中 |

---

## 11. 风险评估与缓解

| 风险 | 影响 | 概率 | 缓解措施 | 负责人 |
|------|------|------|----------|--------|
| APFS 遍历慢于 NTFS | 高 | 中 | 并行优化、缓存 | 技术 |
| FSEvents 事件丢失 | 中 | 低 | 定期校验、轮询兜底 | 技术 |
| 内存占用过高 | 中 | 中 | FST 压缩、LMDB 溢出 | 技术 |
| 用户不愿安装 | 中 | 中 | Homebrew、突出差异化 | 市场 |
| 竞品跟进 | 低 | 中 | 快速迭代、社区建设 | 市场 |
| 开发资源不足 | 高 | 高 | 分阶段、聚焦 MVP | 管理 |
| GUI 复杂度超预期 | 中 | 高 | Tauri 跨平台、延后 | 技术 |

---

## 12. 成功标准 (更新版)

### 12.1 功能标准

- [ ] 支持 Everything 所有基本搜索语法
- [ ] 实时同步延迟 < 500ms
- [ ] 索引持久化，重启 < 2 秒恢复
- [ ] .gitignore 感知正常工作
- [x] 后台服务稳定运行

### 12.2 性能标准

| 场景 | 目标 | 测量方法 |
|------|------|----------|
| 10 万文件前缀搜索 | < 10ms | P99 延迟 |
| 100 万文件前缀搜索 | < 50ms | P99 延迟 |
| 复杂正则搜索 | < 200ms | P99 延迟 |
| 100 万文件初始索引 | < 10 秒 | 端到端 |
| FSEvents 事件处理 | < 500ms | 事件到可见 |
| 内存占用 (100 万文件) | < 200MB | RSS |
| 冷启动时间 | < 2 秒 | 进程启动到可搜索 |

### 12.3 质量标准

| 指标 | 目标 |
|------|------|
| 单元测试覆盖率 | > 80% |
| 集成测试通过率 | 100% |
| Clippy 警告 | 0 |
| 文档完整度 | 100% |

### 12.4 用户标准

- [ ] Homebrew 可安装
- [ ] README 完整 (中英文)
- [ ] 示例丰富 (10+ 场景)
- [ ] 错误信息友好
- [ ] Shell 补全完整

---

## 13. 参考项目与资源

| 项目 | 语言 | 参考点 | URL |
|------|------|--------|-----|
| fd | Rust | 并行遍历、CLI 设计 | https://github.com/sharkdp/fd |
| ripgrep | Rust | 索引架构、性能优化 | https://github.com/BurntSushi/ripgrep |
| fzf | Go | TUI 设计、模糊搜索 | https://github.com/junegunn/fzf |
| notify | Rust | FSEvents 封装 | https://github.com/notify-rs/notify |
| Everything | C | 产品设计、用户体验 | https://www.voidtools.com/ |
| mdfind | C | Spotlight CLI | macOS 系统自带 |

---

## 14. 附录

### A. 术语表

| 术语 | 定义 |
|------|------|
| FST | Finite State Transducer，有限状态转换器 |
| FSEvents | macOS 文件系统事件 API |
| APFS | Apple File System，macOS 默认文件系统 |
| ino/inode | 文件索引节点，唯一标识文件 |
| LMDB | Lightning Memory-Mapped Database |

### B. 决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-03-22 | 选择 Rust | 性能、安全、生态 |
| 2026-03-22 | 使用 FST | 内存效率、搜索性能 |
| 2026-03-22 | 不依赖 Spotlight | 独立性、可靠性 |
| 2026-03-22 | CLI 优先 | 开发者需求、简单启动 |

### C. 变更日志

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-03-22 | 初始版本 |
| v2.0 | 2026-03-22 | 根据市场调研更新，补充功能点、架构优化 |

---

*文档结束*
