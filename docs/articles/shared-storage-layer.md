# 同一份数据，两个入口 — CLI 与 API 的共享存储层

## 当 CLI 和 API 用同一个文件

大多数应用的架构是这样的：CLI 工具读写本地文件，API 服务连接独立数据库，两者之间需要一个同步机制来保持数据一致。

i-rs 选择了另一条路。

```
```
          ┌─────────────────────────────────────┐
          │         ~/.config/i-rs/*.json       │
          │                                     │
          │  weight.json │ kv.json │ mood.json  │
          │  read.json   │ todo.json│ ...       │
          └──────────┬────────────────┬─────────┘
                     │                │
          ┌──────────▼───┐    ┌───────▼──────────┐
          │  CLI 工具     │    │  i-rs-api        │
          │  i-rs-weight  │    │  REST API 服务   │
          │  i-rs-kv      │    │  (Axum)          │
          │  ... 70+ 工具  │    │                  │
          │               │    │  curl /api/weight │
          │  load_store() │    │  SharedStore<T>   │
          │  save_store() │    │  RwLock + 自动落盘 │
          └───────────────┘    └──────────────────┘
```
```

**CLI 和 API 读写的是同一个 JSON 文件。** 没有中间层，没有同步任务，没有数据迁移。

## `Storage<T>` — 一切存储的基石

`i-rs-core` 提供了 `Storage<T>` 泛型，这是所有 70 个 CLI 工具和 API 服务的存储基础：

```rust
// i-rs-core 的核心存储抽象
pub struct Storage<T> {
    data: T,
    file_path: PathBuf,
}

impl<T: Serialize + DeserializeOwned + Default> Storage<T> {
    pub fn new(filename: &str) -> Self { /* 拼接 ~/.config/i-rs/{filename}.json */ }
    pub fn load(&mut self) -> Result<()>  { /* 从磁盘读取 JSON */ }
    pub fn save(&self) -> Result<()>      { /* 将 data 写入磁盘 */ }
    pub fn save_data(&self, data: &T) -> Result<()> { /* 写入指定数据 */ }
}
```

每个 CLI 工具通过 `create_store!` 宏一行代码接入：

```rust
// crates/i-rs-weight/src/storage/mod.rs
i_rs_core::create_store!(WeightStore, "weight");
// 展开为：load_store()、save_store()、export_data()、import_data()、clear_data()
```

这条宏展开的函数操作的都是 **`~/.config/i-rs/weight.json`** 这个文件。

## `SharedStore<T>` — API 的线程安全包装

API 服务不能像 CLI 那样每次请求都 `load_store()`——那会引入磁盘 I/O 和文件锁竞争。所以 i-rs-api 提供了 `SharedStore<T>`：

```rust
// crates/i-rs-api/src/store.rs
pub struct SharedStore<T> {
    inner: Arc<RwLock<T>>,
    filename: String,
}

impl<T> SharedStore<T> {
    /// 启动时从磁盘加载
    pub fn load(filename: &str) -> Self {
        let mut storage = Storage::<T>::new(filename);
        let _ = storage.load();  // 从 ~/.config/i-rs/{filename}.json 读取
        let inner = Arc::new(RwLock::new(storage.data));
        Self { inner, filename: filename.to_string() }
    }

    /// 读操作：零 I/O，纯内存
    pub fn read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.inner.read().unwrap();
        f(&guard)
    }

    /// 写操作：修改内存后自动落盘
    pub fn write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.write().unwrap();
        let result = f(&mut guard);
        // 每次写入后自动刷到磁盘
        let storage = Storage::<T>::new(&self.filename);
        storage.save_data(&guard).ok();
        result
    }
}
```

关键设计决策：

- **读操作零 I/O** — 数据常驻内存，RwLock 允许多个读并发
- **写操作即时落盘** — 每次修改后立即写回 JSON 文件，保证 crash-safe
- **文件名与 CLI 完全一致** — `SharedStore::load("weight")` 操作的是 `weight.json`，和 `i-rs-weight` CLI 完全一样

## 70 个工具，70 个 SharedStore

i-rs-api 通过 `make_app_tools!` 宏一次性生成所有 70 个工具的存储状态：

```rust
// crates/i-rs-api/src/main.rs
make_app_tools! {
    weight: i_rs_weight::models::WeightStore => "weight",
    kv: i_rs_kv::models::KvStore => "kv",
    mood: i_rs_mood::models::MoodStore => "mood",
    read: i_rs_read::models::ReadStore => "read",
    // ... 所有 70 个工具，每个一行
};
```

这个宏展开为：

```rust
pub struct AppState {
    pub weight: SharedStore<WeightStore>,  // → weight.json
    pub kv: SharedStore<KvStore>,          // → kv.json
    pub mood: SharedStore<MoodStore>,      // → mood.json
    // ...
}
```

**重点在于第二列：`"weight"`、`"kv"`、`"mood"` 这些文件名，和 CLI 工具 `create_store!` 宏中指定的完全一致。** 这是共享存储的契约。

## 数据在 CLI 和 API 之间流动

来看一个完整的数据流：

### 场景：用 CLI 添加，用 API 查询

```bash
# 终端 1：通过 CLI 添加一条体重记录
$ i-rs-weight add --weight 72.5 --date 2026-05-15
✅ Added: 2026-05-15: 72.5 kg

# 终端 2：通过 API 查询同一数据（无需重启 API 服务）
$ curl http://localhost:8080/api/weight | jq '.data'
[
  {
    "date": "2026-05-15",
    "weight": 72.5,
    "created_at": 1747296000,
    "updated_at": 1747296000
  }
]
```

API 不需要重启，不需要刷新缓存。**因为 SharedStore 的写操作会即时落盘，而数据已经在内存中。** 不过如果数据是通过 CLI 写入的（而不是通过 API），API 内存中还没有——怎么办？

答案在路由处理器的设计里。对于 **读操作**，API 直接从 SharedStore 的内存中读取，这适用于 API 自身写入的数据。但对于需要实时看到 CLI 变更的场景，有两种策略：

1. **API 优先** — 所有写入走 API，CLI 只读
2. **混合使用** — 各自写入，数据以最后一次写入为准

实际上由于 i-rs-api 的 `write` 方法每次都落盘，而 CLI 的 `save_store` 也写同一个文件，**最后一个写入者决定最终状态**。在个人使用场景下，这完全足够。

### 场景：用 API 添加，用 CLI 查看

```bash
# 通过 API 添加一条 KV 数据
$ curl -X POST http://localhost:8080/api/kv \
  -H "Content-Type: application/json" \
  -d '{"key": "github-token", "value": "ghp_xxxx"}'

# 通过 CLI 立即查看到（新启动的 CLI 进程会读取最新的 JSON）
$ i-rs-kv get github-token
github-token: ghp_xxxx
```

因为 CLI 每次运行都是独立进程，`load_store()` 会从磁盘重新读取最新数据，所以永远能看到 API 写入的内容。

## 为什么不需要数据库？

i-rs 选择 JSON 文件而非数据库，是出于以下考虑：

| 维度 | JSON 文件 | 专用数据库 |
|------|----------|-----------|
| 依赖 | 零额外依赖 | 需要数据库服务和驱动 |
| 备份 | `cp -r ~/.config/i-rs backup` | 需要 dump/restore 工具 |
| 透明 | 可直接 `cat`、`grep`、`jq` | 需要查询语言 |
| 版本控制 | 可纳入 git 管理 | 通常不支持 |
| 便携 | 一个 USB 即可随身携带 | 需要部署环境 |
| 并发 | 单用户场景无竞争 | 为多用户并发设计 |

i-rs 是**个人工具集**，不是企业级系统。JSON 文件在个人使用场景下是完美的匹配——简单、透明、可控。

## 共享存储带来的独特能力

### 1. 数据生态的统一

因为所有工具共享同一套存储文件和同一个 `Storage<T>` 抽象，整个 i-rs 生态的数据是天然统一和可互操作的：

```bash
# weight.json → CLI → CSV 导出 → 电子表格
i-rs-weight list --json | jq -r '.[] | [.date, .weight] | @csv' > weight.csv

# 同一个 weight.json → API → 网页仪表盘
# 同一个 weight.json → 脚本 → 统计分析
```

### 2. 数据生命周期管理

`data` 子命令在 CLI 和 API 上同时存在：

| CLI | API | 功能 |
|-----|-----|------|
| `i-rs-weight data export` | `GET /api/weight/data/export` | 导出全部数据为 JSON |
| `i-rs-weight data import` | `POST /api/weight/data/import` | 从 JSON 导入数据 |
| `i-rs-weight data clear` | `DELETE /api/weight/data/clear` | 清空所有数据 |

操作同一个文件，效果完全一致。

### 3. 备份即信任

```bash
# 一键备份所有 i-rs 数据
cp -r ~/.config/i-rs ~/backups/i-rs-$(date +%Y%m%d)

# 还原
cp -r ~/backups/i-rs-20260515/* ~/.config/i-rs/

# 查看任意工具的数据量
cat ~/.config/i-rs/weight.json | jq '.entries | length'
```

一个目录，70 个 JSON 文件，就是整个 i-rs 的数据宇宙。不需要理解任何数据库 schema，不需要运行任何迁移脚本。

## 与其他架构的对比

传统做法通常是：

```
CLI ──→ 本地 SQLite ←── API 服务
```

或者：

```
CLI ──→ 云 API ──→ 云数据库
```

i-rs 的做法：

```
CLI ──→ ~/.config/i-rs/*.json ←── API 服务
```

不同之处在于：

- **无状态同步** — 没有"同步"这个步骤，因为根本就是同一个文件
- **无数据分叉** — CLI 看到的就是 API 看到的，不会出现"怎么数据不一致"的困惑
- **无服务依赖** — 即使 API 没启动，CLI 照常工作；即使 CLI 没安装，API 也能独立运行
- **无架构升级成本** — 从 CLI-only 到开启 API，数据结构无需任何变更

## 小结

i-rs 的共享存储层设计可以总结为三个层次：

```
第一层：Storage<T>        ── 核心抽象，JSON 文件读写（i-rs-core）
第二层：SharedStore<T>    ── 线程安全包装，内存 + 即时落盘（i-rs-api）
第三层：make_app_tools!   ── 宏生成 70 个存储实例（i-rs-api main.rs）
```

三个层次最终指向同一个东西——`~/.config/i-rs/{tool}.json`。

CLI 用它，API 用它，浏览器扩展通过 Native Messaging 最终也用它。不需要同步，不需要转换，不需要迁移。

数据是你的，放在你的磁盘上，格式是纯 JSON。任何工具都能读，任何方式都能写，**但入口永远只有一个——`~/.config/i-rs/` 目录**。
