# Dashboard Chat API 重设计

&gt; **状态**: 设计稿，待评审
&gt; **范围**: dashboard SSE 聊天流（不涉及 TUI、gateway、MCP）
&gt; **目标**: 与业界标准（OpenAI / Anthropic / Vercel AI SDK）对齐，根治"重复 spawn LLM"问题

---

## 1. 问题陈述

### 1.1 现状缺陷

当前 dashboard 聊天 API 是双端点模式：

```
POST /api/chat              → 保存用户消息，返回 sid
GET  /api/chat/stream/{sid} → 加载历史 → spawn LLM → SSE 流
```

**核心 Bug**: `chat_stream` 每次被调用都会重新触发一次完整的 LLM 调用，无论是否已有回复。
触发场景：
- 用户刷新页面 → 自动重连 → 重复扣费
- 网络抖动 → 客户端重试 → 重复回复被持久化
- 多端同时订阅 → 每端各跑一次 LLM

**根因**: LLM 调用与 HTTP 请求生命周期解耦，由后台 `tokio::spawn` 持有，没有"幂等性"概念。

### 1.2 业界标准

ChatGPT、Claude、Cursor、Vercel AI SDK、OpenAI Chat Completions API **全部**使用：

```
POST /api/chat
Body: { message, idempotency_key }
Response: text/event-stream
  event: token / tool_executed / done
```

四个设计支柱：
1. **请求即流** — POST 响应体直接是 SSE
2. **幂等键** — 客户端生成 UUID，服务端去重
3. **无状态恢复** — `GET /sessions/{sid}` 回拉历史
4. **LLM = HTTP 生命周期** — 连接断开 = 自动 cleanup

---

## 2. 新 API 契约

### 2.1 端点变更

| 操作 | 旧 | 新 |
|------|----|----|
| 发送消息+流 | `POST /api/chat` + `GET /api/chat/stream/{sid}` | **`POST /api/chat`** |
| 加载历史 | `GET /api/sessions/{sid}` | 不变 |
| 创建空会话 | `POST /api/sessions` | 不变 |
| 切换会话 | `POST /api/sessions/{id}/switch` | 不变 |

删除：`GET /api/chat/stream/{session_id}`

### 2.2 请求格式

```http
POST /api/chat HTTP/1.1
Content-Type: application/json
Authorization: Bearer <token>

{
  "message": "用户输入文本",
  "agent_id": "default",
  "session_id": "uuid-of-existing-session"
}
```

字段说明：
- `message` (必填): 用户消息文本
- `agent_id` (可选，默认 "default"): Agent 标识
- `session_id` (可选): 已有会话 ID；省略则自动创建新会话

**不做服务端幂等**。参考 OpenAI / Anthropic / Vercel AI SDK —— 成熟 SSE 流式 API 均不做服务端幂等，理由是：
- SSE 流不可缓存，幂等回放价值低
- 客户端可通过 `GET /sessions/{sid}` 自行 rehydrate
- POST+stream 已经消除了原来 GET 端点重复触发 LLM 的根本 bug

**会话级 streaming flag**（10 行内存守卫）：同一 session 同时只允许一个进行中的流，防止网络重试 / 双击的极端情况。

### 2.3 响应格式

**成功**：HTTP 200，`Content-Type: text/event-stream`

```
event: session
data: {"session_id": "abc-123"}

event: token
data: "你好"

event: reasoning
data: "用户在问..."

event: tool_executed
data: {"name":"i-rs-todo","args":"...","result":"...","step":1,"total_steps":2}

event: new_round
data: ""

event: done
data: {"usage":{"prompt_tokens":120,"completion_tokens":45},"quality":{"score":"0.85","complete":true,"issues":[],"references_valid":true}}

```

**错误**：
- 缺字段 → HTTP 400 `{"success":false,"error":"Missing 'message'"}`
- 未授权 → HTTP 401（已有 auth_guard 处理）
- 同会话已有进行中的流 → HTTP 409 `{"success":false,"error":"会话正在处理中"}`
- LLM 失败 → SSE 流内 `event: error data: "..."`，连接关闭

### 2.4 SSE 事件类型（与现有一致，仅新增 `session`）

| 事件 | data 内容 | 说明 |
|------|----------|------|
| `session` | `{"session_id":"..."}` | **新增**，作为第一个事件，告知客户端会话 ID |
| `token` | 纯文本 | LLM 流式 token |
| `reasoning` | 纯文本 | 推理过程（DeepSeek/Claude） |
| `tool_executed` | JSON | 工具调用结果 |
| `new_round` | 空 | 多轮工具调用分隔 |
| `image_generated` | JSON | 图像生成结果 |
| `evaluation` | JSON | 工具评估 |
| `done` | JSON | 完成事件，含 usage + quality |
| `error` | 纯文本 | 错误事件 |
| `status` | 纯文本 | 状态信息 |

---

## 3. 服务端改动

### 3.1 文件清单

| 文件 | 改动 |
|------|------|
| `crates/claw/src/dashboard/routes.rs` | 删除 `send_message`、`chat_stream`；新增 `chat` handler |
| `crates/claw/src/dashboard/mod.rs` | 路由表：删 `GET /api/chat/stream/{sid}`，`POST /api/chat` 指向新 handler |
| `crates/claw/src/session.rs` | 新增 `streaming_sessions: HashSet<String>` + `set_streaming/clear_streaming/is_streaming` 方法 |

### 3.2 新 `chat` handler 伪代码

```rust
pub async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> axum::response::Response {
    // 1. 校验
    let message = req.message.ok_or_else(|| ...)?;

    // 2. 创建/获取会话
    let mut core = state.core.write().await;
    let agent_id = req.agent_id.unwrap_or("default".into());
    let sid = req.session_id
        .or_else(|| core.session_mgr.current_id().map(String::from))
        .unwrap_or_else(|| core.session_mgr.create_session_for(&agent_id));

    // 3. 会话级 streaming 守卫
    if core.session_mgr.is_streaming(&sid) {
        return HTTP_409;  // 该会话已有进行中的流
    }
    core.session_mgr.set_streaming(&sid);

    // 4. 持久化用户消息 + layered memory
    core.session_mgr.persist_messages(&sid, &[Message::User { text: message }])?;
    core.session_mgr.switch_to(&sid);
    core.agent_store.layered_memory_for_mut(&agent_id).record_user_statement(&message);

    // 5. spawn LLM
    let (llm_tx, rx) = mpsc::unbounded_channel::<LlmEvent>();
    let records = core.session_mgr.load_app_messages(&sid, 50);
    let msgs = core.build_messages_from_log(&records, &agent_id);
    let recent = ...;
    core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
    drop(core);

    // 6. 构造 SSE 流（首事件 = session）
    let stream = build_sse_stream(rx, state.clone(), sid.clone());

    Sse::new(stream).into_response()
}
```

### 3.3 build_sse_stream 的关键变化

1. **第一个事件** `event: session` data: `{"session_id":"..."}`
2. **Done/Error 处理器**：在持久化完成后，调用 `clear_streaming(&sid)` 释放守卫

### 3.4 测试

新增测试（`routes.rs` `#[cfg(test)]` 块）：

| 测试名 | 验证内容 |
|--------|---------|
| `test_chat_creates_session` | 不传 session_id 时自动创建 |
| `test_chat_persists_user_message` | 用户消息被持久化 |
| `test_chat_rejects_concurrent_stream` | 同会话已 streaming 时返回 409 |
| `test_chat_done_clears_streaming` | Done 事件后释放 streaming flag |
| `test_chat_missing_message` | 400 错误 |
| `test_chat_done_persists_assistant` | Done 事件后 assistant 消息入库 |
| `test_chat_done_persists_quality` | Done 事件后 quality 消息入库 |

---

## 4. 客户端改动

### 4.1 Web (`dashboard-ui/src/api.ts`)

**删除**：`sendMessage`、`streamChat` (两个独立函数)

**新增**：合并后的 `chatStream`

```typescript
export interface ChatRequest {
  message: string
  agent_id?: string
  session_id?: string
}

export function chatStream(
  req: ChatRequest,
  handlers: SseEventHandler,
): AbortController {
  const controller = new AbortController()

  authFetch('/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
    signal: controller.signal,
  }).then(async (response) => {
    if (response.status === 409) {
      handlers.onError?.('该会话已有进行中的请求')
      return
    }
    const reader = response.body?.getReader()
    if (!reader) return
    // ... 沿用 streamChat 的 SSE 解析逻辑 ...
    // 新增处理 event: session
    // case 'session': handlers.onSession?.(JSON.parse(data)); break
  })

  return controller
}
```

`Chat.tsx` 改动：
```typescript
const handleSend = async () => {
  const controller = chatStream(
    {
      message: text,
      agent_id: selectedAgent,
      session_id: sessionId,
    },
    {
      onSession: (data) => setSessionId(data.session_id),
      onToken: (t) => { ... },
      onDone: (usage, quality) => { ... },
      // ...
    },
  )
}
```

**估算**: ~50 行新增 / ~30 行删除

### 4.2 iOS (`ClawService.swift`)

`sendMessage` 与 `streamChat` 合并：

```swift
func sendMessage(_ text: String) {
    guard !text.isEmpty, !isProcessing else { return }
    isProcessing = true

    Task {
        let body: [String: Any] = [
            "message": text,
            "agent_id": currentAgentId,
            "session_id": currentSessionId ?? "",
        ]
        guard let bodyData = try? JSONSerialization.data(withJSONObject: body) else { return }

        var request = URLRequest(url: URL(string: "\(baseURL)/api/chat")!)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = bodyData
        request.timeoutInterval = 300
        addAuthHeader(&request)

        // URLSession.bytes 原生支持 POST 流式响应
        let (bytes, response) = try await URLSession.shared.bytes(for: request)
        guard let http = response as? HTTPURLResponse else { return }
        if http.statusCode == 409 {
            // 该会话已有进行中的请求
            return
        }
        // 沿用现有 SSE 解析逻辑
        for try await line in bytes.lines {
            // 解析 event:/data: ...
        }
    }
}
```

**估算**: ~60 行重写 / ~80 行删除

### 4.3 MiniProgram (`utils/api.js`)

`sendMessage` + `streamChat` 合并为 `chatStream`：

```javascript
function chatStream(opts, handlers) {
  handlers = handlers || {}
  var url = baseUrl() + '/chat'
  var body = {
    message: opts.message,
  }
  if (opts.agent_id && opts.agent_id !== 'default') body.agent_id = opts.agent_id
  if (opts.session_id) body.session_id = opts.session_id

  // wx.request POST + enableChunked (基础库 3.16.1 已确认可用)
  var task = wx.request({
    url: url,
    method: 'POST',
    header: Object.assign({ 'Content-Type': 'application/json' }, authHeader()),
    data: body,
    enableChunked: true,
    timeout: STREAM_TIMEOUT,
    success: function (res) {
      if (res.statusCode === 409) {
        if (handlers.onError) handlers.onError({ error: '该会话已有进行中的请求' })
        return
      }
      // ... 沿用 streamChat 的成功逻辑 ...
    },
    fail: function (err) { /* ... */ },
  })

  task.onChunkReceived(function (res) {
    // 沿用 feedChunk/processLine 解析
  })

  return { abort: function () { task.abort() } }
}
```

**估算**: ~40 行重写 / ~30 行删除

**注意**：`wx.request` 在 POST + `enableChunked: true` 时，部分客户端在 `success` 才一次性返回响应体。需要用 `onChunkReceived` 接收流式数据 + 在 `success` 中只处理终止事件。已有此模式的现成代码（当前 `streamChat`），可直接迁移。

---

## 5. 迁移步骤

按以下顺序执行，每步可独立验证：

### Phase 1: 服务端（不破坏旧 API）

1. **新增 `POST /api/chat` 新 handler（命名 `chat_v2` 或直接覆盖）**
   - 实现幂等缓存
   - 实现合并逻辑
   - 新增 `event: session` 首事件
2. **保留旧端点** `POST /api/chat/send` (兼容旧 client) 和 `GET /api/chat/stream/{sid}` 一周
3. **测试**：`cargo test -p i-rs-claw --features dashboard`
4. **Commit**: `feat(claw/dashboard): add unified POST /api/chat with idempotency`

### Phase 2: Web 客户端

1. 修改 `api.ts`：新增 `chatStream`，标记旧函数 deprecated
2. 修改 `Chat.tsx`：用 `chatStream` 替换 `sendMessage` + `streamChat`
3. **本地测试**：手动发送消息、刷新页面、多 tab
4. **Commit**: `refactor(dashboard-ui): migrate to unified chat endpoint`

### Phase 3: iOS 客户端

1. 修改 `ClawService.swift`：合并 sendMessage + streamChat
2. **Commit**: `refactor(ios): migrate to unified chat endpoint`

### Phase 4: MiniProgram 客户端

1. 修改 `api.js`：合并为 `chatStream`
2. **Commit**: `refactor(miniprogram): migrate to unified chat endpoint`

### Phase 5: 清理

1. **删除旧端点** `GET /api/chat/stream/{sid}` 和 `send_message` handler
2. **删除旧客户端代码**（已 deprecated 的 `sendMessage`、`streamChat`）
3. **Commit**: `chore(claw/dashboard): remove deprecated dual-endpoint chat API`

---

## 6. 测试计划

### 6.1 单元测试（Rust）

- [ ] `test_chat_creates_session_when_omitted`
- [ ] `test_chat_uses_provided_session_id`
- [ ] `test_chat_persists_user_message`
- [ ] `test_chat_emits_session_event_first`
- [ ] `test_chat_done_persists_assistant_and_quality`
- [ ] `test_chat_idempotency_replay_same_key`
- [ ] `test_chat_idempotency_different_keys_spawn_independently`
- [ ] `test_chat_idempotency_ttl_expiry`
- [ ] `test_chat_missing_message_returns_400`
- [ ] `test_chat_missing_idempotency_key_returns_400`
- [ ] `test_chat_stream_can_be_aborted_mid_stream`

### 6.2 集成测试（手工）

- [ ] Web 发送消息 → 流式显示 → 持久化正确
- [ ] Web 刷新页面 → 不触发重复 LLM
- [ ] Web 双击发送 → 第二次返回缓存流（同一 idem_key）
- [ ] iOS 同上三项
- [ ] MiniProgram 同上三项
- [ ] 服务端 kill -9 → 客户端连接断开 → 重启后 GET session 能 rehydrate
- [ ] 两端同时发送到同一 session_id → 各自独立完成

### 6.3 回归测试

- [ ] `cargo test -p i-rs-claw --features dashboard`（应保持 379+ 通过）
- [ ] `cargo clippy -p i-rs-claw --features dashboard -- -D warnings`
- [ ] TUI 聊天功能不受影响（`spawn_chat_for_async` 未改）

---

## 7. 回滚方案

每个 Phase 都可独立回滚：

- Phase 1: `git revert` 新 handler commit，旧端点未动
- Phase 2-4: 各客户端独立，互不影响
- Phase 5: 不可逆（旧代码已删），但在前 4 Phase 稳定 1 周后再做

---

## 8. 范围外（明确不做）

1. **WebSocket 改造** — 当前 SSE POST 已足够，多端广播是后续 topic
2. **TUI 聊天流程** — TUI 直接走 `AppCore.chat_loop`，不涉及 HTTP
3. **Gateway（Telegram/WeChat）** — 已有独立的持久化路径，沿用 `persist_messages`
4. **认证改造** — 沿用现有 Bearer token
5. **API 版本化** — 不引入 `/v2/`，直接替换（旧 API 保留 1 周兼容期）

---

## 9. 风险与缓解

| 风险 | 缓解 |
|------|------|
| MiniProgram POST chunked 在低端设备不稳定 | 已确认 libVersion 3.16.1；可加 fallback：POST 失败时自动降级到旧双端点（保留一周） |
| 幂等缓存内存增长 | DashMap + TTL 24h + 后台定期清理；UUID 字符串作为 key，预估单条 &lt; 100KB SSE 字节 |
| iOS URLSession.bytes POST 响应慢首字节 | 实测与 GET 相同；token 间隔由 LLM 决定 |
| 客户端不生成 idem_key | 服务端兜底：缺 idem_key 返回 400，强制要求 |

---

## 10. 决策记录

- **2025-XX-XX**: 评审通过方案 D（重写为单端点 + 幂等键），否决 C1/C2/C3（在错误设计上打补丁）
- **2025-XX-XX**: 确认 MiniProgram libVersion 3.16.1，POST chunked 可用
- **2025-XX-XX**: 决定保留旧端点 1 周作为兼容期（Phase 5 在 Phase 1-4 稳定后执行）
