# 一个人的数据指挥中心，70 个开源 CLI，等你加入

&gt; 从体重记录到域名到期提醒，从宠物喂食到投资管理——你的生活数据，值得一个统一的命令行入口。

## 你在用什么管理这些？

打开你的手机，数一数有多少个 App 在做记录这件事：

- **健康数据** — Apple Health、Garmin、智能手表
- **财务记账** — 随手记、MoneyWiz、Excel
- **待办事项** — Things、Todoist、Notion
- **密码管理** — 1Password、Bitwarden
- **习惯追踪** — Streaks、Habitica
- **阅读记录** — 豆瓣、Goodreads
- **家居维护** — 全靠脑子和便签

每一个都很好用。但你的数据被锁在这些 App 的孤岛里，互不相通。

**i-rs 换了一种思路：不做另一个 App，做 70 个 CLI 工具，覆盖你生活的所有可数据化维度。**

## 为什么用终端管理生活数据？

听起来复古？实际上这是效率的极致：

```bash
# 记录今天的体重（2 秒）
i-rs-weight add --weight 72.5

# 查看本月支出（1 秒）
i-rs-ledger list --month 2026-05

# 检查宠物喂食记录（1 秒）
i-rs-feedpet list

# 统计跑步计划执行情况（1 秒）
i-rs-run stats
```

- **键盘驱动** — 不用找图标、不用点菜单、不用等加载
- **脚本化** — alias、cron、自动化，一行的组合
- **极速** — 每个工具几百 KB，毫秒级启动
- **远程友好** — SSH 会话中完美工作

## 70 个工具，覆盖哪些领域？

```
健康：weight / height / sleep / mood / exercise / run
       cycling / water / step / dose / fast / allergy
       cal / habit / cycle / sit / vision

财务：ledger / budget / invest / debt / recur / invoice
      tax / goal / sub / kv / keys

家庭：appliance / sheet / toothbrush / towel / bed
      filter / purify / ac / plant / aqua

宠物：feedpet / petbath / walkdog

学习：read / vocab / article / spark / snippet / quote

社交：contact / birthday / gift

娱乐：movie / podcast / want

时间：remind / event / time / tick / deploy

其他：domain / bestby / todo / project / grocery
      meal / pig / car / sit / password / note
      bookmark / server / ...
```

70 个，每一个都可以独立安装、独立使用。

## 数据主权

所有数据存在本地 `~/.config/i-rs/*.json`：

```bash
# 全量备份
cp -r ~/.config/i-rs ~/backups/i-rs-$(date +%Y%m%d)

# 用 jq 分析
cat ~/.config/i-rs/weight.json | jq '.entries | length'

# 用 Python 分析
i-rs-weight list --json | python3 analysis.py
```

纯 JSON。不锁定在任何平台，任何语言都能读。

## 关于这个项目

i-rs 是一个开源项目，**AGPL-3.0 协议**，一个人从头写到尾。代码质量严格：`cargo check` 保持 0 errors 0 warnings。

**但 70 个工具有一个"量"的问题——我一个人覆盖不了所有功能。**

## 这里有一些你可能会感兴趣的方向

### 如果你会用 Rust

- **加一个新的 CLI 工具** — 模板化结构，按 checklist 走一遍，万行内搞定
- **优化已有工具** — 有未实现的命令需要补全。每个工具的 `data` 命令（export / import / clear）已经通过 `data_command!` 宏统一了，更多可以宏化的模式等待发现
- **i-rs-api 打磨** — REST API 功能需要更多端点和更好的错误处理
- **性能优化** — 启动速度、内存占用、JSON 解析

### 如果你会用 AI / LLM

- **skill teach 改进** — AI 教学文档可以做得更好，加入更多上下文关联
- **Agent 集成案例** — 写一些 Claude Code / Cursor 使用 i-rs 的教程
- **自然语言转命令** — 一个简单的 LLM wrapper，把"记录今天体重 72.5"转成 `i-rs-weight add --weight 72.5`

### 如果你想贡献非代码内容

- **文档** — 每个工具都需要更好的 README 和示例
- **文章 / 教程** — 推广、用例、最佳实践
- **第三方集成** — Alfred workflow、Raycast extension、macOS Shortcuts

## 快速开始

```bash
# 安装一个工具试试
brew install i-rs/tap/i-rs-kv

# 或者直接从 GitHub 下载二进制
# 或者 cargo install i-rs-kv

# 试试看
i-rs-kv add hello world
i-rs-kv get hello
i-rs-kv skill info
```

---

**[GitHub: i-rs/clis](https://github.com/i-rs/clis)**

一个人的项目，等着变成很多人的项目。
