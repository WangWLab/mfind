# mfind 性能测试指南

本指南说明如何运行和管理 mfind 的性能基准测试。

## 前置准备

### 1. 生成测试数据

测试数据会生成在项目根目录的 `test_data/` 目录（已在 `.gitignore` 中排除）。

```bash
# 快速测试（100 文件）
./scripts/gen-test-data.sh --count 100

# 标准测试（1000 文件）
./scripts/gen-test-data.sh --count 1000

# 大规模测试（10000 文件）
./scripts/gen-test-data.sh --count 10000 --concurrency 500
```

### 2. 确认环境

```bash
# 使用 release 模式编译
cargo build --release

# 确认 criterion 已安装
cargo bench --version
```

## 运行性能测试

### 完整基准测试

```bash
# 运行所有基准测试（约 5-10 分钟）
cargo bench
```

### 运行特定测试组

```bash
# 只测试 FST 构建性能
cargo bench --bench search_bench -- 'fst_build'

# 只测试前缀搜索性能
cargo bench --bench search_bench -- 'prefix_search'

# 只测试正则搜索性能
cargo bench --bench search_bench -- 'regex_search'

# 只测试查询解析性能
cargo bench --bench search_bench -- 'query_parse'

# 只测试索引引擎构建（需要 test_data）
cargo bench --bench search_bench -- 'index_engine_build'

# 只测试搜索模式（需要 test_data）
cargo bench --bench search_bench -- 'search_'
```

### 运行对比测试

```bash
# 保存基准结果
cargo bench -- --save-baseline baseline

# 代码修改后运行对比
cargo bench -- --baseline baseline
```

## 测试规模说明

| 测试名称 | 数据规模 | 典型耗时 | 用途 |
|----------|----------|----------|------|
| `*_100` | 100 文件 | < 50µs | 快速验证，日常开发 |
| `*_1k` | 1,000 文件 | < 150µs | 小规模项目 |
| `*_10k` | 10,000 文件 | < 1.5ms | 中等规模项目 |
| `*_100k` | 100,000 文件 | < 15ms | 大规模项目基准 |

## 性能回归检测流程

### 发布前性能检查清单

```bash
# 1. 生成标准测试数据
./scripts/gen-test-data.sh --count 1000

# 2. 运行完整基准测试
cargo bench

# 3. 检查关键指标（应满足）：
#    - fst_build_100k:   < 20ms
#    - prefix_search_100k: < 10ms
#    - regex_search_100k:  < 15ms
```

### 性能退化定位

```bash
# 1. 保存当前基线
cargo bench -- --save-baseline before_change

# 2. 进行代码修改

# 3. 对比测试
cargo bench -- --baseline before_change

# 4. 如有退化，查看详细报告
cargo bench -- --baseline before_change --verbose
```

## 大规模压力测试

对于极端场景测试：

```bash
# 生成 100 万文件（需要约 10GB 磁盘空间）
./scripts/generate_1m_files.sh

# 测试超大索引构建
cargo bench --bench large_scale_bench
```

## 结果解读

### Criterion 输出格式

```
fst_build_100k
  time:   [11.366 ms 11.372 ms 11.379 ms]
  change: [+15.365% +15.939% +16.467%] (p = 0.00 < 0.05)
  Performance has regressed.
```

- **time**: 执行时间（最小值 - 平均值 - 最大值）
- **change**: 相比基线的变化百分比
- **p < 0.05**: 统计学显著性
- **回归/改进**: 自动判断性能变化方向

### 性能指标参考值

| 操作 | 100 文件 | 1K 文件 | 10K 文件 | 100K 文件 |
|------|----------|---------|----------|-----------|
| FST 构建 | ~33µs | ~136µs | ~1.15ms | ~11.4ms |
| 前缀搜索 | ~4.8µs | ~46µs | ~441µs | ~4.5ms |
| 正则搜索 | ~7.7µs | ~72µs | ~710µs | ~7.2ms |
| 查询解析 | ~30ns | - | - | - |

## 自动化集成

### CI/CD 中的性能测试

```yaml
# .github/workflows/bench.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Generate test data
        run: ./scripts/gen-test-data.sh --count 1000

      - name: Run benchmarks
        run: cargo bench
```

## 故障排查

### 常见问题

**Q: 测试数据目录不存在**
```bash
./scripts/gen-test-data.sh --count 1000
```

**Q: criterion 未安装**
```bash
cargo add criterion --dev
```

**Q: 测试结果不稳定**
- 关闭其他占用 CPU 的程序
- 增加采样次数：`cargo bench -- --sample-size 200`
- 延长测试时间：`cargo bench -- --warm-up-time 5`

**Q: 内存不足**
- 减少测试规模：`--count 100`
- 降低并发度：`--concurrency 50`

## 相关文件

- `tests/benchmarks/search_bench.rs` - 主要基准测试
- `tests/benchmarks/large_scale_bench.rs` - 大规模压力测试
- `scripts/gen-test-data/` - 测试数据生成器源码
- `scripts/gen-test-data.sh` - 测试数据生成包装脚本

## 维护

### 添加新基准测试

1. 在 `tests/benchmarks/search_bench.rs` 中添加测试函数
2. 使用 `c.bench_function()` 定义测试
3. 添加到 `criterion_group!` 宏中
4. 运行 `cargo bench` 验证

### 更新基线值

当性能优化或退化确认后，更新基线：

```bash
cargo bench -- --save-baseline new_baseline
```

---

**最后更新：** 2026-03-23
**文档版本：** 1.0
