# mfind 项目总览

> 更新日期：2026-04-01
> 本文档用于描述项目当前状态，不再保留早期阶段的历史快照式“下一步工作”。

## 项目定位

mfind 是一个面向本地文件系统的高性能搜索工具，目标是提供脱离 Spotlight 的文件索引与检索能力，并同时支持：

- CLI
- TUI
- HTTP API
- Tauri GUI
- 后续跨平台扩展

## 当前状态

项目已经完成从单一 CLI 工具到“搜索内核 + 多入口界面”的第一轮演进。

当前仓库包含 5 个 workspace crate：

- `mfind-core`
  - 核心索引、查询、文件系统监控、存储抽象
- `mfind-cli`
  - 命令行入口
- `mfind-tui`
  - 终端交互界面
- `mfind-api`
  - HTTP/REST API
- `mfind-gui`
  - 基于 Tauri 的桌面界面

## 已落地能力

### 搜索与索引

- FST 文件名索引
- 前缀、通配符、正则、布尔查询
- `.gitignore` 感知
- 元数据缓存与 inode 映射
- 索引导出/导入与持久化恢复

### 文件系统同步

- macOS 原生 FSEvents 路径
- 事件批处理与去重
- 增量更新
- 面向不同平台的监控后端抽象

### 入口层

- CLI 搜索、索引、服务管理命令
- TUI 搜索界面
- HTTP `/health`、`/stats`、`/search` 接口
- Tauri GUI 搜索、预览、系统托盘、快捷键

### 工程化

- 集成测试
- 场景测试
- 基准测试
- 发布脚本与桌面打包配置

## 当前技术结构

项目采用分层架构：

1. 接口层
   - CLI / TUI / GUI / API
2. 服务与编排层
   - 搜索、索引、监控、配置、服务生命周期
3. 核心引擎层
   - IndexEngine / QueryEngine / Storage
4. 文件系统抽象层
   - Scanner / Monitor / Backend
5. 平台适配层
   - macOS / Linux / Windows

更详细的设计说明见 `docs/architecture.md`。

## 当前文档分工

- `README.md`
  - 对外说明与快速开始
- `requirements.md`
  - 产品需求、能力分级、长期演进方向
- `MILESTONES.md`
  - 已完成和待完成里程碑
- `docs/README.md`
  - 全量文档导航

## 下一阶段重点

下一阶段建议聚焦在“把已有能力收口成稳定产品”，而不是继续平铺新功能：

1. 统一文档和外部说明，减少规划与实现脱节
2. 校准 GUI、API、CLI 三个入口的能力边界
3. 补齐大规模数据下的性能和内存验证
4. 明确跨平台能力的真实完成度与发布标准
5. 决定内容搜索、插件系统、RPC/gRPC 的优先级
