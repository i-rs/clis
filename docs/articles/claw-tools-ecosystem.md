# 工具与生态：i-rs-claw 的工具系统设计

## 为什么需要工具系统

LLM 再聪明，也有三个天生短板：
1. **无法访问实时数据** — 不知道今天的天气、最新的跑步记录
2. **无法执行操作** — 不能帮你记账、不能发送消息
3. **计算不可靠** — 三位数乘法都可能算错

工具系统就是来填补这些短板的。i-rs-claw 的工具架构经历了三个阶段：

## 第一阶段：单一工具

最初 claw 只有一个工具——`i_rs`——通过 `std::process::Command` 调用 i-rs CLI 工具：

```rust
// tools/i_rs.rs (简化)
async fn execute(args: &Value) -> Result<String> {
    let tool = args["tool"].as_str()?;
    let subcommand = args["subcommand"].as_str()?;
    let output = Command::new("i-rs")
        .args([tool, subcommand])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

这个设计虽然简单，但有一个关键优势：**claw 和 i-rs CLI 工具完全解耦**。即使 i-rs 内部重写，claw 不受影响。

## 第二阶段：工具注册表 (ToolRegistry)

随着功能增加，单一 `i_rs` 工具不够用了。我们需要 `web_search`、`calculator`、`chart` 等专用工具。

于是有了 `ToolRegistry` 和 `ClawTool` trait：

```rust
#[async_trait]
pub trait ClawTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameter_schema(&self, enabled_cli_tools: &[&str]) -> Value;
    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError>;
}
```

注册中心负责统一发现和调度：

```rust
pub struct ToolRegistry {
    pub tools: Vec<Box<dyn ClawTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: vec![
                Box::new(CalculatorTool),           // 精确计算
                Box::new(ChartTool),                // 图表生成
                Box::new(FileOpsTool),              // 文件操作
                Box::new(IrsTool),                  // 70 个 CLI 工具
                Box::new(SearchConversationsTool),  // 历史搜索
                Box::new(SearchToolsTool),          // 工具搜索
                Box::new(UserMemoryTool),           // 用户记忆
                Box::new(DelegateTool),             // 子 Agent 委托
                Box::new(VisionTool),               // 图片分析
                Box::new(WebSearchTool),            // 联网搜索
            ],
        }
    }
}
```

每个工具的职责：

| 工具 | 能力 | 解决什么问题 |
|------|------|------------|
| `calculator` | 精确数学计算 | LLM 算错数 |
| `chart` | 生成 SVG 图表 | 数据可视化 |
| `file_ops` | 读写文件 | 操作本地文件 |
| `i_rs` | 调用 70 个 CLI 工具 | 个人数据管理 |
| `search_conversations` | 历史会话搜索 | 找回上下文 |
| `search_tools` | 搜索可用工具 | 帮助 LLM 找到正确工具 |
| `user_memory` | 跨会话记忆 | 记住用户信息 |
| `delegate` | 子 Agent 委托 | 复杂任务分解 |
| `vision_tool` | 图片分析 | 多模态理解 |
| `web_search` | 联网搜索 | 获取实时信息 |

### 动态工具生成

`i_rs` 工具的 `parameter_schema` 是动态的——根据 `enabled_cli_tools` 参数生成 `tool.enum`：

```rust
fn parameter_schema(&self, enabled_cli_tools: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": {
            "tool": {
                "type": "string",
                "enum": enabled_cli_tools,  // ← 动态 !!
                "description": "i-rs CLI tool to call"
            },
            "subcommand": { "type": "string" },
            // ...
        },
        "required": ["tool", "subcommand"]
    })
}
```

这意味着 LLM 在生成参数时，只能选择当前启用的工具，大幅降低幻觉。

### MCP 工具集成

MCP (Model Context Protocol) 工具通过 `McpToolWrapper` 适配 `ClawTool` trait：

```rust
// tools/mcp_tools.rs
pub struct McpToolWrapper {
    def: ToolDefinition,
    client: Arc<McpClient>,
}

#[async_trait]
impl ClawTool for McpToolWrapper {
    fn name(&self) -> &str { &self.def.name }
    fn description(&self) -> &str { &self.def.description }
    fn parameter_schema(&self, _enabled: &[&str]) -> Value {
        self.def.input_schema.clone()
    }
    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        // 通过 MCP stdio 协议发送请求
        self.client.call_tool(&self.def.name, args).await
    }
}
```

MCP 工具的自动发现通过 `McpRegistry` 实现：

```rust
pub struct McpRegistry {
    pub clients: Vec<Arc<McpClient>>,
    pub tools: Vec<(usize, ToolDefinition)>,  // (client_idx, definition)
}
```

## 第三阶段：Skill 系统（动态工具注册）

Skill 系统允许用户通过 Markdown 文件定义新工具，claw 在启动时自动扫描并注册：

```markdown
# skills/weather/SKILL.md
---
name: weather
description: 查询天气
parameters:
  type: object
  properties:
    city:
      type: string
      description: 城市名
---

调用公开天气 API 获取指定城市的实时天气信息。
```

启动时加载：

```rust
let reg = ToolRegistry::new()
    .with_skills(&skill_store.list_skills())
    .with_mcp(&mcp_registry);
```

每个 Skill 变成一个 `SkillTool`，与其他内置工具在 LLM 看来没有区别。

## 工具执行引擎

### ToolContext

每个工具执行时都会收到一个共享上下文：

```rust
pub struct ToolContext {
    pub config: Config,
    pub http_client: reqwest::Client,  // 复用 HTTP 连接池
}
```

### ToolExecutor

`ToolExecutor` 负责超时控制和重试：

```rust
pub struct ToolExecutor {
    timeout: Duration,
    max_retries: u32,
}

impl ToolExecutor {
    pub async fn execute(
        &self,
        tool: &dyn ClawTool,
        args: &Value,
        ctx: &ToolContext,
    ) -> Result<String> {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            match tokio::time::timeout(self.timeout, tool.execute(args, ctx)).await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) => last_err = Some(e),
                Err(_) => last_err = Some(anyhow!("timeout after {:?}", self.timeout)),
            }
            tokio::time::sleep(Duration::from_millis(500 * (attempt + 1))).await;
        }
        Err(last_err.unwrap())
    }
}
```

在 ReAct 循环中，多个工具可以并行执行：

```rust
// tools/tool_call 是 LLM 返回的一组调用
let futures: Vec<_> = tool_calls
    .iter()
    .map(|call| executor.execute(&registry, &call.name, &call.args, &ctx))
    .collect();

let results = futures::future::join_all(futures).await;
```

## 工具发现与路由

### 智能路由

LLM 如何知道该用哪个工具？两种机制配合：

1. **OpenAI Function Calling** — 所有工具的 JSON Schema 一次性发给 LLM，LLM 自行选择
2. **SearchToolsTool** — 如果 LLM 不确定，可以先调用 `search_tools` 搜索可用的工具

### Skill Teach 机制

LLM 可以通过 `skill_tool` 创建新技能：

```
用户: "帮我写个脚本，每天自动备份我的笔记"
LLM: (调用 skill_tool → 创建 daily_backup skill)
     "Skill 'daily_backup' 已创建。下次你只需说'开始备份'我就会自动执行。"
```

## 从 1 到 无限：生态扩展

i-rs-claw 的工具生态系统支持三种扩展方式：

### 1. 内置工具（Rust 代码）

在 `crates/claw/src/tools/` 下新建文件，实现 `ClawTool` trait，注册到 `ToolRegistry::new()`。适合深度集成、高性能要求的场景。

### 2. MCP 工具（外部服务）

通过标准化的 MCP 协议接入任意外部服务。支持 stdio transport（本地进程）和 HTTP transport（远程服务）。适合第三方工具、浏览器控制、文件系统等。

### 3. Skill 工具（Markdown 文档）

编写 Markdown 文件即可定义新工具，无需编译。适合快速原型、知识库场景，让非开发者也能贡献工具。

## 未来展望

- **Plugin 热加载** — 运行时发现和加载新插件，无需重启 claw
- **工具市场** — 社区贡献的工具市场，一键安装
- **工具链编排** — 多个工具按 DAG 编排执行，而非简单的顺序或并行
- **权限系统** — 每个工具有独立的权限控制（读/写/执行），保护用户数据安全
