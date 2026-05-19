# claw MCP 插件示例

本目录包含多个 MCP 插件示例，帮助你快速理解如何为 claw 编写插件。

## 目录结构

```
examples/plugins/
├── hello-mcp/          # [完整项目] 最简单的 MCP 服务器：问候工具
│   ├── Cargo.toml      #    Rust 项目配置（仅依赖 serde + serde_json）
│   ├── src/main.rs     #    MCP 服务器实现（~200 行，含详细注释）
│   └── plugin.toml     #    claw 插件清单
├── calculator-mcp/     # [完整项目] 计算器：6 个数学工具
│   ├── Cargo.toml
│   ├── src/main.rs
│   └── plugin.toml
├── filesystem/         # [配置仅] 文件系统操作（指向社区 MCP 包）
│   └── plugin.toml
└── brave-search/       # [配置仅] Brave 搜索引擎
    └── plugin.toml
```

## 示例说明

| 示例 | 类型 | 用途 | 难度 |
|------|------|------|------|
| `hello-mcp` | 完整 Rust 项目 | 演示最简 MCP 结构：注册工具、接收调用、返回结果 | ⭐ |
| `calculator-mcp` | 完整 Rust 项目 | 演示多个工具定义、参数校验、数组参数 | ⭐⭐ |
| `filesystem` | 配置仅 (plugin.toml) | 演示如何引用社区 MCP 包 | ⭐ |
| `brave-search` | 配置仅 (plugin.toml) | 演示环境变量配置 | ⭐ |

## 快速开始

### 方式一：编写自己的 Rust MCP 服务器（推荐）

1. 参考 `hello-mcp` 创建你的项目：

```bash
# 1. 创建项目
cargo init --name my-plugin examples/plugins/my-plugin
cd examples/plugins/my-plugin

# 2. 添加依赖
cargo add serde --features derive
cargo add serde_json

# 3. 参考 hello-mcp/src/main.rs 编写你的 MCP 服务器
```

2. 创建 `plugin.toml`：

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
description = "我的第一个 claw 插件"

[transport]
transport_type = "stdio"
command = "my-plugin"
```

3. 构建并安装：

```bash
cargo build --release
cp target/release/my-plugin ~/.cargo/bin/
cp plugin.toml ~/.i-rs-claw/plugins/my-plugin/
```

4. 启动 claw，插件自动加载！

### 方式二：使用已有的社区 MCP 包

参考 `filesystem/plugin.toml`：

```toml
[plugin]
name = "filesystem"
version = "0.1.0"
description = "文件系统操作"
author = "Model Context Protocol"

[transport]
transport_type = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/workspace"]
```

复制到插件目录即可：

```bash
mkdir -p ~/.i-rs-claw/plugins/filesystem
cp filesystem/plugin.toml ~/.i-rs-claw/plugins/filesystem/
```

### 方式三：SSE 远程服务

支持 SSE（Server-Sent Events）传输类型，可连接远程 MCP 服务：

```toml
[plugin]
name = "weather-api"
version = "0.1.0"
description = "远程天气查询服务"

[transport]
transport_type = "sse"
url = "https://mcp-weather.example.com/sse"
```

## 插件结构要求

```
~/.i-rs-claw/plugins/
└── <plugin-name>/          # 目录名必须和 plugin.toml 中的 name 一致
    └── plugin.toml         # 插件清单文件
```

## 调试技巧

- 插件服务器输出到 stderr 的日志会被 claw 捕获并显示在请求日志中
- 可以使用如下命令在终端单独测试 MCP 服务器：

```bash
# 测试 tools/list
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run -p hello-mcp

# 测试 tools/call
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"arguments":{"name":"hello","arguments":{"name":"世界"}}}}' | cargo run -p hello-mcp
```

## MCP 协议速览

MCP (Model Context Protocol) 使用 JSON-RPC 2.0 over stdio 通信：

```
请求 →  stdin:   {"jsonrpc":"2.0","id":1,"method":"tools/list"}
响应 ←  stdout:  {"jsonrpc":"2.0","id":1,"result":{"tools":[...]}}
日志 →  stderr:  [hello-mcp] 日志信息
```

服务器只需实现三个方法：
- `initialize` — 返回协议版本和服务器能力
- `tools/list` — 返回工具列表（名称、描述、参数 schema）
- `tools/call` — 执行指定工具并返回结果
