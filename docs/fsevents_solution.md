# 后台持续运行场景解决方案

**文档日期:** 2026-03-23
**优先级:** P0
**预计工作量:** 6-8 小时

---

## 问题分析

### 当前实现问题

当前 `FSEventsWatcher` 使用轮询方式监控文件系统变化：

```rust
// 当前实现：轮询
while running.load(Ordering::SeqCst) {
    thread::sleep(poll_interval);  // 轮询间隔
    for path in &watched_paths {
        // 只监控顶层路径，不递归
        if let Ok(metadata) = std::fs::metadata(path) {
            // 比较大小判断变化
        }
    }
}
```

**问题:**
1. ❌ 非实时 - 轮询间隔导致事件延迟 (100ms~1s)
2. ❌ 遗漏事件 - 快速连续的文件变化可能被跳过
3. ❌ 资源浪费 - 持续轮询消耗 CPU
4. ❌ 不递归 - 只监控顶层目录
5. ❌ 功能有限 - 无法区分创建/删除/修改/重命名等事件类型

---

## 解决方案：原生 FSEvents API

### 方案设计

使用 macOS 原生 FSEvents API，通过 `core-foundation` crate  bindings:

```
┌─────────────────────────────────────────────────────────────┐
│                     mfind CLI/TUI                            │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐ │
│  │ IndexEngine │  │ EventProcessor│  │ NativeFSEventsWatcher││
│  └─────────────┘  └──────────────┘  └─────────────────────┘ │
│         ↑                ↑                      ↑             │
│         │                │                      │             │
│  ┌──────┴────────────────┴──────────────────────┴─────┐      │
│  │              FSEvent Channel (flume)                │      │
│  └─────────────────────────────────────────────────────┘      │
└───────────────────────────────────────────────────────────────┘
                              ↑
                              │ FSEvent Stream
                              │
                    ┌─────────┴──────────┐
                    │   macOS FSEvents   │
                    │   (Kernel Level)   │
                    └────────────────────┘
```

### 核心 API

```rust
use core_foundation::{
    array::CFArray,
    base::{CFRelease, TCFType},
    mach_port::CFAllocatorRef,
    run_loop::{CFRunLoop, CFRunLoopMode},
    string::CFString,
};
use core_foundation_sys::file_system::{
    FSEventStreamCreate, FSEventStreamScheduleWithRunLoop, FSEventStreamStart,
    FSEventStreamStop, FSEventStreamInvalidate,
    FSEventStreamContext, FSEventStreamFlags,
    FSEventStreamEventId, FSEventStreamEventCallback,
};
```

### 实现结构

```rust
/// 原生 FSEvents 观察者
pub struct NativeFSEventsWatcher {
    /// FSEvent 流引用
    stream_ref: FSEventStreamRef,
    /// 运行循环
    run_loop: Option<CFRunLoop>,
    /// 事件发送器
    event_sender: flume::Sender<FSEvent>,
    /// 配置
    config: MonitorConfig,
    /// 监控路径
    paths: Vec<PathBuf>,
}

impl NativeFSEventsWatcher {
    /// 创建新的 FSEvents 观察者
    pub fn new(config: MonitorConfig, sender: flume::Sender<FSEvent>) -> Result<Self> {
        Ok(Self {
            stream_ref: ptr::null_mut(),
            run_loop: None,
            event_sender: sender,
            config,
            paths: Vec::new(),
        })
    }

    /// 启动监控
    pub fn start(&mut self, paths: &[PathBuf]) -> Result<()> {
        self.paths = paths.to_vec();

        // 1. 创建 CFArray 路径
        let cf_paths = self.create_cf_paths();

        // 2. 设置回调上下文
        let context = self.create_context();

        // 3. 创建 FSEvent 流
        self.stream_ref = unsafe {
            FSEventStreamCreate(
                kCFAllocatorDefault,
                Some(fsevent_callback),
                &context,
                cf_paths.as_concrete_TypeRef(),
                kFSEventStreamEventIdSinceNow,
                self.config.latency.as_secs_f64(),
                self.get_flags(),
            )
        };

        // 4. 调度到运行循环
        self.schedule_with_run_loop()?;

        // 5. 启动流
        unsafe { FSEventStreamStart(self.stream_ref) };

        Ok(())
    }

    /// 获取 FSEvent 标志
    fn get_flags(&self) -> FSEventStreamFlags {
        kFSEventStreamCreateFlagFileEvents
            | kFSEventStreamCreateFlagWatchRoot
            | kFSEventStreamCreateFlagIgnoreSelf
    }
}

/// FSEvent 回调函数
extern "C" fn fsevent_callback(
    stream_ref: ConstFSEventStreamRef,
    client_callback_info: *mut c_void,
    num_events: usize,
    event_paths: *mut c_void,
    event_flags: *const FSEventStreamEventFlags,
    event_ids: *const FSEventStreamEventId,
) {
    let ctx = unsafe { &mut *(client_callback_info as *mut StreamContext) };

    for i in 0..num_events {
        let path = unsafe {
            CStr::from_ptr(*(event_paths as *const *const i8).add(i))
        };
        let flags = unsafe { *event_flags.add(i) };

        // 转换 FSEvent 标志为我们的 FSEventType
        let event_type = flags_to_event_type(flags);

        // 发送事件
        let _ = ctx.sender.send(FSEvent {
            path: PathBuf::from(path.to_str().unwrap()),
            event_type,
            timestamp: SystemTime::now(),
            ..
        });
    }
}
```

---

## 事件类型映射

| FSEvent 标志 | 我们的 FSEventType |
|-------------|-------------------|
| `kFSEventStreamEventFlagItemCreated` | `FSEventType::Create` |
| `kFSEventStreamEventFlagItemRemoved` | `FSEventType::Delete` |
| `kFSEventStreamEventFlagItemModified` | `FSEventType::Modify` |
| `kFSEventStreamEventFlagItemRenamed` | `FSEventType::Rename` |
| `kFSEventStreamEventFlagItemInodeMetaMod` | `FSEventType::Metadata` |

---

## 依赖配置

```toml
# crates/mfind-core/Cargo.toml
[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.10"
core-foundation-sys = "0.8"
```

---

## 实现步骤

### Step 1: 创建原生 FSEvents 模块

文件：`crates/mfind-core/src/fs/native_fsevents.rs`

- [ ] 定义 `NativeFSEventsWatcher` 结构
- [ ] 实现 `new()` 构造函数
- [ ] 实现 `start()` 启动方法
- [ ] 实现 `stop()` 停止方法
- [ ] 实现回调函数 `fsevent_callback`
- [ ] 实现事件标志转换 `flags_to_event_type`

### Step 2: 更新模块导出

文件：`crates/mfind-core/src/fs/mod.rs`

```rust
#[cfg(target_os = "macos")]
pub mod native_fsevents;

#[cfg(target_os = "macos")]
pub use native_fsevents::NativeFSEventsWatcher;
```

### Step 3: 更新 IndexEngine 集成

文件：`crates/mfind-core/src/index/engine.rs`

```rust
// 在 build 方法后启动后台监控
#[cfg(target_os = "macos")]
pub async fn start_monitoring(&mut self, paths: &[PathBuf]) -> Result<()> {
    use crate::fs::NativeFSEventsWatcher;

    let config = MonitorConfig::default();
    let (tx, mut rx) = flume::bounded(1000);

    let mut watcher = NativeFSEventsWatcher::new(config, tx)?;
    watcher.start(paths)?;

    // 后台处理事件
    tokio::spawn(async move {
        while let Ok(event) = rx.recv_async().await {
            // 处理单个事件或批量处理
            self.update(&[event]).await?;
        }
        Ok::<_, anyhow::Error>(())
    });

    Ok(())
}
```

### Step 4: CLI 集成

文件：`crates/mfind-cli/src/commands/search.rs`

```rust
// 搜索完成后启动后台监控（可选）
#[cfg(target_os = "macos")]
if self.watch {
    eprintln!("{} Starting filesystem monitor...", style("→").blue());
    engine.start_monitoring(&search_paths).await?;

    // 保持运行，等待事件
    tokio::signal::ctrl_c().await?;
    eprintln!("{} Stopping monitor...", style("→").blue());
}
```

---

## 性能对比

| 指标 | 轮询实现 | 原生 FSEvents | 改进 |
|------|----------|---------------|------|
| 事件延迟 | 100-1000ms | <10ms | 10-100x |
| CPU 占用 | 持续轮询 | 事件驱动 | 99% 降低 |
| 事件准确性 | 可能遗漏 | 内核保证 | 100% 可靠 |
| 递归监控 | ❌ 不支持 | ✅ 支持 | - |
| 事件类型 | 有限 | 完整 | - |

---

## 备选方案

如果原生 FSEvents 实现复杂度太高，可考虑以下备选方案：

### 方案 A: 使用 `notify` crate

```toml
[dependencies]
notify = "6"
```

```rust
use notify::{RecommendedWatcher, RecursiveMode, watcher};

let (tx, rx) = std::sync::mpsc::channel();
let mut watcher = watcher(tx)?;

watcher.watch(&path, RecursiveMode::Recursive)?;
```

**优点:** 跨平台，简单
**缺点:** 底层仍使用轮询 (macOS 上不是 FSEvents)

### 方案 B: 使用 `fsevent` crate

```toml
[dependencies]
fsevent = "0.4"
```

**优点:** 封装简单
**缺点:** 社区维护不活跃

---

## 推荐方案

**首选：原生 FSEvents API**

理由:
1. macOS 原生支持，性能最优
2. 事件延迟最低 (<10ms)
3. 内核级事件保证，不会遗漏
4. 符合项目"高性能"定位

---

## 验收标准

- [ ] 事件延迟 < 50ms
- [ ] CPU 占用 < 1% (空闲时)
- [ ] 支持递归监控子目录
- [ ] 正确区分 Create/Delete/Modify/Rename 事件
- [ ] 通过场景 4 测试验证

---

## 时间估算

| 任务 | 预计时间 |
|------|----------|
| 原生 FSEvents 模块实现 | 3-4 小时 |
| IndexEngine 集成 | 1-2 小时 |
| CLI 集成 | 1 小时 |
| 测试和调试 | 1-2 小时 |
| **总计** | **6-8 小时** |

---

## 相关文档

- [Apple FSEvents Reference](https://developer.apple.com/documentation/coreservices/file_system_events)
- [core-foundation-rs](https://github.com/servo/core-foundation-rs)
- [场景测试报告](./scenario_test_report.md)
