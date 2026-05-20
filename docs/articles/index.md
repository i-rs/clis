# 文章

> 从技术到理念，从架构到生态 — 记录 i-rs CLI 工具集的探索与实践。

## 精选文章

### [70 个 CLI 工具，全都能教 AI 自己怎么用](./promo-ai-native-cli)（推广）

> 一个 Rust 宏，70 个工具，每个都内置了 AI 技能系统。适合发在掘金、思否、dev.to、Medium 等技术社区。

### [i-rs CLI 项目状态报告](./project-status)

> 70 个工具、114 次提交、0 个编译警告。一份完整的项目全景报告，展示当前进展、架构和未来方向。

### [70 个 CLI 工具，我是怎么用 Rust 管理过来的](./promo-rust-mono-repo)（推广）

> 一个 Cargo 工作空间，70 个 crate，全部用宏生成。适合 Rust 技术社群分享。

### [一个人的数据指挥中心，70 个开源 CLI，等你加入](./promo-community)（推广）

> 从健康到财务，从宠物到学习——你的生活数据，值得一个统一的命令行入口。适合开源社区、产品爱好者分享。

### [i-rs-claw — 你的 AI 终端数据助理](./i-rs-claw-intro)

> 70 个 CLI 工具，一个自然语言入口。i-rs-claw 把终端变成了能听懂人话的 AI 数据助理。适合所有对 AI + CLI 结合感兴趣的人阅读。

### [i-rs-claw 架构解析 — AI 终端助理的内部结构](./i-rs-claw-architecture)

> 从 Ratatui 事件循环到 skill teach 管道，从 LLM 集成到工具调度系统——一份完整的架构说明书。适合开发者、架构师。

### [与 AI 对话的十种姿势 — i-rs 日常工具的智能体交互话术](./agent-interaction-scripts)

> 10 个工具，10 套话术：从体重记录到心情捕捉，从饮食追踪到灵感管理——看 AI 如何用自然语言驾驭 CLI 工具。适合所有对 AI + CLI 交互设计感兴趣的人。

---

### [同一份数据，两个入口 — CLI 与 API 的共享存储层](./shared-storage-layer)

深入 i-rs 最独特的架构设计：CLI 和 REST API 读写同一个 JSON 文件。拆解 `Storage<T>`、`SharedStore<T>`、`make_app_tools!` 三层架构，看 70 个工具如何在不依赖数据库的情况下实现数据统一。

### [i-rs 内置 Skill 系统 — CLI 工具的 AI 自我介绍](./built-in-skill-system)

全面解读 i-rs 最具特色的能力：一个宏注入 70+ 工具，7 个 skill 子命令覆盖教学、检索、分发全流程，编译时嵌入、AgentSkills 标准兼容、从 `--help` 就开始的 AI 体验。

### [AI 原生命令行 — i-rs 的 AI Skill 系统](./ai-native-cli)

当 CLI 工具学会"自我介绍"，AI 与终端之间的鸿沟被彻底打通。每一把 i-rs 工具都内置了完整的 AI 技能系统，无需外部配置，无需额外文档，AI Agent 读取 `--help` 的那一刻，就知道如何驾驭它。

### [70 工具 · 一柄利刃 — Rust 单体仓库工程实践](./rust-mono-repo)

从体重记录到投资管理，从宠物喂食到域名到期提醒 — 70 个 CLI 工具共享同一套架构、同一个核心库、同一种开发体验。这不是脚本的堆砌，而是一场 Rust 宏工程化的极致实践。

### [从终端到云端 — CLI + API + 浏览器的全栈覆盖](./full-stack-coverage)

同一份数据，三种交互方式：终端的极速响应、浏览器的可视化操作、REST API 的程序化集成。i-rs 不是一组工具，而是一个完整的数据生态。

### [一个人的数据指挥中心 — 全领域覆盖](./data-command-center)

健康、财务、家庭、宠物、学习、创作、投资、收藏…70 个领域的数据汇聚在同一个终端界面下。每个人的生活都值得被精确记录，这是属于你的命令行数据指挥中心。

### [claw MCP 接入指南 — 让 AI 终端助理获得无限工具拓展能力](./mcp-integration-guide)

从配置到插件、从 stdio 到 SSE、从 rmcp SDK 到 ClawTool 适配器——一份完整的 claw MCP 集成说明书。适合想要为 claw 接入外部工具的开发者。
