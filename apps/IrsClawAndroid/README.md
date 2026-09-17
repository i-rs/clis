# IrsClawAndroid

`i-rs-claw` 的原生 Android 客户端（Kotlin + Jetpack Compose），功能对齐 iOS 版 `IrsClawApp`，对接同一 `claw serve` HTTP API。

## 功能

- **Chat** — `POST /api/chat` SSE 流式渲染（逐字节解析、多行 `data:` 按 `\n` 拼接、8KB token 批量刷新）；10 种消息气泡（用户 / 助手 Markdown / 工具调用卡片 / 思考块 / 状态 / 错误 / 评估 / 质量 / 反馈 / 图片）；JSON 语法高亮；点赞点踩（一次性）反馈；token 用量与费用徽章；语音输入（zh-CN 优先，实时转写）
- **会话** — 今天/昨天/本周/更早分组、搜索、长按删除、按 Agent 切换
- **面板** — Tools / Skills / Plugins / Usage（周期切换 + 会话累计用量）
- **智能体** — 新建 / 编辑 / 删除，Provider 选择、模型、系统提示词、启用工具、MCP 与目录详情
- **设置** — 外观（浅色/深色/自动）、多后端配置管理与切换、LLM Provider（8 家预设，PATCH `/api/config`）
- **自适应外壳** — 手机抽屉导航（仿 iPhone），平板/展开宽度双栏布局（仿 iPad split）

## 构建与运行

```bash
cd apps/IrsClawAndroid
./gradlew :app:assembleDebug          # APK: app/build/outputs/apk/debug/
./gradlew :app:testDebugUnitTest      # SSE 解析器 + DTO 单测
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

要求 JDK 17+、Android SDK（compileSdk 36）；`local.properties` 指向 SDK 路径（已被 gitignore）。

## 连接后端

默认连接 `http://127.0.0.1:3000`。真机上请在连接页点 **服务器设置**，把地址改为运行 `claw serve` 的机器局域网地址（如 `http://192.168.1.x:3000`），并填写 Dashboard Auth Token（如已设置）。应用已允许明文 HTTP（cleartext）以支持局域网调试。

## 架构

```
app/src/main/java/me/siwi/irsclaw/
├── IrsClawApplication.kt   # AppContainer 手动 DI（无 Hilt/Retrofit，与 iOS 同等极简）
├── MainActivity.kt         # 主题 + window size class 装配
├── data/
│   ├── model/              # DTO（kotlinx.serialization, snake_case）+ 10 种 UI 气泡模型
│   ├── api/                # ClawApi（OkHttp REST + SSE）+ SseParser（纯函数状态机）
│   └── settings/           # DataStore Preferences（key 与 iOS UserDefaults 对齐）
├── logic/                  # ClawViewModel（连接网关/会话/Agent/SSE 聊天循环）+ 语音输入
└── ui/                     # Compose：theme / components / chat / shell / panels / settings
```

超时策略与 iOS 一致：health 3s、GET·DELETE·PUT 10s、POST 60s、SSE 300s；连接网关 5 次重试、间隔 1s。
