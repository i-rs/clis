# IrsClawTauri 设计文档

**日期**：2026-06-30
**状态**：已批准，待实现
**路径**：`apps/IrsClawTauri/`

## 目标

为 i-rs 项目提供 Android / Windows / Linux 客户端，填补现有客户端矩阵（macOS/iPad/iOS 用 IrsClawApp，Web 用 dashboard-ui，微信用 IrsClawMiniProgram）的空白。

IrsClawTauri 基于 Tauri 2 + React 19，作为纯 HTTP 客户端连接远程 `claw serve` 后端，与 IrsClawApp 同构但跨平台。全新设计前端，布局参考 IrsClawApp 的 macOS NavigationSplitView 与 iPhone Drawer 模式。

## 决策汇总

| 维度 | 决策 | 理由 |
|------|------|------|
| 与 dashboard-ui 关系 | 全新设计，参考 IrsClawApp 布局 | 避免与 Web SPA 耦合，针对桌面/移动端量身设计 |
| 布局适配 | 单代码库 + CSS 断点自适应（≥768 桌面 / <768 移动） | 功能只写一遍，桌面/移动端同步演进，零漂移 |
| 导航项 | 全保留 6 项（Sessions/Tools/Skills/Plugins/Usage/Agents）；移动端 Chat/Sessions/More/Usage/Settings，More 折叠 Tools/Skills/Plugins/Agents | 三端功能对齐，无功能缺失 |
| 后端连接 | 薄壳纯 HTTP 客户端，fetch 远程 `claw serve` | 最快落地，零 Rust 业务逻辑，远程/本地后端通用 |
| 原生壳能力 | 系统托盘 + 通知 + 文件对话框 + 增强剪贴板 + Deep Link + 开机自启（不要自动更新） | 发挥 Tauri 原生优势，不增加打包复杂度 |
| 状态管理 | Zustand（已在依赖） | 轻量，已在 package.json |
| 样式 | CSS 变量 + 原生 CSS + .module.css（同 dashboard-ui） | 与 Web 端一致，零额外依赖 |
| 路由 | wouter（同 dashboard-ui） | 轻量，熟悉度高 |
| 首期功能范围 | 核心闭环：Chat(SSE/工具/反馈) + Sessions + Agents + Usage + Settings；Tools/Skills/Plugins 占位 | 跑通核心闭环，分阶段交付 |

## 架构

### 整体定位

IrsClawTauri 是 Tauri 2 桌面/移动客户端，纯 HTTP 客户端架构。Rust 侧只做原生壳（窗口、托盘、通知、文件对话框、剪贴板、Deep Link、开机自启），所有数据通过 `fetch` 调用远程 `claw serve` 的 HTTP API。前端是全新设计的 React 19 应用，布局参考 IrsClawApp 的 SwiftUI 视图结构。

### 目录结构

```
apps/IrsClawTauri/
├── src/                          # React 前端（全新）
│   ├── main.tsx                  # 入口
│   ├── App.tsx                   # 根：ThemeProvider + BackendProvider + Router
│   ├── api/                      # API 层（借鉴 dashboard-ui/api.ts 模式）
│   │   ├── client.ts             # authFetch 封装、baseUrl 动态
│   │   ├── sessions.ts           # 会话 CRUD
│   │   ├── chat.ts               # streamChat SSE 解析
│   │   ├── agents.ts             # Agent 增删改查
│   │   ├── stats.ts              # 用量统计
│   │   └── types.ts              # 共享类型
│   ├── store/                    # Zustand 状态
│   │   ├── backend.ts            # 连接状态、baseUrl、token
│   │   ├── session.ts            # 当前会话、消息列表
│   │   └── ui.ts                 # 主题、侧边栏折叠、移动抽屉
│   ├── hooks/
│   │   ├── useMediaQuery.ts      # 断点检测（<768 移动 / ≥768 桌面）
│   │   ├── useTheme.ts
│   │   └── useBackendHealth.ts
│   ├── components/
│   │   ├── shell/                # 布局壳（断点切换）
│   │   │   ├── DesktopShell.tsx  # 侧边栏 + 详情（≥768px）
│   │   │   ├── MobileShell.tsx   # 顶栏 + 底部Tab + 抽屉（<768px）
│   │   │   └── Sidebar.tsx       # Agent chips + Sessions 列表 + Manage 导航
│   │   ├── chat/                 # 共享叶子组件
│   │   │   ├── MessageList.tsx
│   │   │   ├── MessageBubble.tsx
│   │   │   ├── StreamingBubble.tsx
│   │   │   ├── ToolCallCard.tsx
│   │   │   ├── MarkdownRenderer.tsx
│   │   │   └── FloatingInput.tsx # 浮动输入栏（mic + textarea + send 圆钮）
│   │   └── common/               # AgentChip, PulsingDot, EmptyState, Toast
│   ├── pages/
│   │   ├── Chat.tsx
│   │   ├── Sessions.tsx
│   │   ├── Agents.tsx
│   │   ├── Tools.tsx             # 首期占位（"即将推出"）
│   │   ├── Skills.tsx            # 首期占位
│   │   ├── Plugins.tsx           # 首期占位
│   │   ├── Usage.tsx
│   │   └── Settings.tsx          # 后端连接/token/主题/开机自启
│   ├── styles/
│   │   ├── theme.css             # CSS 变量 [data-theme=dark|light]
│   │   └── responsive.css        # 断点工具类
│   └── *.module.css              # 组件级作用域（沿用 dashboard-ui 模式）
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs                # Tauri Builder + 插件注册 + mobile_entry_point
│   │   ├── commands.rs           # invoke 命令（见下）
│   │   └── tray.rs               # 系统托盘 + 连接状态点
│   ├── Cargo.toml                # tauri + 6 插件
│   └── tauri.conf.json           # 窗口、bundle、权限、deep-link 协议
└── package.json                  # react 19 + wouter + zustand（已在）
```

### 布局适配

单代码库 + CSS 断点自适应。顶层 `App.tsx` 用 `useMediaQuery(768)` 选择 `DesktopShell` 或 `MobileShell`，两者共享所有叶子组件（MessageList、FloatingInput、ToolCallCard 等），仅 layout shell 不同。

**桌面（Windows/Linux）≥ 768px** — 对应 IrsClawApp macOS `NavigationSplitView`：
- 左侧 220px 固定侧边栏：i-rs-claw 标题 → Agent chips → Sessions 列表 → Manage 导航组 → 底部主题/设置
- 右侧详情区：顶栏（会话标题 + New 按钮）→ 消息流 → 底部浮动输入栏
- 窗口可缩放，断点以下自动切到移动布局

**移动端（Android）< 768px** — 对应 IrsClawApp iPhone `NavigationStack + Drawer`：
- 顶栏：☰ 抽屉按钮 → 标题 → ＋ New
- 消息流（全屏）
- 浮动输入栏（与桌面共享）
- 底部 5-Tab：Chat / Sessions / More / Usage / Settings
- More 展开抽屉：Agent chips + 导航卡片网格（Tools/Skills/Plugins/Agents）+ 最近会话

### 后端连接

纯 HTTP 客户端，前端 `api/client.ts` 封装 `authFetch`：
- `baseUrl` 从 Zustand `backendStore` 读取（默认 `http://localhost:3000`，可在 Settings 改）
- Bearer token 存 localStorage（同 dashboard-ui）
- SSE 流式聊天复用 dashboard-ui 的 `streamChat` 解析模式

无 Rust 侧业务逻辑，Rust 仅提供原生增强 commands。

### Rust 侧 Tauri commands

```rust
// src-tauri/src/commands.rs — 薄壳，无业务逻辑
#[tauri::command] fn get_backend_url() -> String
#[tauri::command] fn set_backend_url(url: String)
#[tauri::command] fn get_token() -> String
#[tauri::command] fn set_token(token: String)
#[tauri::command] async fn check_backend_health(url: String) -> bool  // Rust reqwest 健康检查
#[tauri::command] async fn copy_image_to_clipboard(path: String)      // 图片写剪贴板
#[tauri::command] async fn open_file_dialog(filters: Vec<Filter>) -> Option<String>
#[tauri::command] fn show_in_folder(path: String)
#[tauri::command] fn set_autostart(enabled: bool)
#[tauri::command] fn is_autostart_enabled() -> bool
```

### 插件清单

| 插件 | 用途 | 平台 |
|------|------|------|
| tauri-plugin-opener | 打开外部链接（已在） | 全平台 |
| tauri-plugin-notification | LLM 完成/工具出错/断连通知 | 全平台 |
| tauri-plugin-dialog | 原生文件对话框（导出/保存/上传） | 全平台 |
| tauri-plugin-clipboard-manager | 复制图片/富文本到剪贴板 | 全平台 |
| tauri-plugin-deep-link | `irsclaw://` 协议注册与唤起 | 全平台 |
| tauri-plugin-autostart | 开机自启（Settings 可开关） | 桌面 |

系统托盘使用 Tauri 2 内置 `TrayIconBuilder` API（不需额外插件）。

### 数据流

```
React (wouter 路由)
  → api/client.ts (Bearer auth, baseUrl from backendStore)
  → fetch http://<backend>/api/{chat,sessions,agents,stats,...}
  → SSE /chat/stream/{id} 解析 token/reasoning/tool_executed/done 事件
  → Zustand store 更新 → 组件重渲染

useMediaQuery(768):
  ≥768 → DesktopShell (Sidebar + Detail)
  <768 → MobileShell (TopBar + BottomTab + Drawer)
共享叶子组件: MessageList / FloatingInput / ToolCallCard / MarkdownRenderer
```

## 组件设计

### 状态层（Zustand）

- **`backendStore`**：`baseUrl`、`token`、`connectionState`（disconnected/connecting/connected/failed）、`errorMessage`。持久化 baseUrl/token 到 localStorage。
- **`sessionStore`**：`currentSession`、`messages[]`、`isProcessing`、`messageVersion`（触发滚动）。SSE 事件直接更新此 store。
- **`uiStore`**：`theme`（dark/light）、`sidebarCollapsed`、`mobileDrawerOpen`、`selectedTab`。持久化 theme 到 localStorage。

### API 层

借鉴 `dashboard-ui/src/api.ts` 的模式，按资源拆分到 `api/` 目录下：
- `client.ts`：`authFetch(path, options)` 封装，自动附加 `Authorization: Bearer <token>`，baseUrl 从 `backendStore` 动态读取。
- `chat.ts`：`streamChat(sessionId, handlers)` — SSE 解析，事件类型：token/reasoning/status/error/new_round/tool_executed/image_generated/done/evaluation/quality_score。返回 `AbortController`。
- `sessions.ts`：`listSessions`/`createSession`/`switchSession`/`getSession`/`deleteSession`/`postFeedback`。
- `agents.ts`：`listAgents`/`createAgent`/`deleteAgent`/`getAgentConfig`/`updateAgent`。
- `stats.ts`：`getStats(period)`。
- `types.ts`：共享 TypeScript 类型（SessionMeta、AgentInfo、ChatMessage、ToolCallEvent、TokenUsage、QualityScore 等）。

### 共享叶子组件

- **`FloatingInput`**：浮动圆角输入栏，mic 按钮（调 Tauri 通知/语音 API）+ auto-grow textarea + 圆形 send 按钮（有内容时高亮）。对应 IrsClawApp `ChatView.floatingInputBar`。
- **`MessageBubble`**：消息气泡，支持 user/assistant/error/image/evaluation/quality/feedback 角色，thumbs up/down 按钮。
- **`StreamingBubble`**：流式响应时的临时气泡，光标动画。
- **`ToolCallCard`**：工具调用卡片，折叠/展开 args 与 result。
- **`MarkdownRenderer`**：Markdown 渲染（react-markdown + remark-gfm，同 dashboard-ui）。
- **`AgentChip`**：Agent 选择芯片，激活态高亮。
- **`EmptyState`**：无会话/断连占位视图。
- **`Toast`**：轻量通知（错误/成功）。

### 布局壳

- **`DesktopShell`**：flex 布局，左侧 `Sidebar`（220px 固定）+ 右侧 `<Switch>` 路由出口。对应 macOS SplitView。
- **`MobileShell`**：flex 列布局，顶栏 + 路由出口 + 底部 `BottomTabBar`。☰ 按钮触发 `Drawer`（sheet 从左滑入）。对应 iPhone Drawer。
- **`Sidebar`**：桌面侧边栏，Agent chips 区 + Sessions 列表（可搜索、可滑动删除）+ Manage 导航组 + 底部主题切换/设置按钮。

## 错误处理

- **后端未连接**：全屏 `disconnectedView`（参考 IrsClawApp），输入后端 URL + token 后重试健康检查。
- **SSE 中断**：前端自动重连 3 次（指数退避），失败后 toast 提示并保留已收消息，`connectionState` 置为 failed。
- **原生命令失败**：Rust 侧返回 `Result<T, String>`，前端 `invoke().catch()` 后 toast。
- **主题**：`document.documentElement.setAttribute('data-theme', theme)` + CSS 变量，`uiStore` 持久化到 localStorage。

## 测试

- **前端**：Vitest（已在 pnpm-workspace catalog: `vitest: 4.1.9`）+ React Testing Library。覆盖 store、api client、关键组件。
- **Rust**：`cargo test -p irsclawtauri`（commands 单测，mock 文件/剪贴板）。
- **手动验证**：
  - `pnpm tauri dev` — 桌面端开发
  - `pnpm tauri android dev` — Android 端开发（需 Android SDK）
- **CI 验收命令**：
  - `vp check`（lint + typecheck + format）
  - `vp test`
  - `cargo check -p irsclawtauri`

## 首期实现范围

**核心闭环（首期交付）**：
1. 后端连接流程：disconnectedView → 输入 URL+token → 健康检查 → connected
2. Chat：发送消息、SSE 流式接收、工具调用卡片、reasoning、图片、反馈
3. Sessions：列表、创建、切换、删除
4. Agents：列表、创建、删除、配置编辑
5. Usage：统计图表
6. Settings：后端 URL/token/主题/开机自启
7. 原生壳：系统托盘、通知、文件对话框、剪贴板、Deep Link、开机自启

**占位（首期不实现，显示"即将推出"）**：
- Tools 列表页
- Skills 列表页
- Plugins 列表页

**后续阶段（不在本 spec 范围）**：
- Tools/Skills/Plugins 完整实现
- 应用自动更新
- 离线缓存
- 内嵌 claw serve 后端

## 不在范围内（YAGNI）

- 内嵌 claw serve 后端（已选薄壳纯 HTTP）
- 应用自动更新（未选）
- 离线缓存（纯 HTTP，离线不可用）
- Tauri mobile 的 Android 子进程（薄壳不需）
- i-rs-api 的 70 个 CLI 工具 REST 端点直接调用（走 claw serve 即可）
- 与 dashboard-ui 代码共享（已选全新设计）

## 依赖清单

**前端（package.json，部分已在）**：
- `react` ^19.2.7 ✓（已在）
- `react-dom` ^19.2.7 ✓（已在）
- `@tauri-apps/api` ^2.11.1 ✓（已在）
- `@tauri-apps/plugin-opener` ^2.5.4 ✓（已在）
- `zustand` ^5.0.14 ✓（已在）
- `wouter` ^3.10.0（需添加，同 dashboard-ui）
- `lucide-react` ^1.16.0（需添加，图标库，同 dashboard-ui）
- `react-markdown` ^10.1.0（需添加，Markdown 渲染）
- `remark-gfm` ^4.0.1（需添加，GFM 支持）
- `@tauri-apps/plugin-notification`（需添加）
- `@tauri-apps/plugin-dialog`（需添加）
- `@tauri-apps/plugin-clipboard-manager`（需添加）
- `@tauri-apps/plugin-deep-link`（需添加）
- `@tauri-apps/plugin-autostart`（需添加）

**Rust（src-tauri/Cargo.toml，部分已在）**：
- `tauri` 2 ✓（已在）
- `tauri-build` 2 ✓（已在）
- `tauri-plugin-opener` 2 ✓（已在）
- `serde` / `serde_json` ✓（已在）
- `tauri-plugin-notification` 2（需添加）
- `tauri-plugin-dialog` 2（需添加）
- `tauri-plugin-clipboard-manager` 2（需添加）
- `tauri-plugin-deep-link` 2（需添加）
- `tauri-plugin-autostart` 2（需添加）
- `reqwest` 0.12（需添加，健康检查）
- `tokio` 1（需添加，async commands）

## 与现有客户端的关系

| 客户端 | 平台 | UI 框架 | 数据源 | 与 IrsClawTauri 关系 |
|--------|------|---------|--------|---------------------|
| IrsClawApp | macOS/iPad/iOS | SwiftUI | claw serve HTTP | **布局参考**（SplitView/Drawer/FloatingInput） |
| dashboard-ui | Web 浏览器 | React SPA | claw serve HTTP | **api.ts 模式参考**（不共享代码） |
| IrsClawMiniProgram | 微信 | WXML+WXSS | claw serve HTTP | 无直接关系 |
| **IrsClawTauri** | **Win/Linux/Android** | **React 19 (Tauri 2)** | **claw serve HTTP** | **本文档** |

所有客户端共享同一套 `claw serve` REST API，遵循 AGENTS.md §7 多端适配规范。
