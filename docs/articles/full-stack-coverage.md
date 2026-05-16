# 从终端到云端 — CLI + API + 浏览器的全栈覆盖

## 一个数据，三种入口

数据应该是自由的。它不应该被锁定在特定的界面或协议中。

i-rs 的数据生态围绕一个核心理念构建：**同一份数据，以最适合的方式访问**。

```
┌─────────────┐    ┌──────────────┐    ┌──────────────────┐
│   终端 CLI   │    │  REST API    │    │ 浏览器扩展       │
│   i-rs-kv    │    │  i-rs-api    │    │  Chrome Extension │
│              │    │              │    │                  │
│  极速响应    │    │  程序化集成  │    │  可视化操作      │
│  脚本友好    │    │  远程访问    │    │  日常使用        │
│  Unix哲学    │    │  Webhook     │    │  一键操作        │
└──────┬───────┘    └──────┬───────┘    └────────┬─────────┘
       │                  │                      │
       └──────────────────┴──────────────────────┘
                         │
                  ┌──────▼───────┐
                  │  ~/.config/  │
                  │  i-rs/*.json │
                  │  统一存储    │
                  └──────────────┘
```

## 终端 CLI — 速度与组合性

CLI 是 i-rs 的原生形态，也是最强形态。70 个工具覆盖了个人数据的方方面面，每个工具都拥有完整的 CRUD 命令集。

CLI 的优势在于：

- **无需启动** — 命令即执行，毫秒级响应
- **可组合** — 管道、重定向、脚本，Unix 哲学的极致发挥
- **远程友好** — SSH 会话中完美工作
- **自动化** — cron 定时任务、CI/CD 集成

```bash
# 一行命令完成复杂查询
i-rs-kv list --json | jq '.[] | select(.tags | contains(["config"]))'

# 管道组合多个工具
i-rs-weight list --json | python3 -c "import json,sys; data=json.load(sys.stdin); print(sum(e['weight'] for e in data)/len(data))"
```

## REST API — 当 i-rs 装上 Axum

`i-rs-api` 是一个基于 Axum 的 REST API 服务，将 70 个 CLI 工具的能力暴露为 HTTP 端点。

启动方式：

```bash
i-rs-api serve
# Server running at http://localhost:8080
```

API 路径设计与 CLI 子命令一一对应：

```
GET    /api/{tool}          # 相当于 CLI 的 list
GET    /api/{tool}/{id}     # 相当于 CLI 的 get
POST   /api/{tool}          # 相当于 CLI 的 add
PUT    /api/{tool}/{id}     # 相当于 CLI 的 update
DELETE /api/{tool}/{id}     # 相当于 CLI 的 delete
```

这意味任何支持 HTTP 的应用程序都可以与 i-rs 数据交互：

- **Web 应用** — 前端直接调用 API 呈现数据仪表盘
- **移动端** — iOS/macOS 快捷指令触发 API 调用
- **集成场景** — Zapier、n8n、Make 等自动化平台的 HTTP 模块
- **AI Agent** — AI 通过 HTTP 工具调用直接操作数据

```bash
# curl 即可完成数据操作
curl -X POST http://localhost:8080/api/weight \
  -H "Content-Type: application/json" \
  -d '{"weight": 72.5, "date": "2026-05-15"}'

curl http://localhost:8080/api/kv | jq
```

## 浏览器扩展 — Native Messaging 的桥梁

对于 kv 这样需要频繁查看和操作的数据，i-rs 提供了 Chrome 浏览器扩展。

通过 Chrome Native Messaging 技术，浏览器扩展可以直接调用系统上安装的 i-rs-kv 二进制：

```
┌───────────────┐    Native Messaging    ┌────────────────┐
│ Chrome 扩展   │ ◄──── JSON over stdio ─►│  i-rs-native-msg │
│ (popup UI)    │                        │  (Rust 宿主)     │
└───────────────┘                        └────────┬─────────┘
                                                  │
                                                  ▼
                                         ┌────────────────┐
                                         │  ~/.config/    │
                                         │  i-rs/kv.json  │
                                         └────────────────┘
```

支持的操作：

| 操作 | 描述 | 示例 |
|------|------|------|
| `List` | 列出所有条目 | `{"action": "List"}` |
| `Get` | 获取指定 key | `{"action": "Get", "key": "api-token"}` |
| `Set` | 设置键值对 | `{"action": "Set", "key": "token", "value": "abc"}` |
| `Delete` | 删除条目 | `{"action": "Delete", "key": "temp"}` |
| `Search` | 搜索 | `{"action": "Search", "query": "config"}` |

这意味着你可以**在浏览器弹窗中直接查询和管理数据**，无需打开终端。

## 三者的关系

这三种访问方式并非替代关系，而是互补关系：

| 场景 | 推荐方式 | 理由 |
|------|---------|------|
| 日常快速操作 | CLI | 毫秒级，无 overhead |
| 定时任务/脚本 | CLI | cron 可直接调用 |
| Web 应用集成 | REST API | HTTP 通用协议 |
| 远程访问 | REST API | 通过网络调用 |
| 浏览器内操作 | 浏览器扩展 | 一键触达 |
| 数据迁移 | CLI (export/import) | 批量处理 |

## 更广阔的可能

i-rs 的数据都存储在 `~/.config/i-rs/*.json` 中，标准的 JSON 格式意味着**不锁定在任何特定工具链中**。你可以：

- 用 Python pandas 读取体重数据做分析
- 用 Grafana 连接 API 制作仪表盘
- 用 Swift 快捷指令触发数据记录
- 用 Obsidian 插件引用笔记数据

数据是你的，入口随你选。这是 i-rs 的设计承诺。
