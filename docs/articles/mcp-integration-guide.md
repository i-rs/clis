# claw MCP 接入指南 — 让 AI 终端助理获得无限工具拓展能力

## 总览

i-rs-claw 是一个运行在终端中的 AI 助理，它本身已经内置了大量工具（70+ i-rs CLI 工具、文件操作、搜索、记忆等）。但现实世界中的工具远不止这些——数据库查询、代码分析、项目管理、设计稿审查……每项工作都有专门的工具。

MCP（Model Context Protocol）的出现，让 claw 的"工具箱"不再局限于内置工具。通过 MCP，claw 可以连接任意实现了 MCP 协议的服务器，发现其工具（tool）、调用其能力，就像使用内置工具一样自然。

**阅读本文后你会了解：**

- claw 如何通过 MCP 连接外部工具生态
- 如何配置和接入现有的 MCP 服务器（文件系统、搜索、数据库等）
- 如何通过插件系统自动发现本地 MCP 插件
- claw 的 MCP 实现架构 — 从 `rmcp` SDK 到 `ClawTool` 的完整链路
- 如何手写一个 Rust MCP 服务器样例

---

## 架构总览

```
┌──────────────────────────────────────────────────────────────────┐
│                     claw (AI 终端助理)                            │
│                                                                  │
│  ┌──────────┐   ┌──────────────────┐   ┌─────────────────────┐  │
│  │  LLM     │   │  ToolRegistry    │   │   McpRegistry       │  │
│  │ (deepseek│──►│                  │──►│                      │  │
│  │  /openai)│   │  ┌────────────┐  │   │  ┌────────────────┐ │  │
│  │          │   │  │ ClawTool   │  │   │  │ McpClient[0]   │ │  │
│  └──────────┘   │  │  (内置)     │  │   │  │  ├ stdio proc  │ │  │
│       │         │  ├────────────┤  │   │  │  └─ list_tools()│ │  │
│       │ tool    │  │ ClawTool   │  │   │  │     call_tool() │ │  │
│       │ call    │  │  (MCP桥接)  │◄─┼───┤  └────────────────┘ │  │
│       ▼         │  ├────────────┤  │   │  ┌────────────────┐ │  │
│  ┌──────────┐   │  │ ClawTool   │  │   │  │ McpClient[1]   │ │  │
│  │ 结果聚合  │   │  │  (MCP桥接)  │◄─┼───┤  │  ├ SSE HTTP    │ │  │
│  └──────────┘   │  └────────────┘  │   │  │  └─ ...         │ │  │
│                │                   │   │  └────────────────┘ │  │
│                └──────────────────┘   └─────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
         │                               │
         ▼                               ▼
┌──────────────────┐         ┌──────────────────────┐
│ 内置 i-rs 工具    │         │  外部 MCP 服务器      │
│ (70+ CLI crates) │         │                      │
│                  │         │  ┌────────────────┐   │
│                  │         │  │ hello-mcp      │   │
│                  │         │  │ (stdio 子进程)   │   │
│                  │         │  ├────────────────┤   │
│                  │         │  │ calculator-mcp │   │
│                  │         │  │ (stdio 子进程)   │   │
│                  │         │  ├────────────────┤   │
│                  │         │  │ remote-server  │   │
│                  │         │  │ (SSE 远程)      │   │
│                  │         │  └────────────────┘   │
└──────────────────┘         └──────────────────────┘
```

### 核心模块

| 模块 | 文件 | 职责 |
|------|------|------|
| `McpClient` | `src/mcp.rs` | 单个 MCP 服务器连接（stdio 或 SSE） |
| `McpRegistry` | `src/mcp.rs` | 管理所有 MCP 连接，统一发现工具 |
| `McpToolWrapper` | `src/tools/mcp_tools.rs` | 将 MCP 工具适配为 `ClawTool` trait |
| `ToolRegistry` | `src/tools/mod.rs` | 工具注册中心，合并内置 + MCP 工具 |
| `PluginManager` | `src/plugin.rs` | 插件自动发现 + 状态管理 |
| CLI 命令 | `src/cli.rs` | `mcp --list/--check` + `plugin` 子命令 |

### 数据流

```
1. 启动 → 加载 config.toml（读取 [[mcp_servers]] 列表）
          ↓
2. 发现 ~/.i-rs/claw/plugins/*/plugin.toml（若 plugins_auto_discover = true）
          ↓
3. 合并配置 → 遍历每个 McpServerConfig
          ↓
4. McpClient::connect() / McpClient::connect_sse() → 建立连接
          ↓
5. rmcp 自动完成 initialize 握手
          ↓
6. McpClient::list_tools() → 发现该服务器的所有工具
          ↓
7. McpToolWrapper 包装 → 注册到 ToolRegistry
          ↓
8. LLM 会话 → ToolRegistry::enabled_schemas() → 发送给 LLM
          ↓
9. LLM 选择调用 → ToolRegistry::execute() → McpToolWrapper::execute()
          ↓
10. McpClient::call_tool() → MCP 服务器处理 → 返回结果 → 回复用户
```

---

## 配置 MCP 服务器

claw 的 `config.toml` 支持直接配置 MCP 服务器连接：

```toml
[[mcp_servers]]
name = "filesystem"
transport_type = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/workspace"]

[[mcp_servers]]
name = "remote-api"
transport_type = "sse"
url = "https://api.example.com/mcp/sse"

[[mcp_servers]]
name = "custom-server"
transport_type = "stdio"
command = "/usr/local/bin/my-mcp-server"
args = ["--port", "8080"]
env = ["DEBUG=true", "API_KEY=xxx"]
enabled = true
```

### 配置字段说明

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `name` | string | 必填 | 服务器标识名，用于管理和引用 |
| `transport_type` | string | `"stdio"` | 传输类型：`"stdio"`（子进程）或 `"sse"`（HTTP） |
| `command` | string | `null` | stdio 模式：可执行文件路径 |
| `args` | string[] | `[]` | stdio 模式：命令行参数 |
| `url` | string | `null` | SSE 模式：服务端 URL |
| `env` | string[] | `[]` | stdio 模式：环境变量（`KEY=VAL` 格式） |
| `enabled` | bool | `true` | 是否启用此连接 |

### 两种传输方式

**stdio（本地子进程）**：claw 启动一个子进程，通过 stdin/stdout 与该进程通信。这是最常见的 MCP 连接方式，适用于本地运行的工具服务（如文件系统操作、代码分析等）。

**SSE（远程 HTTP）**：claw 通过 Server-Sent Events 连接远程 MCP 服务端。适用于远程 API、云服务等场景。

---

## 插件系统

除了直接在配置文件中编写 MCP 服务器，claw 还支持通过"插件"机制自动发现和加载 MCP 服务器。

### 插件目录结构

```
~/.i-rs/claw/plugins/
├── hello-mcp/
│   ├── plugin.toml          # 插件清单
│   └── <二进制 或 脚本>       # 可执行文件
├── calculator-mcp/
│   ├── plugin.toml
│   └── ...
└── state.json               # 自动生成的启停状态
```

### plugin.toml 格式

```toml
[plugin]
name = "hello-mcp"
version = "0.1.0"
description = "一个简单的问候 MCP 服务器"

[transport]
transport_type = "stdio"
command = "hello-mcp"
```

### 自动发现流程

1. 启动时，若 `plugins_auto_discover = true`（默认），claw 扫描 `~/.i-rs/claw/plugins/` 目录
2. 每个子目录下查找 `plugin.toml`，解析为 `PluginManifest`
3. 校验目录名与 `plugin.name` 一致
4. 从 `state.json` 读取启用/禁用状态
5. 将已启用的插件转换为 `McpServerConfig`（名称自动加 `plugin:` 前缀）
6. 合并到 MCP 配置列表中统一初始化

### 插件管理命令

```
# 列出所有已发现的插件
i-rs-claw plugin --list

# 启用/禁用插件
i-rs-claw plugin --enable hello-mcp
i-rs-claw plugin --disable hello-mcp
```

插件方便的地方在于：你不需要修改 `config.toml`，只需把 `plugin.toml` 和可执行文件放到 `plugins/<name>/` 目录下，下次启动 claw 会自动加载。

---

## MCP 核心实现

### McpClient — 单个 MCP 服务器连接

`McpClient` 是对 `rmcp` SDK 的一层轻量封装，负责一个 MCP 服务器的完整生命周期：

```rust
pub struct McpClient {
    pub name: String,
    rt: Arc<tokio::runtime::Runtime>,
    service: Arc<RunningService<RoleClient, ()>>,
}
```

关键设计决策：

- **同步接口**：内部持有私有 `tokio::runtime`，对外暴露同步 API。这样上层调用方（`ClawTool::execute()`、CLI 命令）无需关心异步细节。
- **Arc 共享**：`McpClient` 实现 `Clone`，通过 `Arc<RunningService>` 和 `Arc<Runtime>` 共享底层连接，多个 `McpToolWrapper` 可共用同一个客户端。
- **自动握手**：`().serve(transport)` 在连接建立时自动完成 `initialize` 请求/响应和 `notifications/initialized` 通知。

### stdio 传输连接

```rust
pub fn connect(config: &McpServerConfig) -> Result<Self, String> {
    let rt = tokio::runtime::Runtime::new().map_err(...)?;
    let service = rt.block_on(async {
        let mut cmd = Command::new(command);
        cmd.args(args);
        for var in env_vars {
            if let Some((k, v)) = var.split_once('=') { cmd.env(k, v); }
        }
        let transport = TokioChildProcess::new(cmd)
            .map_err(|e| format!("创建 MCP 子进程失败: {}", e))?;
        ().serve(transport).await.map_err(...)
    })?;
    Ok(Self { name, rt: Arc::new(rt), service: Arc::new(service) })
}
```

`TokioChildProcess` 是 rmcp 提供的 stdio 传输实现。它将子进程的 stdin/stdout 封装为异步读写流，自动处理 JSON-RPC 消息的行分割。

### SSE 传输连接

```rust
pub fn connect_sse(config: &McpServerConfig) -> Result<Self, String> {
    let rt = tokio::runtime::Runtime::new()?;
    let service = rt.block_on(async {
        let transport = StreamableHttpClientTransport::from_uri(url.to_string());
        ().serve(transport).await.map_err(...)
    })?;
    Ok(Self { name, rt: Arc::new(rt), service: Arc::new(service) })
}
```

`StreamableHttpClientTransport` 是 rmcp 的 HTTP SSE 传输实现，使用 `reqwest` 作为 HTTP 客户端，支持 `text/event-stream` 的长连接。

### 工具发现

```rust
pub fn list_tools(&self) -> Result<Vec<McpToolDefinition>, String> {
    let tools = self.rt.block_on(self.service.list_all_tools())?;
    Ok(tools.into_iter().map(|t| McpToolDefinition {
        server_name: self.name.clone(),
        name: t.name.to_string(),
        description: t.description.unwrap_or_default().to_string(),
        input_schema: Value::Object((*t.input_schema).clone()),
    }).collect())
}
```

`list_all_tools()` 会处理分页（若有 `next_cursor` 则自动请求下一页），返回完整的工具列表。每个 `Tool` 的 schema 从 `Arc<JsonObject>` 克隆为 `Value::Object`，方便后续序列化为 LLM 兼容的工具描述。

### 工具调用

```rust
pub fn call_tool(&self, tool_name: &str, args: &Value) -> Result<String, String> {
    let json_map = args.as_object()?;
    let params = CallToolRequestParams::new(tool_name.to_string())
        .with_arguments(json_map.clone());
    let result: CallToolResult = self.rt.block_on(
        self.service.call_tool(params)
    )?;
    // 提取文本内容
    let text_parts: Vec<String> = result.content.iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.clone()),
            RawContent::Resource(r) => match &r.resource {
                ResourceContents::TextResourceContents { text, .. } => Some(text.clone()),
                _ => None,
            },
            _ => None,
        }).collect();
    Ok(text_parts.join("\n"))
}
```

`CallToolRequestParams::new(name).with_arguments(args)` 是 rmcp 1.7.0 的 builder 模式。因为 `CallToolRequestParams` 标注了 `#[non_exhaustive]`，不能使用结构体字面量构造，必须使用 builder。

调用结果中的 `content` 数组包含服务器返回的 `TextContent` 或 `ResourceContents`。代码会提取所有文本片段，用换行拼接；若没有文本内容则回退到序列化完整结果。

### McpRegistry — 连接管理

`McpRegistry` 管理所有 MCP 连接，负责批量初始化和工具发现：

```
McpRegistry
├── clients: Vec<McpClient>          # 所有连接成功的客户端
└── tools: Vec<(usize, McpToolDef)>  # (client_index, tool_definition)
```

- `McpRegistry::new(servers)` — 遍历配置，跳过禁用的，按传输类型分派，连接失败仅打印警告不阻塞启动
- `McpRegistry::for_agent(agent_config, global_servers)` — 支持多 Agent 的服务器隔离；若 Agent 指定了专属 MCP 服务器则使用专属列表，否则使用全局列表

---

## MCP 工具集成到 AI 工具系统

MCP 工具要能被 LLM 调用，必须适配为 claw 的 `ClawTool` trait。

### ClawTool trait

```rust
pub trait ClawTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameter_schema(&self, enabled_cli_tools: &[&str]) -> Value;
    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, String>;
}
```

### McpToolWrapper — MCP 工具适配器

```rust
impl ClawTool for McpToolWrapper {
    fn name(&self) -> &str { &self.definition.name }

    fn description(&self) -> &str { &self.definition.description }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        let schema = &self.definition.input_schema;
        let mut adapted = schema.clone();
        if adapted.get("additionalProperties").is_none() {
            adapted["additionalProperties"] = Value::Bool(false);
        }
        adapted
    }

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        self.client.call_tool(&self.definition.name, args)
    }
}
```

关键点：

- **名称和描述**：直接使用 MCP 服务器定义的 `name` 和 `description`
- **参数 Schema**：MCP 工具使用 JSON Schema，这与 OpenAI 兼容的 function calling 格式天然匹配。唯一补充的是 `additionalProperties: false`，开启 strict mode 避免 LLM 产生未定义的参数。
- **执行**：通过 `McpClient::call_tool()` 将调用转给 MCP 服务器，返回文本结果。

### 工具注册

在 `ToolRegistry` 中，MCP 工具与内置工具合并在一起：

```rust
pub fn with_mcp(mcp_registry: &McpRegistry) -> Self {
    let mut reg = Self::new();
    for (client_idx, tool_def) in &mcp_registry.tools {
        if let Some(client) = mcp_registry.clients.get(*client_idx) {
            reg.tools.push(Box::new(McpToolWrapper::new(
                tool_def.clone(), client.clone(),
            )));
        }
    }
    reg
}
```

所有工具（内置 + MCP）统一通过 `enabled_schemas()` 生成 LLM 兼容的 function calling 描述：

```rust
pub fn enabled_schemas(&self, enabled: Option<&HashSet<String>>) -> Vec<Value> {
    self.tools.iter().map(|tool| serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name(),
            "description": tool.description(),
            "parameters": tool.parameter_schema(&enabled_cli),
        }
    })).collect()
}
```

对 LLM 来说，MCP 工具和内置工具没有任何区别——都是 `type: function` 的 JSON schema。

---

## CLI 管理命令

### 列出所有 MCP 服务器

```
i-rs-claw mcp --list
```

输出显示配置来源（`配置`/`插件`）、状态和命令：

```
MCP 服务器 (配置 2 个 + 插件 1 个):

  hello-mcp             [配置]  [enabled]  /tmp/hello-mcp
  filesystem            [配置]  [enabled]  npx ...
  plugin:calculator-mcp [插件]  [enabled]  calculator-mcp
```

### 测试 MCP 连接

```
i-rs-claw mcp --check hello-mcp
```

该命令执行完整的连接测试：

1. 查找配置（支持按 `name` 或 `plugin:name` 前缀匹配）
2. 调用 `McpClient::connect()` 建立连接
3. 自动完成 initialize 握手
4. 调用 `list_tools()` 发现工具并打印

```
正在连接 MCP 服务器 'hello-mcp'...
  传输: stdio
  命令: /tmp/hello-mcp
  ✓ 初始化成功
  ✓ 发现 1 个工具:
    - hello: 向某人打招呼，返回个性化的问候语
```

### 启用/禁用

```
i-rs-claw mcp --enable filesystem
i-rs-claw mcp --disable filesystem
```

---

## 编写自己的 MCP 插件

claw 自带两个完整的插件示例，位于 `examples/plugins/`：

- `hello-mcp/` — 最简单入门：一个问候工具
- `calculator-mcp/` — 多工具示例：6 个数学运算工具

下面以 hello-mcp 为例，说明编写 MCP 服务器需要做什么。

### MCP 协议基础

MCP 服务器本质上是一个 JSON-RPC 2.0 over stdio 的服务：

```
请求 →  stdin:   {"jsonrpc":"2.0","id":1,"method":"tools/list"}
响应 ←  stdout:  {"jsonrpc":"2.0","id":1,"result":{"tools":[...]}}
日志 →  stderr:  [hello-mcp] 日志信息
```

服务器只需实现三个方法：

| 方法 | 用途 | 是否必须 |
|------|------|----------|
| `initialize` | 返回协议版本和服务器能力 | **是** |
| `notifications/initialized` | 通知（无需响应） | 否，启动后由客户端发送 |
| `tools/list` | 返回工具列表 | **是** |
| `tools/call` | 执行指定工具并返回结果 | **是** |

### 最小实现框架

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<serde_json::Value>,
    method: Option<String>,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

fn main() {
    let tools = vec![ToolDefinition {
        name: "my_tool".into(),
        description: "工具说明".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "param1": {
                    "type": "string",
                    "description": "参数说明"
                }
            },
            "required": ["param1"]
        }),
    }];

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.trim().is_empty() { continue; }

        let req: JsonRpcRequest = serde_json::from_str(&line).unwrap();
        let id = req.id.clone();
        if id.is_none() { continue; }  // 通知，不需要响应

        let result = match req.method.as_deref() {
            Some("initialize") => Ok(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "my-plugin", "version": "0.1.0" }
            })),
            Some("tools/list") => Ok(serde_json::json!({ "tools": tools })),
            Some("tools/call") => {
                let params = req.params.unwrap_or_default();
                let name = params["name"].as_str().unwrap_or("");
                let args = params["arguments"].clone();
                handle_call(name, &args)
            }
            _ => Err("未知方法".into()),
        };

        // 发送 JSON-RPC 响应到 stdout
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result.ok(),
            "error": result.err().map(|e| serde_json::json!({
                "code": -32603, "message": e
            })),
        });
        println!("{}", serde_json::to_string(&response).unwrap());
    }
}
```

### 重要注意事项

1. **`inputSchema` 必须用驼峰命名**：rmcp SDK 的 `Tool` 结构体使用 `#[serde(rename_all = "camelCase")]`，所以 JSON 中必须用 `inputSchema`，而非 `input_schema`。如果自定义序列化结构，记得加上 `#[serde(rename_all = "camelCase")]`。

2. **`params.name` + `params.arguments`**：`tools/call` 请求中，工具名在 `params.name`，参数在 `params.arguments`（都是外层，`arguments` 里没有再嵌套 `name`）。

3. **通知要忽略**：`initialize` 成功后客户端会发送 `notifications/initialized`（无 `id` 字段），服务器必须忽略（不要响应）。

4. **initialize 必须正确**：`protocolVersion` 必须是 `"2024-11-05"`（当前 MCP 协议版本），且 `capabilities` 中声明支持的功能（如 `tools`）。

### 安装插件

构建后将二进制文件和 `plugin.toml` 放到插件目录：

```bash
# 1. 构建
cargo build --release -p hello-mcp

# 2. 复制二进制到 PATH
cp target/release/hello-mcp ~/.cargo/bin/

# 3. 创建插件目录和清单
mkdir -p ~/.i-rs/claw/plugins/hello-mcp
cp plugin.toml ~/.i-rs/claw/plugins/hello-mcp/

# 4. 启动 claw，插件自动加载！
i-rs-claw
```

---

## 调试与排查

### 1. 单独测试 MCP 服务器

在集成到 claw 之前，先用 echo 测试服务器本身是否工作：

```bash
# 测试 tools/list
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run -p hello-mcp

# 测试 tools/call
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"hello","arguments":{"name":"世界"}}}' | cargo run -p hello-mcp
```

### 2. 使用 --check 验证

```bash
i-rs-claw mcp --check hello-mcp
```

如果 `list_tools()` 返回 `Unexpected response type`：

- 检查 `tools/list` 响应的 JSON 格式是否正确
- 确认 `ToolDefinition` 使用了 `#[serde(rename_all = "camelCase")]`（`input_schema` → `inputSchema`）
- 直接 echo 测试看原始响应内容

### 3. 检查 stderr 日志

MCP 服务器输出到 stderr 的内容会被 claw 捕获并显示在终端中，首次启动时关注：

```
⚠ MCP 连接失败 'xxx': ...
⚠ MCP 工具发现失败 'xxx': ...
✓ MCP: 2 个服务器已连接, 7 个工具已发现
```

### 4. 常见错误

| 错误 | 原因 |
|------|------|
| `Unexpected response type` | 服务器返回的 JSON 格式与 rmcp 期望不匹配，通常是字段命名问题（蛇形 vs 驼峰） |
| `TransportClosed` | 子进程意外退出或连接断开 |
| `创建 MCP 子进程失败` | 可执行文件不存在或没有执行权限 |
| `缺少 command 配置` | stdio 模式下 `command` 字段未设置 |

---

## 总结

claw 通过四层架构实现了对 MCP 生态的完整接入：

```
配置层  (config.toml / plugin.toml)     ← 用户如何告诉 claw 要连接什么
   ↓
连接层  (McpClient / rmcp SDK)          ← 建立 MCP 协议通信
   ↓
适配层  (McpToolWrapper → ClawTool)     ← 将 MCP 工具转换为统一工具接口
   ↓
调度层  (ToolRegistry → LLM)            ← 向 AI 展现工具、执行调用
```

这套设计的核心思想是：**让 MCP 工具对 LLM 而言"无感"**。不管是内置的 CLI 工具，还是远程的 MCP 服务器，LLM 看到的都是同一个 function schema，执行和调用的路径完全一致。

得益于此，claw 可以连接任意 MCP 生态中的工具——无论是官方维护的 `@modelcontextprotocol/server-filesystem`，还是社区开发的 `server-sequelize`、`server-puppeteer`，或是你自己一行行写的 JSON-RPC 服务器，都可以在同一个终端 AI 会话中自由组合使用。

### 相关资源

- [MCP 官方协议规范](https://spec.modelcontextprotocol.io)
- [rmcp — Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- 示例插件源码：`crates/claw/examples/plugins/`
- claw 架构文档：[i-rs-claw 架构解析](./i-rs-claw-architecture)
