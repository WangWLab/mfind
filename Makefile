# 性能测试 Makefile
# 用法：make bench / make bench-quick / make bench-save

.PHONY: bench bench-quick bench-save bench-compare bench-report help

# 默认目标
all: bench

# 生成测试数据
test-data:
	@echo "生成测试数据..."
	@./scripts/gen-test-data.sh --count 1000

# 完整基准测试
bench: test-data
	@echo ""
	@echo "========================================"
	@echo "  运行完整基准测试"
	@echo "========================================"
	@echo ""
	cargo bench

# 快速基准测试（仅 100/1k 规模）
bench-quick:
	@echo ""
	@echo "========================================"
	@echo "  快速基准测试（100/1k 规模）"
	@echo "========================================"
	@echo ""
	cargo bench --bench search_bench -- 'fst_build_100$$'
	cargo bench --bench search_bench -- 'fst_build_1k$$'
	cargo bench --bench search_bench -- 'prefix_search_100$$'
	cargo bench --bench search_bench -- 'prefix_search_1k$$'

# 特定规模测试
bench-100:
	cargo bench --bench search_bench -- '_100$$'

bench-1k:
	cargo bench --bench search_bench -- '_1k$$'

bench-10k:
	cargo bench --bench search_bench -- '_10k$$'

bench-100k:
	cargo bench --bench search_bench -- '_100k$$'

# 特定测试类型
bench-fst:
	cargo bench --bench search_bench -- 'fst_build'

bench-prefix:
	cargo bench --bench search_bench -- 'prefix_search'

bench-regex:
	cargo bench --bench search_bench -- 'regex_search'

bench-parse:
	cargo bench --bench search_bench -- 'query_parse'

bench-index: test-data
	cargo bench --bench search_bench -- 'index_engine'

bench-search: test-data
	cargo bench --bench search_bench -- 'search_'

# 保存基线
bench-save:
	@echo "保存基线..."
	@mkdir -p .benchmarks
	cargo bench --bench search_bench -- --save-baseline .benchmarks/latest

# 对比基线
bench-compare:
	@echo "与基线对比..."
	cargo bench --bench search_bench -- --baseline .benchmarks/latest

# 保存具名基线
bench-save-name:
	cargo bench --bench search_bench -- --save-baseline .benchmarks/$(name)

# 对比具名基线
bench-compare-name:
	cargo bench --bench search_bench -- --baseline .benchmarks/$(name)

# 生成报告
bench-report:
	@echo "生成性能报告..."
	@mkdir -p .benchmarks
	@echo "# mfind 性能测试报告" > .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "" >> .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "**时间:** $$(date '+%Y-%m-%d %H:%M:%S')" >> .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "" >> .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "## 测试结果" >> .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "" >> .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "\`\`\`" >> .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@cargo bench 2>&1 | tee -a .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "" >> .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "\`\`\`" >> .benchmarks/report_$$(date +%Y%m%d_%H%M%S).md
	@echo "报告已生成"

# 清理
bench-clean:
	rm -rf .benchmarks
	rm -rf test_data

# 帮助
help:
	@echo "mfind 性能测试 Makefile"
	@echo ""
	@echo "用法：make [目标]"
	@echo ""
	@echo "主要目标:"
	@echo "  bench          完整基准测试"
	@echo "  bench-quick    快速测试（100/1k 规模）"
	@echo "  bench-save     保存当前基线"
	@echo "  bench-compare  与基线对比"
	@echo ""
	@echo "按规模测试:"
	@echo "  bench-100      100 文件规模"
	@echo "  bench-1k       1000 文件规模"
	@echo "  bench-10k      10000 文件规模"
	@echo "  bench-100k     100000 文件规模"
	@echo ""
	@echo "按类型测试:"
	@echo "  bench-fst      FST 构建测试"
	@echo "  bench-prefix   前缀搜索测试"
	@echo "  bench-regex    正则搜索测试"
	@echo "  bench-parse    查询解析测试"
	@echo "  bench-index    索引引擎测试"
	@echo "  bench-search   搜索模式测试"
	@echo ""
	@echo "其他:"
	@echo "  help           显示帮助"
	@echo "  bench-clean    清理基准数据"
