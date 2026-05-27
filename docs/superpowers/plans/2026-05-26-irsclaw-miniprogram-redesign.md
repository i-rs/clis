# IrsClawMiniProgram Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the WeChat Mini Program with a consistent WeChat-native design system (#EDEDED background, #07C160 brand green, white cards, light theme throughout)

**Architecture:** 7 pages total. Chat (index) is the main page with custom navbar + sidebar drawer. All secondary pages (sessions/tools/skills/plugins/agents/settings) are navigated via `wx.navigateTo`. Unified design tokens via `app.wxss`. Minimal changes to `app.js` (API client stays, CSS-driven redesign).

**Tech Stack:** WeChat Mini Program (JS/WXML/WXSS/JSON), SDK 3.16.1+

---

### Task 1: Foundation — Global Styles & Config

**Files:**
- Modify: `apps/IrsClawMiniProgram/app.wxss`
- Modify: `apps/IrsClawMiniProgram/app.json`
- Modify: `apps/IrsClawMiniProgram/app.js` (minor: remove unused data + theme code)

- [ ] **Step 1: Rewrite `app.wxss` with design tokens**

```wxss
/* WeChat Design System */
page {
  --page-bg: #EDEDED;
  --card-bg: #FFFFFF;
  --brand: #07C160;
  --brand-press: #06AD56;
  --danger: #FA5151;
  --text-primary: #1A1A1A;
  --text-secondary: #666666;
  --text-tertiary: #999999;
  --divider: #EBEBEB;
  --mask: rgba(0, 0, 0, 0.5);
  --user-bubble: #07C160;
  --ai-bubble: #FFFFFF;
  --ai-bubble-border: #E5E5E5;
  --list-icon-orange: rgba(255, 149, 0, 0.12);
  --list-icon-green: rgba(52, 199, 89, 0.12);
  --list-icon-purple: rgba(88, 86, 214, 0.12);

  font-family: -apple-system, BlinkMacSystemFont, 'Helvetica Neue', Arial, sans-serif;
  font-size: 16px;
  line-height: 1.47;
  color: var(--text-primary);
  background: var(--page-bg);
  -webkit-font-smoothing: antialiased;
}

/* ===== Shared Components ===== */

/* Standard List Item */
.list-item {
  display: flex;
  align-items: center;
  padding: 14px 16px;
  gap: 12px;
  background: var(--card-bg);
  border-bottom: 0.5px solid var(--divider);
  position: relative;
}
.list-item:active {
  background: #F2F2F2;
}
.list-item:last-child {
  border-bottom: none;
}

.list-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  flex-shrink: 0;
}
.list-icon.orange { background: var(--list-icon-orange); }
.list-icon.green { background: var(--list-icon-green); }
.list-icon.purple { background: var(--list-icon-purple); }
.list-icon.blue { background: rgba(0, 122, 255, 0.12); }

.list-body {
  flex: 1;
  overflow: hidden;
}

.list-title {
  font-size: 16px;
  font-weight: 400;
  color: var(--text-primary);
  display: block;
  margin-bottom: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.list-desc {
  font-size: 14px;
  color: var(--text-secondary);
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.list-arrow {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  font-size: 16px;
}

/* List Page Shell */
.list-page {
  min-height: 100vh;
  background: var(--page-bg);
}

.list-scroll {
  flex: 1;
  overflow-y: auto;
}

/* Section Group */
.section-group {
  margin-top: 10px;
}
.section-group:first-child {
  margin-top: 0;
}

.section-header {
  font-size: 13px;
  font-weight: 400;
  color: var(--text-tertiary);
  padding: 12px 16px 6px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.section-body {
  background: var(--card-bg);
  border-top: 0.5px solid var(--divider);
  border-bottom: 0.5px solid var(--divider);
}

/* Card */
.card {
  background: var(--card-bg);
  border-radius: 0;
  margin: 0;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 100px 20px;
  text-align: center;
}

.empty-icon {
  width: 56px;
  height: 56px;
  font-size: 28px;
  margin-bottom: 16px;
  opacity: 0.4;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.empty-sub {
  font-size: 14px;
  color: var(--text-secondary);
}

/* Button */
.btn-primary {
  background: var(--brand);
  color: #fff;
  border-radius: 6px;
  padding: 14px;
  text-align: center;
  font-size: 17px;
  font-weight: 500;
}
.btn-primary:active {
  background: var(--brand-press);
}

/* Card with rounded corners (for settings groups) */
.card-rounded {
  background: var(--card-bg);
  border-radius: 10px;
  overflow: hidden;
  margin: 0 16px;
}

/* Input row */
.input-row {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 0.5px solid var(--divider);
}
.input-row:last-child {
  border-bottom: none;
}

.input-label {
  font-size: 16px;
  color: var(--text-primary);
  width: 90px;
  flex-shrink: 0;
}

.input-field {
  flex: 1;
  font-size: 16px;
  color: var(--text-primary);
  text-align: right;
  border: none;
  outline: none;
  background: transparent;
  padding: 0;
}

/* Status indicator */
.status-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #FA5151;
  flex-shrink: 0;
}
.status-dot.online {
  background: var(--brand);
}

.status-text {
  font-size: 16px;
  color: var(--text-primary);
  flex: 1;
}

/* Info row */
.info-row {
  display: flex;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 0.5px solid var(--divider);
  font-size: 16px;
  color: var(--text-primary);
}
.info-row:last-child {
  border-bottom: none;
}
.info-val {
  color: var(--text-secondary);
}
```

- [ ] **Step 2: Update `app.json` for light theme navbar**

```json
{
  "pages": [
    "pages/index/index",
    "pages/sessions/index",
    "pages/tools/index",
    "pages/skills/index",
    "pages/plugins/index",
    "pages/settings/index",
    "pages/agents/index"
  ],
  "window": {
    "backgroundTextStyle": "dark",
    "navigationBarBackgroundColor": "#EDEDED",
    "navigationBarTitleText": "IrsClaw",
    "navigationBarTextStyle": "black"
  },
  "style": "v2",
  "sitemapLocation": "sitemap.json",
  "lazyCodeLoading": "requiredComponents"
}
```

- [ ] **Step 3: Keep `app.js` as-is** (API client is well-structured, no changes needed)

---

### Task 2: Chat Page — WXML Template

**Files:**
- Rewrite: `apps/IrsClawMiniProgram/pages/index/index.wxml`

- [ ] **Step 1: Write the complete template**

```wxml
<view class="chat-page">
  <!-- Custom NavBar -->
  <view class="navbar">
    <view class="nav-btn" bindtap="onToggleMenu">
      <text class="nav-icon">☰</text>
    </view>
    <view class="nav-title-wrap">
      <text class="nav-title">{{currentAgentId || 'IrsClaw'}}</text>
    </view>
    <view class="nav-btn nav-btn-brand" bindtap="onNewChat">
      <text class="nav-icon-plus">+</text>
    </view>
  </view>

  <!-- Messages -->
  <scroll-view scroll-y class="msg-list" scroll-into-view="bottom" scroll-with-animation>
    <block wx:if="{{messages.length === 0}}">
      <view class="empty-state">
        <view class="empty-avatar">
          <text>AI</text>
        </view>
        <text class="empty-title">开始对话</text>
        <text class="empty-sub">向 AI 助手提问或描述你的需求</text>
      </view>
    </block>

    <block wx:else>
      <view wx:for="{{messages}}" wx:for-index="idx" wx:key="id">
        <!-- User Message -->
        <view wx:if="{{item.role === 'user'}}" class="msg-row msg-user">
          <view class="msg-bubble user-bubble">
            <text class="msg-text">{{item.content}}</text>
          </view>
          <view class="msg-avatar user-avatar">
            <text>U</text>
          </view>
        </view>

        <!-- AI Message -->
        <view wx:elif="{{item.role === 'assistant' || item.role === 'ai'}}" class="msg-row msg-ai">
          <view class="msg-avatar ai-avatar">
            <text>AI</text>
          </view>
          <view class="msg-content">
            <!-- Reasoning block -->
            <view wx:if="{{item.reasoning}}" class="fold-block reasoning-block">
              <view class="fold-header" bindtap="onToggleReasoning" data-id="{{item.id}}">
                <text class="fold-icon">{{item._showReasoning ? '▼' : '▶'}}</text>
                <text class="fold-label">思考过程</text>
              </view>
              <view wx:if="{{item._showReasoning}}" class="fold-body">
                <text class="fold-text">{{item.reasoning}}</text>
              </view>
            </view>

            <!-- Tool calls block -->
            <view wx:if="{{item.tool_calls && item.tool_calls.length > 0}}" class="fold-block tool-block">
              <view class="fold-header" bindtap="onToggleTools" data-id="{{item.id}}">
                <text class="fold-icon">{{item._showTools ? '▼' : '▶'}}</text>
                <text class="fold-label">使用 {{item.tool_calls.length}} 个工具</text>
              </view>
              <view wx:if="{{item._showTools}}" class="fold-body">
                <view wx:for="{{item.tool_calls}}" wx:for-item="tool" wx:key="index" class="tool-item">
                  <text class="tool-call-name">{{tool.name || tool.function?.name}}</text>
                  <text class="tool-call-args">{{tool.arguments || JSON.stringify(tool.function?.arguments)}}</text>
                </view>
              </view>
            </view>

            <!-- Thinking block (alias for reasoning) -->
            <view wx:if="{{item.thinking}}" class="fold-block reasoning-block">
              <view class="fold-header" bindtap="onToggleThinking" data-id="{{item.id}}">
                <text class="fold-icon">{{item._showThinking ? '▼' : '▶'}}</text>
                <text class="fold-label">思考过程</text>
              </view>
              <view wx:if="{{item._showThinking}}" class="fold-body">
                <text class="fold-text">{{item.thinking}}</text>
              </view>
            </view>

            <view class="ai-bubble">
              <text class="msg-text">{{item.content}}</text>
            </view>
          </view>
        </view>
      </view>
    </block>

    <view id="bottom" class="scroll-bottom"></view>
  </scroll-view>

  <!-- Input Bar -->
  <view class="input-bar">
    <view class="input-wrap">
      <textarea class="input-box" value="{{inputText}}" bindinput="onInput" placeholder="输入消息..." disabled="{{!isConnected}}" maxlength="4000" auto-height show-confirm-bar="{{false}}" cursor-spacing="12" />
      <view class="send-btn {{inputText.trim() ? 'send-active' : ''}}" bindtap="onSend">
        <text>↑</text>
      </view>
    </view>
  </view>

  <!-- Sidebar Drawer -->
  <view class="drawer-mask {{menuOpen ? 'show' : ''}}" bindtap="onCloseMenu"></view>
  <view class="drawer {{menuOpen ? 'open' : ''}}">
    <view class="drawer-head">
      <text class="drawer-brand">IrsClaw</text>
    </view>
    <view class="drawer-body">
      <view class="drawer-group">
        <view class="drawer-item" bindtap="onGoSessions">
          <text class="drawer-icon">💬</text>
          <text class="drawer-label">会话历史</text>
        </view>
        <view class="drawer-item" bindtap="onGoTools">
          <text class="drawer-icon">🔧</text>
          <text class="drawer-label">工具</text>
        </view>
        <view class="drawer-item" bindtap="onGoSkills">
          <text class="drawer-icon">📚</text>
          <text class="drawer-label">技能</text>
        </view>
        <view class="drawer-item" bindtap="onGoPlugins">
          <text class="drawer-icon">🧩</text>
          <text class="drawer-label">插件</text>
        </view>
      </view>
      <view class="drawer-divider"></view>
      <view class="drawer-group">
        <view class="drawer-item" bindtap="onGoAgents">
          <text class="drawer-icon">👤</text>
          <text class="drawer-label">Agent 切换</text>
        </view>
        <view class="drawer-item" bindtap="onGoSettings">
          <text class="drawer-icon">⚙️</text>
          <text class="drawer-label">设置</text>
        </view>
      </view>
    </view>
  </view>
</view>
```

---

### Task 3: Chat Page — WXSS Styles

**Files:**
- Rewrite: `apps/IrsClawMiniProgram/pages/index/index.wxss`

- [ ] **Step 1: Write light-themed chat styles**

```wxss
page {
  background: var(--page-bg);
}

.chat-page {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--page-bg);
}

/* ===== NavBar ===== */
.navbar {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  padding-top: calc(12px + env(safe-area-inset-top));
  background: var(--card-bg);
  border-bottom: 0.5px solid var(--divider);
}

.nav-btn {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #F2F2F2;
}
.nav-btn:active {
  background: #E0E0E0;
}

.nav-btn-brand {
  background: var(--brand);
}
.nav-btn-brand:active {
  background: var(--brand-press);
}

.nav-icon {
  font-size: 18px;
  color: var(--text-primary);
}

.nav-icon-plus {
  font-size: 22px;
  color: #fff;
  font-weight: 300;
  line-height: 1;
}

.nav-title-wrap {
  flex: 1;
  text-align: center;
}

.nav-title {
  font-size: 17px;
  font-weight: 600;
  color: var(--text-primary);
}

/* ===== Messages ===== */
.msg-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 120px;
  gap: 12px;
}

.empty-avatar {
  width: 60px;
  height: 60px;
  border-radius: 18px;
  background: var(--brand);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 600;
  color: #fff;
  margin-bottom: 4px;
}

/* ===== Message Row ===== */
.msg-row {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  animation: msgFadeIn 0.25s ease;
}

@keyframes msgFadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.msg-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  flex-shrink: 0;
}

.ai-avatar {
  background: var(--text-secondary);
}

.user-avatar {
  background: var(--brand);
}

/* ===== User Bubble (right-aligned) ===== */
.msg-user {
  flex-direction: row-reverse;
}

.user-bubble {
  background: var(--user-bubble);
  padding: 12px 16px;
  border-radius: 10px;
  border-bottom-right-radius: 2px;
  max-width: 80%;
}

.msg-user .msg-text {
  color: #fff;
}

/* ===== AI Bubble (left-aligned) ===== */
.msg-content {
  flex: 1;
  max-width: 80%;
}

.ai-bubble {
  background: var(--ai-bubble);
  padding: 12px 16px;
  border-radius: 10px;
  border-bottom-left-radius: 2px;
  border: 0.5px solid var(--ai-bubble-border);
}

.msg-text {
  font-size: 16px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.msg-ai .msg-text {
  color: var(--text-primary);
}

/* ===== Foldable Blocks (Reasoning / Tools) ===== */
.fold-block {
  margin-bottom: 8px;
}

.fold-header {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  border-radius: 8px;
  font-size: 13px;
}

.reasoning-block .fold-header {
  background: rgba(255, 149, 0, 0.1);
  color: #B86800;
}

.tool-block .fold-header {
  background: rgba(88, 86, 214, 0.08);
  color: #5856D6;
}

.fold-icon {
  font-size: 10px;
}

.fold-label {
  font-size: 13px;
  font-weight: 500;
}

.fold-body {
  margin-top: 6px;
  padding: 10px 12px;
  background: #F8F8F8;
  border-radius: 8px;
}

.fold-text {
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
}

/* ===== Tool Calls ===== */
.tool-item {
  margin-bottom: 6px;
  padding: 8px 10px;
  background: var(--card-bg);
  border-radius: 6px;
}
.tool-item:last-child {
  margin-bottom: 0;
}

.tool-call-name {
  font-size: 12px;
  font-weight: 600;
  color: #5856D6;
  display: block;
  margin-bottom: 2px;
}

.tool-call-args {
  font-size: 11px;
  color: var(--text-tertiary);
  font-family: Menlo, Monaco, monospace;
  display: block;
  white-space: pre-wrap;
  word-break: break-all;
}

/* ===== Input Bar ===== */
.input-bar {
  padding: 8px 16px calc(8px + env(safe-area-inset-bottom));
  background: var(--card-bg);
  border-top: 0.5px solid var(--divider);
}

.input-wrap {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  background: #F2F2F2;
  border-radius: 8px;
  padding: 8px 8px 8px 14px;
}

.input-box {
  flex: 1;
  min-height: 22px;
  max-height: 88px;
  font-size: 17px;
  line-height: 1.4;
  color: var(--text-primary);
  background: transparent;
  border: none;
  outline: none;
  caret-color: var(--brand);
  padding: 0;
}

.send-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #E0E0E0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  color: #fff;
  flex-shrink: 0;
}

.send-btn.send-active {
  background: var(--brand);
}

/* ===== Sidebar Drawer ===== */
.drawer-mask {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--mask);
  z-index: 100;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.3s ease;
}
.drawer-mask.show {
  opacity: 1;
  pointer-events: auto;
}

.drawer {
  position: fixed;
  top: 0;
  left: 0;
  width: 280px;
  height: 100%;
  background: var(--card-bg);
  z-index: 200;
  transform: translateX(-100%);
  transition: transform 0.3s ease;
}
.drawer.open {
  transform: translateX(0);
}

.drawer-head {
  padding: calc(24px + env(safe-area-inset-top)) 20px 16px;
  border-bottom: 0.5px solid var(--divider);
}

.drawer-brand {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
}

.drawer-body {
  padding: 8px 0;
}

.drawer-group {
  padding: 4px 12px;
}

.drawer-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 12px;
  border-radius: 10px;
  margin-bottom: 2px;
}
.drawer-item:active {
  background: #F2F2F2;
}

.drawer-icon {
  width: 28px;
  text-align: center;
  font-size: 18px;
}

.drawer-label {
  font-size: 16px;
  color: var(--text-primary);
}

.drawer-divider {
  height: 0.5px;
  background: var(--divider);
  margin: 8px 16px;
}

.scroll-bottom {
  height: 1px;
}
```

- [ ] **Step 2: Update `pages/index/index.json`**

```json
{
  "navigationStyle": "custom",
  "navigationBarTitleText": "IrsClaw"
}
```

---

### Task 4: Chat Page — JS Logic

**Files:**
- Rewrite: `apps/IrsClawMiniProgram/pages/index/index.js`

- [ ] **Step 1: Write the complete controller**

```javascript
const app = getApp()

Page({
  data: {
    isConnected: false,
    currentAgentId: 'default',
    messages: [],
    inputText: '',
    menuOpen: false
  },

  onLoad() {
    this.checkAndLoad()
  },

  onShow() {
    if (app.globalData.currentSession) {
      this.loadSessionMessages(app.globalData.currentSession.id)
    }
  },

  async checkAndLoad() {
    const connected = await app.ping()
    this.setData({ isConnected: connected })
    app.globalData.isConnected = connected

    if (connected) {
      await app.getConfig()
      const sessions = await app.getSessions()
      if (sessions.length > 0) {
        const latest = sessions[0]
        app.globalData.currentSession = latest
        await this.loadSessionMessages(latest.id)
      }
    }
  },

  async loadSessionMessages(sessionId) {
    const data = await app.getSession(sessionId)
    if (data && data.messages) {
      const msgs = data.messages.map(m => {
        let toolCalls = m.tool_calls
        if (toolCalls && typeof toolCalls === 'string') {
          try { toolCalls = JSON.parse(toolCalls) } catch (e) { toolCalls = [] }
        }
        if (toolCalls && toolCalls.length > 0) {
          toolCalls = toolCalls.map(tc => {
            if (tc.function && typeof tc.function.arguments === 'string') {
              try { tc.function.arguments = JSON.parse(tc.function.arguments) } catch (e) {}
            }
            return tc
          })
        }
        return {
          ...m,
          tool_calls: toolCalls,
          _showThinking: false,
          _showTools: false,
          _showReasoning: false
        }
      })
      this.setData({
        messages: msgs,
        currentAgentId: data.meta?.agent_id || 'default'
      })
    }
  },

  onInput(e) {
    this.setData({ inputText: e.detail.value })
  },

  async onSend() {
    const content = this.data.inputText.trim()
    if (!content || !this.data.isConnected) return

    this.setData({ inputText: '' })

    const tempId = `temp-${Date.now()}`
    const msgs = [...this.data.messages, {
      id: tempId,
      role: 'user',
      content,
      _showThinking: false,
      _showTools: false,
      _showReasoning: false
    }]
    this.setData({ messages: msgs })

    const res = await app.sendMessage(content, this.data.currentAgentId)

    if (res && res.session_id) {
      app.globalData.currentSession = { id: res.session_id }
      setTimeout(() => {
        this.loadSessionMessages(res.session_id)
      }, 500)
    }
  },

  onToggleReasoning(e) {
    const id = e.currentTarget.dataset.id
    const msgs = this.data.messages.map(m => {
      if (m.id === id) return { ...m, _showReasoning: !m._showReasoning }
      return m
    })
    this.setData({ messages: msgs })
  },

  onToggleTools(e) {
    const id = e.currentTarget.dataset.id
    const msgs = this.data.messages.map(m => {
      if (m.id === id) return { ...m, _showTools: !m._showTools }
      return m
    })
    this.setData({ messages: msgs })
  },

  onToggleThinking(e) {
    const id = e.currentTarget.dataset.id
    const msgs = this.data.messages.map(m => {
      if (m.id === id) return { ...m, _showThinking: !m._showThinking }
      return m
    })
    this.setData({ messages: msgs })
  },

  async onNewChat() {
    const session = await app.createSession(this.data.currentAgentId)
    if (session) {
      app.globalData.currentSession = session
      this.setData({ messages: [], menuOpen: false })
    }
  },

  onToggleMenu() {
    this.setData({ menuOpen: !this.data.menuOpen })
  },

  onCloseMenu() {
    this.setData({ menuOpen: false })
  },

  onGoAgents() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/agents/index' })
  },

  onGoSessions() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/sessions/index' })
  },

  onGoTools() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/tools/index' })
  },

  onGoSkills() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/skills/index' })
  },

  onGoPlugins() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/plugins/index' })
  },

  onGoSettings() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/settings/index' })
  }
})
```

---

### Task 5: Sessions Page

**Files:**
- Rewrite: `apps/IrsClawMiniProgram/pages/sessions/index.wxml`
- Rewrite: `apps/IrsClawMiniProgram/pages/sessions/index.wxss`
- Keep: `apps/IrsClawMiniProgram/pages/sessions/index.js` (logic is fine)

- [ ] **Step 1: Rewrite `index.wxml`**

```wxml
<view class="list-page">
  <view class="section-body">
    <block wx:if="{{sessions.length === 0}}">
      <view class="empty-state">
        <view class="empty-icon">💬</view>
        <text class="empty-title">暂无会话</text>
        <text class="empty-sub">开始一段新对话吧</text>
      </view>
    </block>
    <block wx:else>
      <view wx:for="{{sessions}}" wx:key="id" class="list-item" bindtap="onSelect" data-id="{{item.id}}">
        <view class="list-icon purple">💬</view>
        <view class="list-body">
          <text class="list-title">{{item.title || '未命名会话'}}</text>
          <text class="list-desc">{{item.message_count || 0}} 条消息</text>
        </view>
        <view class="list-item-del" catchtap="onDelete" data-id="{{item.id}}">🗑️</view>
      </view>
    </block>
  </view>
</view>
```

- [ ] **Step 2: Rewrite `index.wxss`**

```wxss
/* Sessions Page — uses shared .list-* from app.wxss */
page {
  background: var(--page-bg);
}

.list-item-del {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  flex-shrink: 0;
}
.list-item-del:active {
  background: #F2F2F2;
}
```

---

### Task 6: Tools / Skills / Plugins Pages

**Files:**
- Rewrite: `apps/IrsClawMiniProgram/pages/tools/index.wxml`
- Rewrite: `apps/IrsClawMiniProgram/pages/tools/index.wxss`
- Rewrite: `apps/IrsClawMiniProgram/pages/skills/index.wxml`
- Rewrite: `apps/IrsClawMiniProgram/pages/skills/index.wxss`
- Rewrite: `apps/IrsClawMiniProgram/pages/plugins/index.wxml`
- Rewrite: `apps/IrsClawMiniProgram/pages/plugins/index.wxss`

These 3 pages are structurally identical, only the icon class and emoji differ.

- [ ] **Step 1: Rewrite all 3 WXML files**

Tools `index.wxml`:
```wxml
<view class="list-page">
  <view class="section-body">
    <block wx:if="{{tools.length === 0}}">
      <view class="empty-state">
        <view class="empty-icon">🔧</view>
        <text class="empty-title">暂无工具</text>
        <text class="empty-sub">连接服务器后工具将自动加载</text>
      </view>
    </block>
    <view wx:for="{{tools}}" wx:key="id" class="list-item" wx:else>
      <view class="list-icon orange">🔧</view>
      <view class="list-body">
        <text class="list-title">{{item.name || item.id}}</text>
        <text class="list-desc">{{item.description || '暂无描述'}}</text>
      </view>
      <view class="list-arrow">›</view>
    </view>
  </view>
</view>
```

Skills `index.wxml` (change icon class to `green`, emoji to `📚`):
```wxml
<view class="list-page">
  <view class="section-body">
    <block wx:if="{{skills.length === 0}}">
      <view class="empty-state">
        <view class="empty-icon">📚</view>
        <text class="empty-title">暂无技能</text>
        <text class="empty-sub">连接服务器后技能将自动加载</text>
      </view>
    </block>
    <view wx:for="{{skills}}" wx:key="id" class="list-item" wx:else>
      <view class="list-icon green">📚</view>
      <view class="list-body">
        <text class="list-title">{{item.name || item.id}}</text>
        <text class="list-desc">{{item.description || '暂无描述'}}</text>
      </view>
      <view class="list-arrow">›</view>
    </view>
  </view>
</view>
```

Plugins `index.wxml` (change icon class to `blue`, emoji to `🧩`):
```wxml
<view class="list-page">
  <view class="section-body">
    <block wx:if="{{plugins.length === 0}}">
      <view class="empty-state">
        <view class="empty-icon">🧩</view>
        <text class="empty-title">暂无插件</text>
        <text class="empty-sub">连接服务器后插件将自动加载</text>
      </view>
    </block>
    <view wx:for="{{plugins}}" wx:key="id" class="list-item" wx:else>
      <view class="list-icon blue">🧩</view>
      <view class="list-body">
        <text class="list-title">{{item.name || item.id}}</text>
        <text class="list-desc">{{item.description || '暂无描述'}}</text>
      </view>
      <view class="list-arrow">›</view>
    </view>
  </view>
</view>
```

- [ ] **Step 2: Rewrite all 3 WXSS files** (all the same, since shared styles are in `app.wxss`)

```wxss
page {
  background: var(--page-bg);
}
```

(Replace tools/index.wxss, skills/index.wxss, plugins/index.wxss with the same 3 lines)

---

### Task 7: Agents Page

**Files:**
- Rewrite: `apps/IrsClawMiniProgram/pages/agents/index.wxml`
- Rewrite: `apps/IrsClawMiniProgram/pages/agents/index.wxss`
- Keep: `apps/IrsClawMiniProgram/pages/agents/index.js`

- [ ] **Step 1: Rewrite `index.wxml`**

```wxml
<view class="list-page">
  <view class="section-header">AGENT</view>
  <view class="section-body">
    <block wx:if="{{agents.length === 0}}">
      <view class="empty-state">
        <view class="empty-icon">👤</view>
        <text class="empty-title">暂无 Agent</text>
        <text class="empty-sub">连接服务器后可查看可用 Agent</text>
      </view>
    </block>
    <view wx:for="{{agents}}" wx:key="id" class="list-item" bindtap="onSelectAgent" data-id="{{item.id}}" wx:else>
      <view class="list-icon blue">👤</view>
      <view class="list-body">
        <text class="list-title">{{item.name || item.id}}</text>
        <text class="list-desc">{{item.model || ''}}</text>
      </view>
      <view wx:if="{{item.id === currentAgentId}}" class="agent-check">
        <text>✓</text>
      </view>
    </view>
  </view>
</view>
```

- [ ] **Step 2: Rewrite `index.wxss`**

```wxss
page {
  background: var(--page-bg);
}

.agent-check {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--brand);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 13px;
  font-weight: bold;
  flex-shrink: 0;
}
```

---

### Task 8: Settings Page

**Files:**
- Rewrite: `apps/IrsClawMiniProgram/pages/settings/index.wxml`
- Rewrite: `apps/IrsClawMiniProgram/pages/settings/index.wxss`
- Rewrite: `apps/IrsClawMiniProgram/pages/settings/index.js`

- [ ] **Step 1: Rewrite `index.wxml`**

```wxml
<view class="list-page">
  <!-- Connection Status -->
  <view class="section-group">
    <view class="section-header">连接</view>
    <view class="card-rounded">
      <view class="status-row">
        <view class="status-dot {{isConnected ? 'online' : ''}}"></view>
        <text class="status-text">{{isConnected ? '已连接' : '未连接'}}</text>
        <text class="info-val">{{serverUrl}}</text>
      </view>
    </view>
  </view>

  <!-- Server Config -->
  <view class="section-group">
    <view class="section-header">服务器配置</view>
    <view class="card-rounded">
      <view class="input-row">
        <text class="input-label">服务器地址</text>
        <input class="input-field" type="text" value="{{serverUrl}}" bindinput="onUrl" placeholder="http://localhost:3000" />
      </view>
      <view class="input-row">
        <text class="input-label">认证令牌</text>
        <input class="input-field" type="text" value="{{authToken}}" bindinput="onToken" placeholder="选填" />
      </view>
    </view>
    <view style="padding: 12px 16px;">
      <view class="btn-primary" bindtap="onSave">保存并重连</view>
    </view>
  </view>

  <!-- AI Provider Info -->
  <view class="section-group" wx:if="{{config}}">
    <view class="section-header">AI 提供方</view>
    <view class="card-rounded">
      <view class="info-row">
        <text>Provider</text>
        <text class="info-val">{{config.provider || '-'}}</text>
      </view>
      <view class="info-row">
        <text>Model</text>
        <text class="info-val">{{config.model || '-'}}</text>
      </view>
    </view>
  </view>
</view>
```

- [ ] **Step 2: Rewrite `index.wxss`**

```wxss
page {
  background: var(--page-bg);
}
```

- [ ] **Step 3: Rewrite `index.js`** (remove dark/light theme toggle; simplify)

```javascript
const app = getApp()

Page({
  data: {
    isConnected: false,
    serverUrl: '',
    authToken: '',
    config: null
  },

  onLoad() {
    this.setData({
      serverUrl: app.globalData.serverUrl,
      authToken: app.globalData.authToken,
      isConnected: app.globalData.isConnected
    })
    this.loadConfig()
  },

  onShow() {
    this.setData({ isConnected: app.globalData.isConnected })
  },

  async loadConfig() {
    const config = await app.getConfig()
    if (config) {
      this.setData({ config })
    }
  },

  onUrl(e) {
    this.setData({ serverUrl: e.detail.value })
  },

  onToken(e) {
    this.setData({ authToken: e.detail.value })
  },

  onSave() {
    const url = this.data.serverUrl.trim()
    if (!url) {
      wx.showToast({ title: '请输入服务器地址', icon: 'none' })
      return
    }

    app.globalData.serverUrl = url
    app.globalData.authToken = this.data.authToken.trim()
    wx.setStorageSync('serverUrl', url)
    wx.setStorageSync('authToken', app.globalData.authToken)

    app.ping().then(ok => {
      this.setData({ isConnected: ok })
      if (ok) {
        wx.showToast({ title: '连接成功', icon: 'success' })
        this.loadConfig()
      } else {
        wx.showToast({ title: '连接失败', icon: 'none' })
      }
    })
  }
})
```

---

### Task 9: Remove Theme-Related JS from app.js (if any)

**Files:**
- Modify: `apps/IrsClawMiniProgram/app.js` — no changes needed (theme was only in settings page)

---

## Self-Review Checklist

1. **Spec coverage:** 
   - ✅ Light theme throughout (Task 1 global tokens + Task 8 removes dark mode)
   - ✅ Chat as main page with custom navbar (Task 2-4)
   - ✅ Sidebar drawer with all navigation items (Task 2 WXML)
   - ✅ Sessions/tools/skills/plugins/agents/settings pages (Tasks 5-8)
   - ✅ #07C160 brand green color (Task 1 CSS vars)
   - ✅ Standard WeChat list patterns (Task 1 shared components)

2. **Placeholder scan:** All code blocks have complete implementations. No TBD/TODO/fill-in-later patterns.

3. **Type consistency:** All page data fields, event handlers, and API response fields match across JS/WXML/WXSS. `agentId` → `currentAgentId`, `session.id` consistent everywhere.
