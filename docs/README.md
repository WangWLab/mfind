# mfind 文档导航

本目录用于统一 mfind 的项目文档，减少需求、进展、架构和专题方案之间的重复描述。

## 建议阅读顺序

1. `../README.md`
   - 对外项目说明、快速开始、当前能力概览
2. `../PROJECT_SUMMARY.md`
   - 当前项目状态、模块组成、已落地能力、下一阶段重点
3. `../requirements.md`
   - 产品目标、需求分级、技术演进方向、长期能力规划
4. `../MILESTONES.md`
   - 里程碑完成情况、已完成阶段、待推进事项
5. `architecture.md`
   - 核心模块和系统分层
6. `development.md`
   - 本地开发、测试、代码组织

## 专题文档

- `market-research.md`
  - 市场与竞品分析，解释为什么要做 Spotlight 独立的文件搜索工具
- `everything-principle.md`
  - Everything 的技术原理与可借鉴点
- `fsevents_solution.md`
  - FSEvents 从轮询到原生事件驱动的演进方案
- `scenario_test_report.md`
  - 场景测试记录，保留历史验证过程

## 文档分工

- `README.md` 只回答“这个项目是什么、现在能做什么、怎么上手”
- `PROJECT_SUMMARY.md` 只回答“项目当前处于什么阶段、有哪些模块、下一步是什么”
- `requirements.md` 只回答“产品应该发展到哪里、需求如何分级、技术路线是什么”
- `MILESTONES.md` 只回答“哪些阶段已完成、哪些里程碑仍在进行”
- `docs/*.md` 只放专题说明或开发资料

## 当前整理原则

- 尽量避免在多个文件里重复维护同一份状态表
- 历史性文档保留，但不再作为项目现状的唯一来源
- 当前状态以 `PROJECT_SUMMARY.md` 和 `MILESTONES.md` 为准
