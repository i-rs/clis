# Token 用量统计 — 设计方案

> 状态: Draft  
> 日期: 2026-05-20  
> 范围: i-rs-claw  

---

## 1. 背景

### 1.1 现状

当前 i-rs-claw 已有基础的 token 追踪能力：

- `TokenUsage` 结构体 (`llm.rs`) 包含 `prompt_tokens` / `completion_tokens` / `total_tokens`
- `LlmEvent::Done` 和 `LlmEvent::HttpLog` 携带 token 数据
- `app.token_usage` 存储最后一次请求的用量，在状态栏显示
- `memory.rs` 追踪工具调用频率（`tool_frequency: HashMap<String, usize>`），但不追踪 token

**缺失的能力**：
- ❌ 无持久化存储 — 重启后丢失所有 token 数据
- ❌ 无聚合统计 — 无法按 agent / model / 时间段查看
- ❌ 无成本估算 — 不知道花了多少钱
- ❌ 无趋势分析 — 无法识别 token 消耗异常
- ❌ Dashboard 无用量面板

### 1.2 目标

- 持久化记录每次 LLM 请求的 token 用量
- 支持多维度聚合查询（按 agent、model、时间段）
- 支持成本估算（按模型定价）
- TUI 和 Dashboard 均可查看统计
- 零性能开销（异步写入，不阻塞主循环）

---

## 2. 数据模型

### 2.1 TokenRecord — 单条记录

```rust
// src/stats/mod.rs

/// 单次 LLM 请求的 token 用量记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRecord {
    /// 唯一 ID (ULID 或 UUID v7)
    pub id: String,
    /// 请求时间戳 (Unix epoch seconds)
    pub timestamp: i64,
    /// Agent ID
    pub agent_id: String,
    /// 模型名称 (e.g. "gpt-4o-mini", "claude-sonnet-4-20250514")
    pub model: String,
    /// Provider 类型 (e.g. "openai", "anthropic")
    pub provider: String,
    /// 输入 token 数
    pub prompt_tokens: u32,
    /// 输出 token 数
    pub completion_tokens: u32,
    /// 总 token 数
    pub total_tokens: u32,
    /// 是否包含工具调用
    pub has_tool_calls: bool,
    /// 工具调用数量
    pub tool_call_count: u32,
    /// ReAct 轮数 (该次用户请求经历的 LLM 调用次数)
    pub react_rounds: u32,
    /// 是否成功
    pub success: bool,
    /// 请求延迟 (ms)
    pub latency_ms: u64,
    /// 估算成本 (USD)
    pub estimated_cost_usd: f64,
}
```

### 2.2 TokenStats — 聚合结果

```rust
/// 聚合统计结果，用于 UI 展示。
#[derive(Debug, Clone, Serialize)]
pub struct TokenStats {
    /// 统计时间范围
    pub period: StatsPeriod,
    /// 总请求次数
    pub total_requests: u32,
    /// 总 token 数
    pub total_tokens: u64,
    /// 总输入 token
    pub total_prompt_tokens: u64,
    /// 总输出 token
    pub total_completion_tokens: u64,
    /// 估算总成本 (USD)
    pub total_cost_usd: f64,
    /// 平均每次请求 token 数
    pub avg_tokens_per_request: f64,
    /// 平均每次请求延迟 (ms)
    pub avg_latency_ms: f64,
    /// 成功率
    pub success_rate: f64,
    /// 按模型分组
    pub by_model: Vec<ModelStats>,
    /// 按 Agent 分组
    pub by_agent: Vec<AgentStats>,
    /// 按天分组的时间序列
    pub daily_series: Vec<DailyStats>,
}

#[derive(Debug, Clone, Serialize)]
pub enum StatsPeriod {
    Today,
    Last7Days,
    Last30Days,
    All,
    Custom { from: i64, to: i64 },
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelStats {
    pub model: String,
    pub request_count: u32,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStats {
    pub agent_id: String,
    pub request_count: u32,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyStats {
    pub date: String,  // "2026-05-20"
    pub request_count: u32,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
}
```

### 2.3 ModelPricing — 成本估算

```rust
/// 模型定价表 (per 1M tokens, USD)
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_per_m: f64,   // 输入 token 每百万价格
    pub output_per_m: f64,  // 输出 token 每百万价格
}

impl ModelPricing {
    pub fn estimate_cost(&self, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        (prompt_tokens as f64 / 1_000_000.0) * self.input_per_m
            + (completion_tokens as f64 / 1_000_000.0) * self.output_per_m
    }
}
```

内置定价表（可配置覆盖）：

| 模型 | Input ($/1M) | Output ($/1M) |
|------|-------------|--------------|
| gpt-4o-mini | 0.150 | 0.600 |
| gpt-4o | 2.500 | 10.000 |
| claude-sonnet-4-20250514 | 3.000 | 15.000 |
| claude-haiku-4-20250514 | 0.800 | 4.000 |
| ollama/* | 0.000 | 0.000 |
| 未识别模型 | 0.000 | 0.000 |

---

## 3. 存储设计

### 3.1 存储格式: JSONL

与现有 session 消息存储 (`*.jsonl`) 保持一致：

```
~/.i-rs-claw/stats/
└── usage.jsonl
```

每条记录一行 JSON：

```jsonl
{"id":"01JXYZ...","timestamp":1716220800,"agent_id":"default","model":"gpt-4o-mini","provider":"openai","prompt_tokens":1234,"completion_tokens":567,"total_tokens":1801,"has_tool_calls":true,"tool_call_count":2,"react_rounds":3,"success":true,"latency_ms":1523,"estimated_cost_usd":0.000524}
{"id":"01JXYZ...","timestamp":1716220860,"agent_id":"analyst","model":"gpt-4o","provider":"openai","prompt_tokens":5678,"completion_tokens":234,"total_tokens":5912,"has_tool_calls":false,"tool_call_count":0,"react_rounds":1,"success":true,"latency_ms":3210,"estimated_cost_usd":0.016528}
```

**选择 JSONL 的理由**：
- 追加写入，无需读-改-写
- 与现有 session JSONL 模式一致
- 可用 `grep`/`jq` 直接查询调试
- 无需引入数据库依赖

### 3.2 写入策略

```
LlmEvent::Done(usage) → stats_collector.record(record) → 内存缓冲 → 批量刷盘
```

- **内存缓冲**: 最多缓存 50 条或 30 秒
- **批量刷盘**: 一次性写入 JSONL，避免频繁 I/O
- **优雅关闭**: 应用退出时强制 flush
- **原子写入**: 复用现有 `atomic_write` 模式（先写 `.tmp` 再 `rename`）

### 3.3 读取策略

- **按需加载**: 启动时不加载全部记录
- **范围扫描**: 按 `timestamp` 范围读取 JSONL 行
- **内存索引**: 维护一个轻量索引（日期 → 文件偏移），加速范围查询

---

## 4. 模块设计

### 4.1 新增文件

```
crates/claw/src/stats/
├── mod.rs          — 模块入口 + StatsManager 结构体
├── collector.rs    — 异步收集器 (mpsc channel + 批量写入)
├── pricing.rs      — 模型定价表 + 成本估算
├── aggregator.rs   — 聚合逻辑 (TokenRecord → TokenStats)
└── store.rs        — JSONL 读写 + 索引管理
```

### 4.2 StatsManager — 核心结构体

```rust
pub struct StatsManager {
    /// 写入通道 (collector 从此接收记录)
    tx: mpsc::UnboundedSender<TokenRecord>,
    /// 存储路径
    store_path: PathBuf,
    /// 定价表
    pricing: ModelPricingTable,
}

impl StatsManager {
    /// 创建 StatsManager 并启动后台写入任务
    pub fn new(claw_dir: &PathBuf) -> Self;
    
    /// 记录一次 LLM 请求的 token 用量
    pub fn record(&self, record: TokenRecord);
    
    /// 查询聚合统计
    pub fn query(&self, period: StatsPeriod) -> TokenStats;
    
    /// 按模型查询统计
    pub fn by_model(&self, period: StatsPeriod) -> Vec<ModelStats>;
    
    /// 按 Agent 查询统计
    pub fn by_agent(&self, period: StatsPeriod) -> Vec<AgentStats>;
    
    /// 获取时间序列数据
    pub fn daily_series(&self, period: StatsPeriod) -> Vec<DailyStats>;
    
    /// 获取今日摘要 (用于状态栏快速显示)
    pub fn today_summary(&self) -> TodaySummary;
}

pub struct TodaySummary {
    pub requests: u32,
    pub tokens: u64,
    pub cost_usd: f64,
}
```

### 4.3 Collector — 后台写入

```rust
// 运行在 tokio::spawn 中的后台任务
async fn collector_worker(
    mut rx: mpsc::UnboundedReceiver<TokenRecord>,
    store_path: PathBuf,
) {
    let mut buffer: Vec<TokenRecord> = Vec::with_capacity(50);
    let mut last_flush = Instant::now();
    const FLUSH_INTERVAL: Duration = Duration::from_secs(30);
    const FLUSH_THRESHOLD: usize = 50;
    
    loop {
        let timeout = tokio::time::sleep(FLUSH_INTERVAL);
        tokio::select! {
            maybe_record = rx.recv() => {
                match maybe_record {
                    Some(record) => {
                        buffer.push(record);
                        if buffer.len() >= FLUSH_THRESHOLD {
                            flush(&mut buffer, &store_path).await;
                            last_flush = Instant::now();
                        }
                    }
                    None => break, // 通道关闭，退出
                }
            }
            _ = timeout => {
                if !buffer.is_empty() {
                    flush(&mut buffer, &store_path).await;
                    last_flush = Instant::now();
                }
            }
        }
    }
    
    // 退出前强制 flush
    if !buffer.is_empty() {
        flush(&mut buffer, &store_path).await;
    }
}
```

### 4.4 Aggregator — 聚合逻辑

```rust
pub struct Aggregator<'a> {
    records: &'a [TokenRecord],
    pricing: &'a ModelPricingTable,
}

impl<'a> Aggregator<'a> {
    pub fn new(records: &'a [TokenRecord], pricing: &'a ModelPricingTable) -> Self;
    
    /// 按时间范围过滤
    pub fn filter_period(&self, period: &StatsPeriod) -> Vec<&TokenRecord>;
    
    /// 生成完整统计
    pub fn aggregate(&self, period: StatsPeriod) -> TokenStats;
    
    /// 按模型分组聚合
    pub fn group_by_model(&self, records: &[&TokenRecord]) -> Vec<ModelStats>;
    
    /// 按 Agent 分组聚合
    pub fn group_by_agent(&self, records: &[&TokenRecord]) -> Vec<AgentStats>;
    
    /// 按天分组聚合
    pub fn group_by_day(&self, records: &[&TokenRecord]) -> Vec<DailyStats>;
}
```

---

## 5. 集成点

### 5.1 engine.rs — chat_loop

在 `chat_loop` 结束时记录 token 用量：

```rust
// engine.rs: chat_loop 中 LlmEvent::Done 发送处
Ok(StreamResult::Text(usage, text)) => {
    // ... 现有逻辑 ...
    
    // 记录 token 用量
    if let Some(usage) = &usage {
        stats_manager.record(TokenRecord {
            id: generate_id(),
            timestamp: chrono::Local::now().timestamp(),
            agent_id: config.agent_id.clone(),  // 需传入
            model: provider.model().to_string(),
            provider: config.provider.clone(),
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            has_tool_calls: false,  // Text 模式无工具调用
            tool_call_count: 0,
            react_rounds: round_count,
            success: true,
            latency_ms: /* 从 HttpLog 获取 */,
            estimated_cost_usd: pricing.estimate(
                &config.model, usage.prompt_tokens, usage.completion_tokens
            ),
        });
    }
    
    let _ = tx.send(LlmEvent::Done(msgs.clone(), usage));
    break;
}
```

### 5.2 AppCore — 持有 StatsManager

```rust
// core/mod.rs: AppCore 结构体
pub struct AppCore {
    pub config: Config,
    pub session_manager: SessionManager,
    pub agent_store: AgentRuntimeStore,
    pub stats_manager: StatsManager,  // ← 新增
}
```

### 5.3 App — 显示今日摘要

```rust
// app.rs: App 结构体
pub struct App {
    // ... 现有字段 ...
    pub today_stats: Option<TodaySummary>,  // ← 新增
}
```

### 5.4 TUI 状态栏 — 显示今日用量

```
状态栏当前:
[agent:default] [model:gpt-4o-mini] [tok: 1234p+567c] [session:abc]

改为:
[agent:default] [model:gpt-4o-mini] [今日: 12次 | 45.2K tok | $0.03] [session:abc]
```

### 5.5 Dashboard API — 新增端点

```
GET /api/stats           — 获取今日摘要
GET /api/stats?period=7d — 获取最近 7 天统计
GET /api/stats/by-model  — 按模型分组
GET /api/stats/by-agent  — 按 Agent 分组
GET /api/stats/daily     — 时间序列数据（用于图表）
```

---

## 6. TUI 展示设计

### 6.1 状态栏增强

在底部状态栏增加今日摘要（不超过 40 字符）：

```
今日: 12次 · 45.2K tok · $0.03
```

### 6.2 用量面板（新面板，Ctrl+U 打开）

```
┌── Token 用量统计 ──────────────────────────────┐
│  周期: [今天] [7天] [30天] [全部]              │
├───────────────────────────────────────────────┤
│  总请求: 128 次                                │
│  总 Token: 1.2M (输入 890K + 输出 310K)        │
│  估算成本: $0.47                               │
│  平均延迟: 1,523ms                             │
│  成功率: 98.4%                                 │
├───────────────────────────────────────────────┤
│  按模型:                                       │
│  gpt-4o-mini    98次  890K tok  $0.18  █████  │
│  gpt-4o         22次  280K tok  $0.25  ██     │
│  claude-sonnet   8次   30K tok  $0.04  █      │
├───────────────────────────────────────────────┤
│  按 Agent:                                     │
│  default       102次  980K tok  $0.32  █████  │
│  analyst        26次  220K tok  $0.15  █      │
├───────────────────────────────────────────────┤
│  最近 7 天趋势:                                │
│  5/14 ████ 52K    5/18 █████ 68K              │
│  5/15 ███  38K    5/19 ████ 55K               │
│  5/16 ██   22K    5/20 ██████ 82K             │
│  5/17 █████ 65K                               │
└───────────────────────────────────────────────┘
```

---

## 7. 配置

```toml
# ~/.i-rs-claw/config.toml

[stats]
# 是否启用 token 统计 (默认启用)
enabled = true

# 自定义模型定价 (覆盖内置定价表)
# 格式: "模型名" = { input = 每百万输入价格, output = 每百万输出价格 }
[stats.pricing]
"gpt-4o-mini" = { input = 0.15, output = 0.60 }
"gpt-4o" = { input = 2.50, output = 10.00 }
"claude-sonnet-4-20250514" = { input = 3.00, output = 15.00 }

# 统计保留天数 (0 = 无限期)
[stats.retention]
keep_days = 90
```

---

## 8. 实现阶段

### Phase 1: 基础设施 (1-2 天)
1. 创建 `stats/` 模块结构
2. 实现 `TokenRecord` / `TokenStats` 数据模型
3. 实现 `ModelPricing` 定价表
4. 实现 `store.rs` JSONL 读写

### Phase 2: 收集器 (1 天)
5. 实现 `collector.rs` 后台写入
6. 在 `chat_loop` 中集成记录逻辑
7. 在 `AppCore` 中持有 `StatsManager`
8. 优雅关闭时 flush

### Phase 3: TUI 展示 (1-2 天)
9. 状态栏显示今日摘要
10. 新增用量面板 (Ctrl+U)
11. 周期切换 (今天/7天/30天/全部)
12. 按模型/Agent 分组展示

### Phase 4: Dashboard (1-2 天)
13. 新增 `/api/stats` 端点
14. Dashboard 前端用量面板
15. 时间序列图表

### Phase 5: 高级功能 (可选)
16. 预算告警（日用量超过阈值时通知）
17. 异常检测（token 突增告警）
18. 导出功能（CSV/JSON）

---

## 9. 边界情况处理

| 场景 | 处理方式 |
|------|---------|
| 请求失败 (网络错误) | 记录 `success: false`，token 为 0 |
| 流式中断 | 记录已收到的 token，`success: false` |
| 定价表未匹配 | 成本估算为 $0.00，记录 warn 日志 |
| JSONL 文件损坏 | 跳过损坏行，记录 warn 日志 |
| 磁盘空间不足 | 静默丢弃，记录 error 日志 |
| 并发写入 | collector 单线程消费，无并发 |
| 应用崩溃 | 最多丢失缓冲区中未刷盘的记录 (<50 条) |

---

## 10. 性能评估

| 操作 | 预期延迟 | 说明 |
|------|---------|------|
| record() | <1μs | 仅 mpsc channel send |
| 批量刷盘 (50 条) | ~5ms | 单次 fsync |
| 查询今日统计 | <10ms | 扫描当日行（通常 <200 条） |
| 查询 30 天统计 | <50ms | 扫描 30 天行（通常 <5000 条） |
| 内存占用 | <1MB | 索引 + 聚合结果 |

**对主循环的影响**: 零。record() 是异步 channel send，不阻塞。

---

## 11. 替代方案评估

### 方案 A: JSONL 文件（推荐 ✅）
- 优点: 简单、无依赖、与现有模式一致、可 grep 调试
- 缺点: 大数据量时查询慢（但个人使用场景不会有大数据量）

### 方案 B: SQLite
- 优点: 查询能力强、支持复杂聚合
- 缺点: 引入新依赖、增加编译体积、对个人场景过度设计

### 方案 C: 内存 + 定期序列化
- 优点: 查询极快
- 缺点: 崩溃丢失数据、内存随时间增长

### 方案 D: 复用 session JSONL
- 优点: 无需新文件
- 缺点: 混合关注点、查询效率低、难以按时间范围扫描

**结论**: JSONL 专用文件最适合当前场景。个人 AI 助理日均 <200 次请求，30 天 <6000 条记录，JSONL 扫描性能完全足够。
