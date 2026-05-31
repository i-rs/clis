# CLI Tools 统一 Trait + 存储抽象 设计方案

## 1. 现状分析

### 1.1 70 个 CLI tool 的公共模式

每个 `i-rs-{name}` crate 结构几乎一致：

```
i-rs-{name}/
├── src/
│   ├── main.rs                # Cli::parse() → run() → match command
│   ├── models/mod.rs          # Entity + Store(BTreeMap) + Row(tabled) + ListItem(json)
│   ├── storage/mod.rs         # i_rs_core::create_store!(XxxStore, "xxx");
│   ├── commands/
│   │   ├── mod.rs             # re-export handles
│   │   ├── add.rs             # load_store → add_entry → save_store
│   │   ├── delete.rs          # load_store → remove_entry → save_store
│   │   ├── get.rs             # load_store → get_entry
│   │   ├── list.rs            # load_store → filter → output_list/format_table
│   │   ├── update.rs          # load_store → get_entry_mut → save_store
│   │   ├── example.rs         # i_rs_core::example_command!("...");
│   │   ├── skill.rs           # i_rs_core::skill_command!("i-rs-xxx");
│   │   └── data.rs            # i_rs_core::data_command!();
│   ├── presentation/
│   │   └── mod.rs             # i_rs_core::presentation!(XxxRow, "records");
│   └── service/
│       └── mod.rs             # list_xxx(store, ...) — 简单过滤逻辑
├── Cargo.toml                 # deps: clap, serde, chrono, anyhow, i-rs-core, tabled, owo-colors
└── README.md
```

**关键问题：**

| 问题 | 说明 |
|------|------|
| **大量样板代码** | main.rs、commands/ 的 add/delete/get/update/list 逻辑每个 crate 都重复 |
| **结构不统一** | 有些 crate 用 `key` 做主键（kv），有些用 `id`（weight），有些用 `name`（habit） |
| **特殊命令无约束** | stats/chart/calendar/checkin/streak 等特殊命令完全自由，claw 的 LLM 无法自动发现 |
| **claw 适配靠 CLI 子进程** | claw 通过 `i_rs` tool 调 `std::process::Command("i-rs-{tool}")`，无类型安全 |
| **工具发现靠外部缓存** | `discover_i_rs_tools()` 调 `i-rs-{name} skill summary` 获取描述，缓存到文件 |

### 1.2 数据模型差异分类

70 个 tool 可按主键和数据特征分类：

| 类别 | 工具示例 | 主键类型 | 特殊命令 |
|------|---------|---------|---------|
| **简单 CRUD** | kv, note, bookmark, quote, snippet, want, keys | String (key/id) | - |
| **日期驱动** | weight, mood, sleep, water, exercise, height, step, sit, fast, cal | UUID + date | stats, chart |
| **状态跟踪** | todo (done), remind (done), habit (checkin/streak), goal (deposit/milestone), project (milestone) | UUID/name | 状态变更 |
| **事件日历** | event, birthday, domain, bestby, sub, cycle, allergy, dose, movie, podcast | UUID + date | calendar, stats |
| **关联关系** | ledger (double-entry), debt (pay), invoice (stats), invest (stats), budget (expense/stats) | UUID + 复杂逻辑 | 统计计算 |
| **维护周期** | ac, filter, purify, sheet, toothbrush, towel, bed, appliance, plant, aqua, petbath, walkdog, feedpet | UUID + date | replace/maintain |
| **车辆管理** | car (fuel/maintain/stats) | UUID | 子资源 |
| **时间追踪** | time (start/stop/report), tick (duration), deploy (rollback) | UUID + timer | 计时逻辑 |

### 1.3 存储层现状

```
i_rs_core::Storage<T>
├── 文件: ~/.i-rs/data/{filename}.json
├── 操作: load() → 全量反序列化, save() → 全量序列化
├── 锁: fs2 (file advisory lock)
├── 支持: CONFIG_DIR 环境变量覆盖
└── 宏: create_store!(XxxStore, "xxx") → load_store/save_store/export_data/import_data/clear_data
```

**i-rs-api 服务器**：
- `make_app_tools!` 宏生成 `AppState`（每个 tool 一个 `SharedStore<T>`）
- `SharedStore<T>` 也是基于文件的，用 `Arc<RwLock<T>>`
- 提供 REST 端点：`GET/POST/DELETE/PATCH /api/{tool}/{id}`

**claw 的 i_rs tool**：
- 通过 `std::process::Command("i-rs-{tool}")` 调 CLI 子进程
- 解析 stdout 文本输出（非结构化）
- 不直接访问存储层

### 1.4 List 全量加载问题

```rust
// 典型的 list.rs — 每个 tool 都这样
pub fn handle_list(...) -> Result<()> {
    let store = crate::storage::load_store()?;  // ← 全量读入内存
    let records = crate::service::list_xxx(&store, days)?;  // ← 内存过滤
    // format & output
}
```

问题：
- 数据量大时（如 weight 积累数年、habit 大量 checkin）内存开销大
- 没有分页/偏移机制
- JSON 反序列化整个文件 O(n) 每次请求都做

---

## 2. 设计方案

### 2.1 CLI Tool Trait — `IrsTool`

在 `i-rs-core` 中定义一个 trait，所有 CLI tool 的 Store 实现它：

```rust
/// 所有 i-rs CLI 工具必须实现的统一接口。
pub trait IrsTool: Serialize + DeserializeOwned + Default + Send + Sync + 'static {
    /// 工具名称（不含 i-rs- 前缀）
    fn tool_name() -> &'static str;

    /// 实体类型
    type Entity: Serialize + DeserializeOwned + Clone;

    /// 表格行类型（用于 `tabled` 渲染）
    type Row: Tabled + for<'a> From<&'a Self::Entity>;

    /// JSON 列表项类型
    type ListItem: Serialize + for<'a> From<&'a Self::Entity>;

    /// 存储文件名
    fn filename() -> &'static str { Self::tool_name() }

    // ── CRUD 操作（默认实现即可覆盖大部分 tool） ──

    /// 返回 entries 的可变引用（BTreeMap key → Entity）
    fn entries(&self) -> &BTreeMap<String, Self::Entity>;
    fn entries_mut(&mut self) -> &mut BTreeMap<String, Self::Entity>;

    /// 主键生成策略
    fn generate_id(&self) -> String { uuid::Uuid::new_v4().to_string() }
    fn id_field(entity: &Self::Entity) -> &str;  // 从 entity 提取主键

    // ── 可选扩展 ──

    /// 是否支持特殊命令
    fn special_commands() -> Vec<SpecialCommand> { vec![] }

    /// 统计信息
    fn stats(&self) -> Option<Value> { None }
}

/// CLI tool 可能有的特殊命令。
pub enum SpecialCommand {
    /// 日期范围过滤（weight/mood/sleep...）
    DateRange { key: &'static str },
    /// 图表渲染（weight/sleep... 的 ASCII chart）
    Chart,
    /// 日历视图（mood 的月度日历）
    Calendar,
    /// 打卡 + 连续天数（habit 的 checkin streak）
    Checkin { streak: bool },
    /// 完成标记（todo/remind 的 done）
    Done,
}
```

**对 claw 的好处**：
- `i_rs` tool 不再调 `i-rs-{name} skill teach` 解析文本 → 直接读 trait 方法的类型信息
- LLM 参数 schema 自动从 trait 生成（不再需要外部 SKILL.md 解析）
- `discover_i_rs_tools()` 可改为加载 crate（link-time discovery）而非子进程调用

### 2.2 命令生成宏 — `define_cli_commands!`

基于 trait 自动生成 90% 的样板代码：

```rust
// 在 i-rs-weight/src/main.rs 中：
i_rs_core::define_cli_tool!(
    tool: WeightStore,
    entity: WeightRecord,
    row: WeightRow,
    list_item: ListItem,
    filename: "weights",
    // 自定义参数（超出口令）
    add_args: {
        weight: f64 => "体重值(kg)",
        date: Option<String> => "日期(YYYY-MM-DD)",
    },
    update_args: {
        weight: Option<f64> => "体重值(kg)",
    },
    list_args: {
        days: Option<usize> => "最近N天",
        chart: bool => "显示图表",
        stats: bool => "显示统计",
    },
    special: [DateRange, Chart],
);
```

这个宏展开为：
- `struct Cli` + `enum Commands`（完整 CLI 定义）
- `fn main()` 
- `fn run()`
- `handle_add/handle_delete/handle_get/handle_list/handle_update`
- `commands/example.rs`, `commands/skill.rs`, `commands/data.rs`
- `presentation/mod.rs`
- `storage/mod.rs`

→ **main.rs 从 ~120 行缩减到 ~30 行宏调用**，且保证所有 tool 结构一致。

### 2.3 List 分页/流式方案

```rust
pub trait IrsTool {
    // ... 现有方法 ...

    /// 分页查询（默认实现：全量加载后切片）
    fn list_paginated(
        &self,
        filter: &ListFilter,
        page: Pagination,
    ) -> Vec<&Self::Entity> {
        let all: Vec<&Self::Entity> = self.entries().values().collect();
        // ... filter ...
        all.into_iter().skip(page.offset).take(page.limit).collect()
    }
}

pub struct Pagination {
    pub offset: usize,
    pub limit: usize,  // 0 = no limit
}

pub struct ListFilter {
    pub days: Option<usize>,
    pub tag: Option<String>,
    pub keyword: Option<String>,
}
```

**CLI 层面**：`list` 命令加 `--limit` 和 `--offset` 参数。

**存储后端优化**：当使用 SQL 后端时，`list_paginated` 可以 override 为真正的 SQL `LIMIT/OFFSET`，避免全量加载。

### 2.4 CLI 存储层抽象

```
                    ┌──────────────────────────┐
                    │     IrsTool trait         │
                    │  (BTreeMap<String, E>)    │
                    └──────────┬───────────────┘
                               │ uses
                    ┌──────────┴───────────────┐
                    │   StorageBackend trait    │
                    │   (file / sql / api)      │
                    └──────────┬───────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          ▼                    ▼                    ▼
    FileBackend           SqlBackend           ApiBackend
    (~/.i-rs/data/*.json) (sqlite/mysql/pg)   (i-rs-api HTTP)
```

```rust
/// 存储后端抽象 — CLI tool 不直接读写文件，通过此 trait 操作。
pub trait StorageBackend: Send + Sync {
    /// 加载整个 Store
    fn load<T: DeserializeOwned>(&self, filename: &str) -> anyhow::Result<T>;
    /// 保存整个 Store
    fn save<T: Serialize>(&self, filename: &str, data: &T) -> anyhow::Result<()>;
    /// 导出 JSON
    fn export_json(&self, filename: &str) -> anyhow::Result<String>;
    /// 导入 JSON
    fn import_json(&self, filename: &str, input: &str) -> anyhow::Result<()>;
    /// 清空数据
    fn clear(&self, filename: &str) -> anyhow::Result<()>;

    // ── 可选高级查询 ──
    /// 分页查询（默认回退到全量加载）
    fn query_paginated<T: DeserializeOwned + IrsTool>(
        &self,
        filename: &str,
        filter: &ListFilter,
        page: Pagination,
    ) -> anyhow::Result<Vec<T::Entity>> {
        // 默认：全量加载后过滤切片
        let store: T = self.load(filename)?;
        // ... filter & slice ...
        Ok(vec![])
    }
}
```

**后端实现**：

| 后端 | 配置 | 适用场景 |
|------|------|---------|
| `FileBackend` | 默认，零配置 | 单机个人使用 |
| `SqliteBackend` | `i_rs_core` 加 sqlx feature | 数据量大，需要查询 |
| `ApiBackend` | `api_url = "http://localhost:3000"` | 多端共享，通过 i-rs-api 中转 |
| `MySqlBackend` / `PgBackend` | sqlx URL | 生产环境 |

**配置示例**（`~/.i-rs/config.toml`）：

```toml
[storage]
backend = "file"       # file | sqlite | api | mysql | postgres

[storage.sqlite]
path = "~/.i-rs/data/i-rs.db"

[storage.api]
base_url = "http://localhost:3000"
token = "your-api-token"
```

### 2.5 claw 工具发现的改进

当前：
```
claw 启动 → discover_i_rs_tools() → 对每个 tool 调子进程 "i-rs-xxx skill summary" → 缓存
```

改进后（两种方案）：

**方案 A — 编译时链接**（推荐）：
```toml
# claw Cargo.toml
[dependencies]
i-rs-weight = { path = "../clis/i-rs-weight" }
i-rs-mood = { path = "../clis/i-rs-mood" }
# ... 所有需要的 tool
```

```rust
// claw 启动时
let tools: Vec<Box<dyn IrsTool>> = vec![
    Box::new(i_rs_weight::WeightStore::default()),
    Box::new(i_rs_mood::MoodStore::default()),
];
for tool in &tools {
    let name = tool.tool_name();
    let commands = tool.commands();  // 类型安全的命令列表
    tool_index.insert(name, commands);
}
```

**方案 B — 动态发现**（保持现状但改进）：
- `i-rs-{name} tool info --json` 输出结构化工具元数据
- claw 缓存到 `i_rs_tool_index.json`
- 比 `skill summary` 更结构化

**推荐方案 A + B 并存**：编译时链接提供类型安全，动态发现作为 fallback。

---

## 3. 实施路线图

### Phase 1: `IrsTool` trait + `define_cli_tool!` 宏（~3-5 天）

1. 在 `i-rs-core` 定义 `IrsTool` trait、`SpecialCommand` enum、`ListFilter`、`Pagination`
2. 实现 `define_cli_tool!` 宏，替换 main.rs 样板
3. 选 3 个代表性 tool 迁移：`i-rs-weight`（日期+stats）、`i-rs-kv`（简单CRUD）、`i-rs-habit`（打卡+streak）
4. 验证所有现有测试通过

### Phase 2: 命令宏 + 分页（~2 天）

1. `Commands` enum、`handle_*` 函数由宏生成
2. 所有 list 命令加 `--limit`/`--offset`
3. 批量迁移剩余 67 个 tool

### Phase 3: 存储后端抽象（~3 天）

1. `StorageBackend` trait 定义
2. `FileBackend` 实现（搬现有代码）
3. `SqliteBackend` 实现（SQL 分页查询）
4. `ApiBackend` 实现（HTTP 调 i-rs-api）

### Phase 4: claw 集成改进（~2 天）

1. claw 的 `i_rs` tool 从 trait 获取命令 schema
2. 可选：编译时链接 tool crate
3. 工具发现不再依赖子进程

---

## 4. 待讨论的决策点

| # | 问题 | 选项 | 建议 |
|---|------|------|------|
| 1 | claw 是否需要编译时链接所有 70 个 tool crate？ | A. 是（类型安全） B. 动态发现（灵活） C. A+B 混合 | **C** — 核心 tool 链接，可选 tool 动态发现 |
| 2 | IrsTool trait 用 associated type 还是泛型参数？ | A. associated type B. 泛型 | **A** — 每个 Store 只对应一种 Entity |
| 3 | `define_cli_tool!` 宏覆盖度：90% 还是 100%？ | A. 100% 覆盖（所有 tool 无手写代码）B. 90% 覆盖（特殊 tool 手写） | **B** — time/ledger/car 等复杂 tool 留手写路径 |
| 4 | CLI 存储抽象是否需要支持 `ApiBackend`？ | A. 是（与 i-rs-api 统一）B. 否（CLI 只做本地，多端走 claw gateway） | **A** — 为微信小程序等场景提供统一数据源 |
| 5 | 分页粒度：CLI 层面还是仅 API 层面？ | A. CLI 也支持 B. 仅 API 支持 | **A** — 但 CLI 默认不分页，加 flag 开启 |
| 6 | 是否需要 trait 的 `commands()` 返回 clap Subcommand 动态构造？ | A. 动态 clap B. 手写 enum | **B** — clap derive 不支持动态 enum，宏生成即可 |

---

## 5. 对现有代码的影响

| 组件 | 影响 |
|------|------|
| `i-rs-core` | 新增 `IrsTool` trait + `define_cli_tool!` 宏 + `StorageBackend` trait |
| 70 个 CLI crate | 每个 crate 删 80% 样板代码，替换为宏调用（main.rs 30行） |
| `i-rs-api` | `make_app_tools!` 宏可直接从 `IrsTool` trait 推导 schema |
| `claw` i_rs tool | 从 "调子进程解析文本" 升级为 "读 trait 类型信息" |
| `claw` discover | 可选编译时链接替代子进程发现 |
