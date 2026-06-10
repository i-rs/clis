# 存储层测试计划设计

## Goal

建立一套覆盖 6 个存储驱动的测试体系，包含：

1. **契约测试** — 每个存储后端的 ClawStorage trait 行为一致性验证
2. **对话脚本** — 基于真实 CLI 工具的端到端引导教程测试
3. **报告系统** — 输出可追溯的分析报告，指导 claw/claw-core 迭代改进

测试输出要提供 actionable 的洞察，而非简单的 pass/fail。

---

## 1. 契约测试架构

### 核心机制

定义一个 `contract!` 宏，将同一组断言注入每个存储后端：

```rust
// contracts/message_log_contract.rs
contract!("MessageLog", |store: &ClawStorage| {
    let mut msgs = make_test_messages(3);
    store.message_log.append_batch("s1", &msgs).await?;
    let loaded = store.message_log.load("s1", 100).await?;
    assert_eq!(loaded.len(), 3);

    let found = store.message_log.search("hello", 10).await?;
    assert!(found.len() >= 1);

    store.message_log.delete_session("s1").await?;
    let after = store.message_log.load("s1", 100).await?;
    assert!(after.is_empty());
})
```

### 后端注册

```rust
// backends/sqlite.rs
define_backend_tests!("sqlite", || {
    let tmp = tempfile::tempdir()?;
    let config = StorageConfig {
        backend: StorageBackend::Sqlite,
        sqlite_path: Some(tmp.path().join("test.db")),
        ..Default::default()
    };
    ClawStorage::sqlite(&config).await?
});
```

### 契约覆盖矩阵

| 契约文件 | 覆盖操作 | 断言重点 |
|----------|---------|----------|
| `session_contract` | CRUD、状态转换、count、message_count 更新 | 幂等 upsert，删除级联 |
| `message_log_contract` | append、load、search、delete_session、count | 搜索大小写不敏感，seq 顺序，尾部限制 |
| `memory_contract` | save、load、dirty 标记、flush | agent_id 隔离，序列化往返 |
| `stats_contract` | upsert_batch、read_range、prune | 去重（按 id），时间范围过滤 |
| `config_store_contract` | agent/providers/users/mcp/settings CRUD | ConfigStore 5 个 repo 全部 |
| `tool_cache_contract` | get、set、delete、agent 隔离 | HashMap&lt;String,String&gt; 序列化 |
| `skill_contract` | install、list、get、remove、executable | Markdown 文件内容，参数 schema |

### 设计约束

- 契约测试不启动 HTTP server，直接构造 `ClawStorage` 实例
- File 和 SQLite 使用 `tempfile::tempdir()` 隔离数据
- 外部 DB 通过 docker-compose 提供连接字符串
- 每个契约独立函数，`define_backend_tests!` 生成 `#[tokio::test]`

---

## 2. 对话脚本结构

每个 CLI 工具一个脚本目录，数字前缀 = 推荐学习顺序：

```
conversations/scenarios/
├── 01-kv/                  ← 简单键值存储
│   ├── README.md           ← 自然语言剧本（给人类阅读）
│   ├── script.json         ← 结构化定义（可执行）
│   └── report.json         ← 测试运行后自动生成
├── 02-todo/                ← 待办事项（特殊命令 done）
├── 03-weight/              ← 体重追踪（数值型 + stats + chart）
├── 04-water/               ← 喝水记录（今日汇总）
├── 05-sleep/               ← 睡眠追踪（质量评分 1-5 + stats）
├── 06-meal/                ← 饮食记录（meal_type + 可选 calories）
├── 07-mood/                ← 情绪记录（7 级枚举 + calendar 视图）
├── 08-sit/                 ← 久坐记录（自动计算 start/end）
├── 09-pig/                 ← 吃的念想（craving 记录）
├── 10-spark/               ← 灵感捕捉（content + source）
├── _template/              ← 新增工具的模板
│   ├── README.md
│   └── script.json
└── cross-tool/             ← 跨工具组合场景
    ├── daily-checkin/      ← "早间打卡"：mood + weight + water 一次完成
    └── health-summary/     ← "健康周报"：weight + sleep + meal + sit 汇总
```

### script.json 格式

```json
{
  "meta": {
    "tool": "i-rs-kv",
    "name": "kv_basic_usage",
    "description": "引导用户学习使用 KV 存储：增删改查",
    "required_capabilities": ["add", "get", "list", "delete", "search"],
    "storage_backends": ["file", "sqlite", "mysql", "postgres"],
    "tags": ["guided-tutorial", "kv"]
  },
  "steps": [
    {
      "step": 1,
      "title": "存储一条配置",
      "user_message": "帮我记一下，我的博客地址是 https://example.com",
      "expected_tool": "i-rs-kv",
      "expected_command": "add",
      "expected_args": {"KEY": "blog_url", "VALUE": "https://example.com"},
      "check_reply": {"contains": ["已记录", "blog_url"]},
      "verify_storage": {
        "file_check": "~/.i-rs/data/kv.json → entries.blog_url.value == 'https://example.com'",
        "expected_state": {"key": "blog_url", "value": "https://example.com"}
      }
    },
    {
      "step": 2,
      "title": "读取存储的值",
      "user_message": "我的博客地址是什么？",
      "expected_tool": "i-rs-kv",
      "expected_command": "get",
      "expected_args": {"KEY": "blog_url"},
      "verify_storage": {"no_new_records": true}
    },
    {
      "step": 3,
      "title": "列出所有键",
      "user_message": "帮我看看我存了哪些东西",
      "expected_tool": "i-rs-kv",
      "expected_command": "list",
      "verify_storage": {"no_new_records": true}
    },
    {
      "step": 4,
      "title": "删除一条记录",
      "user_message": "把 blog_url 删掉",
      "expected_tool": "i-rs-kv",
      "expected_command": "delete",
      "expected_args": {"KEY": "blog_url"},
      "check_reply": {"contains": ["已删除", "blog_url"]},
      "verify_storage": {
        "file_check": "~/.i-rs/data/kv.json → entries 不含 blog_url"
      }
    },
    {
      "step": 5,
      "title": "带标签的记录",
      "user_message": "记一下 API 密钥 sk-test，标签是 test，备注是 测试用",
      "expected_tool": "i-rs-kv",
      "expected_command": "add",
      "expected_args": {"KEY": "api_key_test", "VALUE": "sk-test"},
      "expected_flags": {"--tag": "test", "--remark": "测试用"},
      "verify_storage": {
        "expected_state": {
          "key": "api_key_test",
          "tags": ["test"],
          "remark": ["测试用"]
        }
      }
    }
  ]
}
```

### 核心原则

1. 每个步骤的 CLI 命令必须有依据（来自真实 CLI 的 subcommand + args）
2. 验证点检查实际存储状态，不只是 API 响应
3. 脚本按"新手引导"顺序排列：简单→复杂
4. `expected_args` 只检查必要参数，不要求精确匹配
5. 要留好扩展接品——后续 70+ 工具可复制 `_template/` 快速创建

---

## 3. 对话脚本执行引擎

### 模块结构

```
runner/
├── mod.rs               — ScriptRunner: 加载 → 执行 → 断言 → 报告
├── executor.rs          — 执行每一步
│   ├── send_message()   — POST /api/chat/stream → 解析 SSE
│   └── extract_calls()  — 从回复中提取工具调用信息
├── verifier.rs          — 多路径存储验证
│   ├── verify_storage() — 根据 verify_storage 断言
│   ├── check_file()     — 直接读 JSON 文件验证
│   ├── check_sql()      — SQL 查询验证
│   └── check_trait()    — 通过 ClawStorage trait 验证
└── reporter.rs          — 生成 Markdown + JSON 报告
```

### 执行模式

| 模式 | 路径 | 速度 | 用途 |
|------|------|------|------|
| 直接模式 | 直连 AppCore，不走 HTTP | ~毫秒/步 | 快速迭代脚本 |
| HTTP 模式 | 启动真实 server，走完整 API 栈 | ~秒/步 | 上线前完整验证 |

### 存储验证多路径策略

```
verify_storage(step, backend_type, claw_storage)
  ├── 路径 1: ClawStorage trait（推荐）
  │   └── storage.sessions.get_one(), storage.message_log.count(), etc.
  ├── 路径 2: 直接读文件（仅 File 后端）
  │   └── 解析 ~/.i-rs/data/kv.json 检查 entries
  └── 路径 3: SQL 查询（仅 SQL 后端）
      └── SELECT * FROM sessions WHERE id = ?
```

---

## 4. 对话场景设计原则

### 按 CLI 工具分组

每个工具的场景覆盖：

| 场景类别 | 覆盖内容 | 示例 |
|----------|---------|------|
| 基础 CRUD | add → list → get → update → delete | kv, spark |
| 数值型 | add 数值 → list stats → chart | weight, sleep |
| 特殊命令 | done, stats, calendar | todo, mood |
| 标签系统 | --tag / --remark 组合 | 全部工具 |
| 跨会话 | 数据持久性验证 | 全部工具 |
| 参数推导 | agent 能否正确推导默认参数 | date=today, time=now |

### 跨工具场景（可选）

```
cross-tool/
├── daily-checkin/       ← "早间打卡"：同时记录 mood + weight + water
└── health-summary/      ← "健康周报"：汇总 weight + sleep + meal + sit
```

跨工具场景用于验证 agent 在复杂上下文中的工具选择能力。

---

## 5. docker-compose & CI 策略

### docker-compose.yml

```yaml
services:
  mysql:
    image: mysql:8
    environment:
      MYSQL_ROOT_PASSWORD: test
      MYSQL_DATABASE: claw_test
    ports: ["3306:3306"]

  postgres:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: test
      POSTGRES_DB: claw_test
    ports: ["5432:5432"]

  mongo:
    image: mongo:7
    ports: ["27017:27017"]

  redis:
    image: redis:7
    ports: ["6379:6379"]
```

### CI 策略

| 触发条件 | 运行内容 | 耗时 |
|----------|---------|------|
| 每次 push | File + SQLite 契约 + SQLite 对话 | ~2min |
| PR 标记 `ci:full-storage` | MySQL/PG/Mongo/Redis 全部 | ~8min |
| 定时（每日） | 全量测试 + 报告对比 | ~10min |

---

## 6. 报告系统

### 报告生成

每次测试运行生成：
- `reports/<timestamp>/summary.md` — 人类可读
- `reports/<timestamp>/summary.json` — 机器可读

### 报告章节

**1. 契约测试结果** — 驱动 vs 契约矩阵
```
| 驱动 | 会话 | 消息日志 | 记忆 | 统计 | 配置 | 工具缓存 | 技能 |
```

**2. 对话脚本结果** — 每个工具 pass/fail/partial
```
| # | 工具 | pass/fail | 调用正确 | 存储一致 | 备注 |
```

**3. Agent 意图识别分析** — 每步偏差明细
```
工具: weight
步骤 3: "记录一下今天早上空腹体重 74.8，tag 为 morning"
  → 期望: add 74.8 --tag morning --date $(today)
  → 实际: add 74.8 --tag morning
  → 偏差: 未传 --date（默认为 today，功能正确）
  → 改进建议: 无阻塞偏差
```

**4. Token 使用统计**
```
| 工具 | 脚本步骤数 | 总 prompt | 总 completion | 总 cost | 平均轮数 |
```

**5. 系统提示演化追踪** — 每次 tool call 后 system prompt 大小
```
消息 1: TOOL_INDEX=[8 tools], USER_MEMORY=空, SKILLS=空  → 2,430 tokens
消息 2: TOOL_INDEX=[8 tools], USER_MEMORY=有(体重记录)   → 2,580 tokens
消息 3: TOOL_INDEX=[8 tools], USER_MEMORY=有(体重+偏好)  → 2,720 tokens
```

**6. 驱动性能对比**
```
| 操作 | File | SQLite | MySQL | PG |
```

### 改进工作流

```
运行测试 → 生成报告 → git diff 发现偏差
   ↓
修改 claw/claw-core（tool descriptions / system prompt / storage impl）
   ↓
重新运行 → 报告对比（偏差减少？新问题？）
   ↓
满意后提交
```

---

## 7. 实现顺序

| 阶段 | 内容 | 预估 |
|------|------|------|
| **P0** | 契约宏 + File/SQLite 契约测试 | 2-3 天 |
| **P0** | docker-compose + MySQL/PG 契约 | 1 天 |
| **P0** | 报告系统基础（Markdown + JSON 输出） | 1 天 |
| **P1** | 对话脚本引擎（直接模式） | 2 天 |
| **P1** | 10 个 CLI 引导脚本（kv→spark） | 3 天 |
| **P1** | Agent 意图分析 + 系统提示演化报告 | 1 天 |
| **P2** | Mongo/Redis 契约 | 1 天 |
| **P2** | 对话脚本 HTTP 模式 + 跨工具场景 | 1 天 |
| **P2** | 驱动性能对比报告 | 1 天 |
| **P3** | 扩展至 70 个 CLI 脚本 | 持续 |

---

## 8. 目录结构（完整）

```
crates/claw-core-storage-tests/
├── Cargo.toml
├── src/
│   └── lib.rs                      ← 契约宏 + 辅助函数
├── contracts/                      ← 契约定义
│   ├── mod.rs
│   ├── session_contract.rs
│   ├── message_log_contract.rs
│   ├── memory_contract.rs
│   ├── stats_contract.rs
│   ├── config_store_contract.rs
│   ├── tool_cache_contract.rs
│   └── skill_contract.rs
├── backends/                       ← 后端注册
│   ├── mod.rs
│   ├── file.rs
│   ├── sqlite.rs
│   └── external.rs                 ← mysql / pg / mongo / redis
├── conversations/
│   ├── scenarios/                  ← 对话脚本
│   │   ├── 01-kv/
│   │   ├── 02-todo/
│   │   ├── ... → 10-spark/
│   │   ├── _template/
│   │   └── cross-tool/
│   │       ├── daily-checkin/
│   │       └── health-summary/
│   └── runner/                     ← 执行引擎
│       ├── mod.rs
│       ├── executor.rs
│       ├── verifier.rs
│       └── reporter.rs
├── tests/                          ← 集成测试入口
│   ├── contracts_file.rs
│   ├── contracts_sqlite.rs
│   └── contracts_external.rs
├── reports/                        ← 运行时生成（gitignore）
│   └── latest -> <timestamp>/      ← 最新报告软链接
├── observability/                  ← 存储巡检
│   ├── storage_inspector.rs
│   └── token_analyzer.rs
└── docker-compose.yml
```
