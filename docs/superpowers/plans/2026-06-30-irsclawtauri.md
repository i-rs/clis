# IrsClawTauri Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri 2 + React 19 desktop/mobile client (Windows/Linux/Android) for i-rs-claw, acting as a pure HTTP client to remote `claw serve`, with native shell enhancements (tray, notifications, file dialog, clipboard, deep-link, autostart).

**Architecture:** Single React codebase with CSS breakpoint adaptive layout (>=768px desktop sidebar / <768px mobile tab+drawer). Zustand for state, wouter for routing, CSS variables for theming. Rust side is a thin shell exposing Tauri commands for native OS features only — no business logic. All data flows through `fetch` to `claw serve`'s HTTP API with Bearer auth + SSE streaming.

**Tech Stack:** Tauri 2, React 19, TypeScript, Zustand 5, wouter 3, lucide-react, react-markdown + remark-gfm, Vite+ (`vp` CLI), Vitest 4, Rust (tauri plugins: opener/notification/dialog/clipboard-manager/deep-link/autostart), reqwest for health checks.

**Spec:** `docs/superpowers/specs/2026-06-30-irsclawtauri-design.md`

**Working directory for all commands:** `apps/IrsClawTauri/` unless noted. Repo root: `/Users/mankong/volumes/code/i-rs/clis`.

**Conventions:** No comments in code unless asked. Use `expect("msg")` in Rust, never `unwrap()`. CSS variables + `.module.css` (mirror dashboard-ui). Keep the existing `vp` toolchain (Vite+). Run `pnpm --filter irsclawtauri exec tsc --noEmit` after frontend changes, `cargo check -p irsclawtauri` after Rust changes. The dashboard-ui source at `crates/claw/dashboard-ui/src/` is the pattern reference for ported components.

**Pattern reference (read these once before starting):**
- `crates/claw/dashboard-ui/src/api.ts` — API client + SSE parser patterns
- `crates/claw/dashboard-ui/src/hooks/useChatStream.ts` — SSE state machine
- `crates/claw/dashboard-ui/src/hooks/useSessionLoader.ts` — session load/create
- `crates/claw/dashboard-ui/src/components/*.tsx` — chat components to port
- `crates/claw/dashboard-ui/src/styles.css` — CSS design tokens to copy
- `apps/IrsClawApp/IrsClawApp/Views/ContentView.swift` — layout reference (SplitView/Drawer)
- `apps/IrsClawApp/IrsClawApp/Views/ChatView.swift` — FloatingInput reference

---

## File Structure

**Frontend (`apps/IrsClawTauri/src/`)** — each file one responsibility:
- `main.tsx` — entry, mounts App, imports global styles
- `App.tsx` — root: theme bootstrap + backend connect gate + shell selection + router
- `styles/theme.css` — CSS design tokens + reset + animations (copied from dashboard-ui)
- `styles/responsive.css` — breakpoint helpers + mobile overrides
- `api/types.ts` — shared TypeScript types
- `api/client.ts` — `authFetch` + `baseUrl` from backendStore + response helpers
- `api/sessions.ts` / `agents.ts` / `stats.ts` / `chat.ts` — resource modules (chat.ts = SSE)
- `store/backend.ts` — Zustand: baseUrl, token, connectionState (persisted)
- `store/ui.ts` — Zustand: theme, selectedTab, mobileDrawerOpen
- `hooks/useMediaQuery.ts` / `useTheme.ts` / `useBackendHealth.ts`
- `hooks/useChatStream.ts` / `useSessionLoader.ts` — adapted from dashboard-ui
- `components/common/` — AgentChip, EmptyState, PulsingDot, Toast
- `components/chat/` — MarkdownRenderer, ToolCallCard, MessageBubble, StreamingBubble, FloatingInput, MessageList
- `components/shell/` — Sidebar, DesktopShell, MobileShell, BottomTabBar, Drawer, ConnectScreen
- `pages/` — Chat, Sessions, Agents, Usage, Settings, Placeholder
- `*.module.css` — colocated component styles

**Rust (`apps/IrsClawTauri/src-tauri/src/`)**:
- `main.rs` — binary entry (exists, unchanged)
- `lib.rs` — `run()`: Tauri builder + plugin registration + invoke handlers + tray + deep-link
- `commands.rs` — all `#[tauri::command]` functions
- `tray.rs` — system tray icon with menu

**Config**:
- `src-tauri/Cargo.toml` — add 6 plugins + reqwest + tokio
- `src-tauri/tauri.conf.json` — window title/size, deep-link protocol
- `src-tauri/capabilities/default.json` — plugin permissions
- `package.json` — add frontend deps + test scripts + vitest.config.ts

---

## Task 1: Install frontend dependencies

**Files:**
- Modify: `apps/IrsClawTauri/package.json`

- [ ] **Step 1: Add runtime + dev dependencies**

Run from repo root:
```bash
pnpm --filter irsclawtauri add wouter lucide-react react-markdown remark-gfm @tauri-apps/plugin-notification @tauri-apps/plugin-dialog @tauri-apps/plugin-clipboard-manager @tauri-apps/plugin-deep-link @tauri-apps/plugin-autostart
pnpm --filter irsclawtauri add -D vitest@catalog: @testing-library/react @testing-library/jest-dom jsdom @types/node
```

- [ ] **Step 2: Add test scripts to package.json**

Edit `apps/IrsClawTauri/package.json` `scripts` to add:
```json
"test": "vitest run",
"test:watch": "vitest"
```

- [ ] **Step 3: Verify install**

Run: `pnpm --filter irsclawtauri list wouter lucide-react react-markdown`
Expected: all three listed with versions.

- [ ] **Step 4: Commit**
```bash
git add apps/IrsClawTauri/package.json apps/IrsClawTauri/pnpm-lock.yaml
git commit -m "feat(IrsClawTauri): add frontend deps (wouter, lucide, markdown, tauri plugins, vitest)"
```

---

## Task 2: Rust plugin dependencies

**Files:**
- Modify: `apps/IrsClawTauri/src-tauri/Cargo.toml`

- [ ] **Step 1: Replace dependencies section**

Edit `apps/IrsClawTauri/src-tauri/Cargo.toml`, replace the `[dependencies]` block:
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-opener = "2"
tauri-plugin-notification = "2"
tauri-plugin-dialog = "2"
tauri-plugin-clipboard-manager = "2"
tauri-plugin-deep-link = "2"
tauri-plugin-autostart = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p irsclawtauri`
Expected: downloads crates, 0 errors. Plugins not yet registered in lib.rs — that's fine.

- [ ] **Step 3: Commit**
```bash
git add apps/IrsClawTauri/src-tauri/Cargo.toml Cargo.lock
git commit -m "feat(IrsClawTauri): add tauri plugins + reqwest + tokio to Cargo.toml"
```

---

## Task 3: Global theme CSS

**Files:**
- Create: `apps/IrsClawTauri/src/styles/theme.css`
- Create: `apps/IrsClawTauri/src/styles/responsive.css`

- [ ] **Step 1: Create theme.css**

Copy the design tokens from `crates/claw/dashboard-ui/src/styles.css` (lines 1-188: the `@import` font, `:root` dark tokens, `[data-theme="light"]` block, reset, scrollbar, animations). Then append shared UI primitives. Create `apps/IrsClawTauri/src/styles/theme.css`:

```css
@import url('https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600;700&family=Geist+Mono:wght@400;500&display=swap');

:root {
  --bg-primary: #08080b; --bg-secondary: #0d0d12; --bg-tertiary: #121218;
  --bg-elevated: #16161e; --bg-hover: rgba(255,255,255,0.04); --bg-active: rgba(255,255,255,0.06);
  --text-primary: #f4f4f5; --text-secondary: #a1a1aa; --text-muted: #52525b; --text-tertiary: #3f3f46;
  --border: rgba(255,255,255,0.06); --border-subtle: rgba(255,255,255,0.04); --border-light: rgba(255,255,255,0.1);
  --accent: #22d3ee; --accent-hover: #06b6d4; --accent-muted: rgba(34,211,238,0.12); --accent-glow: rgba(34,211,238,0.2);
  --success: #4ade80; --success-bg: rgba(74,222,128,0.1); --warning: #fbbf24; --warning-bg: rgba(251,191,36,0.1);
  --error: #f87171; --error-bg: rgba(248,113,113,0.1); --info: #60a5fa; --info-bg: rgba(96,165,250,0.1);
  --ai-primary: #a78bfa; --ai-secondary: #c4b5fd; --ai-muted: rgba(167,139,250,0.12); --ai-glow: rgba(167,139,250,0.2);
  --sidebar-width: 240px; --radius-sm: 6px; --radius: 10px; --radius-lg: 14px; --radius-xl: 18px;
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.3); --shadow-md: 0 4px 12px rgba(0,0,0,0.4); --shadow-lg: 0 8px 24px rgba(0,0,0,0.5);
  --ease-out: cubic-bezier(0.16,1,0.3,1); --ease-spring: cubic-bezier(0.34,1.56,0.64,1);
  --duration-fast: 0.15s; --duration-normal: 0.25s; --duration-slow: 0.4s;
}

[data-theme="light"] {
  --bg-primary: #fafafa; --bg-secondary: #ffffff; --bg-tertiary: #f4f4f5; --bg-elevated: #ffffff;
  --bg-hover: rgba(0,0,0,0.03); --bg-active: rgba(0,0,0,0.05);
  --text-primary: #18181b; --text-secondary: #52525b; --text-muted: #a1a1aa; --text-tertiary: #d4d4d8;
  --border: rgba(0,0,0,0.06); --border-subtle: rgba(0,0,0,0.04); --border-light: rgba(0,0,0,0.1);
  --accent: #0891b2; --accent-hover: #0e7490; --accent-muted: rgba(8,145,178,0.08); --accent-glow: rgba(8,145,178,0.15);
  --ai-primary: #7c3aed; --ai-secondary: #8b5cf6; --ai-muted: rgba(124,58,237,0.08); --ai-glow: rgba(124,58,237,0.15);
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.04); --shadow-md: 0 4px 12px rgba(0,0,0,0.06); --shadow-lg: 0 8px 24px rgba(0,0,0,0.08);
}

*, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }
html, body { font-family: 'Geist', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif; font-size: 14px; line-height: 1.6; background: var(--bg-primary); color: var(--text-primary); -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; overflow: hidden; height: 100vh; }
#root { height: 100vh; display: flex; }
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border-light); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: var(--text-tertiary); }

@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
@keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
@keyframes slideInLeft { from { opacity: 0; transform: translateX(-8px); } to { opacity: 1; transform: translateX(0); } }
@keyframes scaleIn { from { opacity: 0; transform: scale(0.95); } to { opacity: 1; transform: scale(1); } }
@keyframes spin { to { transform: rotate(360deg); } }
@keyframes float { 0%,100% { transform: translateY(0); } 50% { transform: translateY(-4px); } }
@keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.5; } }
@keyframes typingBounce { 0%,60%,100% { transform: translateY(0); opacity: 0.4; } 30% { transform: translateY(-6px); opacity: 1; } }

.btn { display: inline-flex; align-items: center; justify-content: center; gap: 8px; padding: 10px 16px; border: none; border-radius: var(--radius); cursor: pointer; font-size: 13px; font-weight: 600; font-family: inherit; transition: all var(--duration-fast) var(--ease-out); white-space: nowrap; }
.btn-primary { background: linear-gradient(135deg, var(--accent), var(--accent-hover)); color: #000; box-shadow: 0 2px 8px var(--accent-muted); }
.btn-primary:hover { transform: translateY(-1px); box-shadow: 0 4px 16px var(--accent-glow); }
.btn-secondary { background: var(--bg-tertiary); color: var(--text-primary); border: 1px solid var(--border); }
.btn-secondary:hover { background: var(--bg-hover); border-color: var(--border-light); }
.btn-ghost { background: transparent; color: var(--text-secondary); }
.btn-ghost:hover { background: var(--bg-hover); color: var(--text-primary); }
.btn-sm { padding: 6px 12px; font-size: 12px; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; transform: none !important; }

.badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 20px; font-size: 11px; font-weight: 600; }
.badge-info { background: var(--info-bg); color: var(--info); }
.typing-dots { display: flex; gap: 4px; align-items: center; padding: 8px 0; }
.typing-dots span { width: 6px; height: 6px; border-radius: 50%; background: var(--ai-primary); animation: typingBounce 1.4s ease-in-out infinite; }
.typing-dots span:nth-child(2) { animation-delay: 0.15s; }
.typing-dots span:nth-child(3) { animation-delay: 0.3s; }
.loading { display: flex; align-items: center; justify-content: center; gap: 10px; padding: 40px; color: var(--text-muted); }
.loading-spinner { width: 18px; height: 18px; border: 2px solid var(--border); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.6s linear infinite; }
.sessions-list { display: flex; flex-direction: column; gap: 8px; }
.session-card { display: flex; align-items: center; gap: 16px; padding: 16px; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius-lg); cursor: pointer; transition: all var(--duration-fast) var(--ease-out); }
.session-card:hover { border-color: var(--border-light); background: var(--bg-tertiary); transform: translateY(-1px); }
.session-icon { width: 40px; height: 40px; border-radius: 10px; background: var(--ai-muted); display: flex; align-items: center; justify-content: center; color: var(--ai-primary); flex-shrink: 0; }
.session-info { flex: 1; min-width: 0; }
.session-title { font-size: 14px; font-weight: 600; margin-bottom: 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.session-meta { font-size: 12px; color: var(--text-muted); display: flex; align-items: center; gap: 8px; }
.session-actions { display: flex; gap: 8px; flex-shrink: 0; }
.session-btn { display: inline-flex; align-items: center; gap: 6px; padding: 8px 14px; border-radius: var(--radius); font-size: 12px; font-weight: 600; cursor: pointer; border: 1px solid var(--border); background: transparent; color: var(--text-muted); font-family: inherit; transition: all var(--duration-fast); }
.session-btn-delete:hover { background: var(--error-bg); color: var(--error); border-color: var(--error); }
.card { background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: 16px; }
.card-title { font-size: 14px; font-weight: 600; margin-bottom: 8px; }
.period-selector { display: flex; gap: 8px; margin-bottom: 24px; padding: 4px; background: var(--bg-secondary); border-radius: var(--radius); border: 1px solid var(--border); }
.period-btn { flex: 1; padding: 10px 16px; border: none; background: transparent; color: var(--text-secondary); font-size: 13px; font-weight: 500; border-radius: var(--radius-sm); cursor: pointer; transition: all var(--duration-fast); font-family: inherit; }
.period-btn.active { background: var(--accent-muted); color: var(--accent); }
.usage-cards { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 24px; }
.usage-card { background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: 20px; display: flex; align-items: center; gap: 16px; }
.usage-card-icon { width: 48px; height: 48px; border-radius: var(--radius); background: var(--bg-tertiary); display: flex; align-items: center; justify-content: center; color: var(--accent); }
.usage-card-value { font-size: 24px; font-weight: 700; font-family: ui-monospace, monospace; }
.usage-card-label { font-size: 13px; color: var(--text-secondary); margin-top: 4px; }
.today-stats { display: flex; gap: 32px; }
.today-stat { display: flex; flex-direction: column; align-items: center; gap: 4px; }
.today-stat-value { font-size: 28px; font-weight: 700; font-family: ui-monospace, monospace; }
.today-stat-label { font-size: 12px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.config-input { width: 100%; padding: 10px 14px; background: var(--bg-tertiary); border: 1px solid var(--border); border-radius: var(--radius); color: var(--text-primary); font-size: 13px; font-family: inherit; }
.config-input:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-muted); }

@media (max-width: 767px) { .usage-cards { grid-template-columns: 1fr; } }
@media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; } }
```

- [ ] **Step 2: Create responsive.css**

Create `apps/IrsClawTauri/src/styles/responsive.css`:
```css
.app-shell { display: flex; width: 100%; height: 100vh; background: var(--bg-primary); overflow: hidden; }
.app-detail { flex: 1; display: flex; flex-direction: column; overflow: hidden; background: var(--bg-primary); min-width: 0; }
```

- [ ] **Step 3: Commit**
```bash
git add apps/IrsClawTauri/src/styles/
git commit -m "feat(IrsClawTauri): add theme tokens + responsive base CSS"
```

---

## Task 4: API types

**Files:**
- Create: `apps/IrsClawTauri/src/api/types.ts`

- [ ] **Step 1: Write shared types**

Create `apps/IrsClawTauri/src/api/types.ts`. Copy the type/interface declarations from `crates/claw/dashboard-ui/src/api.ts` lines 39-351 (all `interface`/`type` blocks: `ApiResponse`, `SessionMeta`, `AgentInfo`, `ToolSchema`, `PluginInfo`, `SkillInfo`, `StatsResponse`, `CurrentSession`, `ToolCallEvent`, `ImageGeneratedEvent`, `EvaluationEvent`, `QualityScore`, `FeedbackEvent`, `TokenUsage`, `ToolCallMsg`, `ChatMessage`, `SseEventHandler`). Do NOT include the function implementations from api.ts — only the types.

- [ ] **Step 2: Commit**
```bash
git add apps/IrsClawTauri/src/api/types.ts
git commit -m "feat(IrsClawTauri): add shared API types"
```

---

## Task 5: backend store + api client + vitest (TDD)

**Files:**
- Create: `apps/IrsClawTauri/vitest.config.ts`
- Create: `apps/IrsClawTauri/src/test-setup.ts`
- Create: `apps/IrsClawTauri/src/store/backend.test.ts`
- Create: `apps/IrsClawTauri/src/store/backend.ts`
- Create: `apps/IrsClawTauri/src/api/client.ts`

- [ ] **Step 1: Create vitest config + setup**

Create `apps/IrsClawTauri/vitest.config.ts`:
```ts
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: { environment: 'jsdom', globals: true, setupFiles: ['./src/test-setup.ts'] },
})
```
Create `apps/IrsClawTauri/src/test-setup.ts`:
```ts
import '@testing-library/jest-dom/vitest'
```

- [ ] **Step 2: Write failing test for backend store**

Create `apps/IrsClawTauri/src/store/backend.test.ts`:
```ts
import { describe, it, expect, beforeEach } from 'vitest'
import { useBackendStore } from './backend'

describe('backendStore', () => {
  beforeEach(() => {
    localStorage.clear()
    useBackendStore.setState({ baseUrl: 'http://localhost:3000', token: '', connectionState: 'disconnected', errorMessage: null })
  })

  it('persists baseUrl and token to localStorage', () => {
    useBackendStore.getState().setBaseUrl('http://192.168.1.5:3000')
    useBackendStore.getState().setToken('abc123')
    expect(localStorage.getItem('claw-backend-url')).toBe('http://192.168.1.5:3000')
    expect(localStorage.getItem('claw-dashboard-token')).toBe('abc123')
  })

  it('transitions connection state', () => {
    useBackendStore.getState().setConnecting()
    expect(useBackendStore.getState().connectionState).toBe('connecting')
    useBackendStore.getState().setConnected()
    expect(useBackendStore.getState().connectionState).toBe('connected')
    useBackendStore.getState().setFailed('boom')
    expect(useBackendStore.getState().connectionState).toBe('failed')
    expect(useBackendStore.getState().errorMessage).toBe('boom')
  })
})
```

- [ ] **Step 3: Run test to verify it fails**

Run: `pnpm --filter irsclawtauri test`
Expected: FAIL — `Failed to resolve import "./backend"`.

- [ ] **Step 4: Implement backend store**

Create `apps/IrsClawTauri/src/store/backend.ts`:
```ts
import { create } from 'zustand'

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'failed'

const URL_KEY = 'claw-backend-url'
const TOKEN_KEY = 'claw-dashboard-token'

interface BackendState {
  baseUrl: string
  token: string
  connectionState: ConnectionState
  errorMessage: string | null
  setBaseUrl: (url: string) => void
  setToken: (token: string) => void
  setConnecting: () => void
  setConnected: () => void
  setFailed: (msg: string) => void
  setDisconnected: () => void
}

export const useBackendStore = create<BackendState>((set) => ({
  baseUrl: localStorage.getItem(URL_KEY) || 'http://localhost:3000',
  token: localStorage.getItem(TOKEN_KEY) || '',
  connectionState: 'disconnected',
  errorMessage: null,
  setBaseUrl: (url) => { localStorage.setItem(URL_KEY, url); set({ baseUrl: url }) },
  setToken: (token) => { localStorage.setItem(TOKEN_KEY, token); set({ token }) },
  setConnecting: () => set({ connectionState: 'connecting', errorMessage: null }),
  setConnected: () => set({ connectionState: 'connected', errorMessage: null }),
  setFailed: (msg) => set({ connectionState: 'failed', errorMessage: msg }),
  setDisconnected: () => set({ connectionState: 'disconnected', errorMessage: null }),
}))
```

- [ ] **Step 5: Run test to verify it passes**

Run: `pnpm --filter irsclawtauri test`
Expected: PASS (2 tests).

- [ ] **Step 6: Implement api client**

Create `apps/IrsClawTauri/src/api/client.ts`:
```ts
import { useBackendStore } from '../store/backend'
import type { ApiResponse } from './types'

export function baseUrl(): string {
  return useBackendStore.getState().baseUrl.replace(/\/$/, '')
}

export function authHeaders(): Record<string, string> {
  const token = useBackendStore.getState().token
  if (!token) return {}
  return { Authorization: `Bearer ${token}` }
}

export async function authFetch(path: string, options?: RequestInit): Promise<Response> {
  return fetch(`${baseUrl()}/api${path}`, {
    ...options,
    headers: { ...authHeaders(), ...(options?.headers || {}) },
  })
}

export async function apiGet<T>(path: string): Promise<ApiResponse<T>> {
  const res = await authFetch(path)
  return res.json()
}

export async function apiJson<T>(path: string, method: string, body?: unknown): Promise<ApiResponse<T>> {
  const res = await authFetch(path, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  })
  return res.json()
}
```

- [ ] **Step 7: Typecheck**

Run: `pnpm --filter irsclawtauri exec tsc --noEmit`
Expected: 0 errors.

- [ ] **Step 8: Commit**
```bash
git add apps/IrsClawTauri/src/store/backend.ts apps/IrsClawTauri/src/store/backend.test.ts apps/IrsClawTauri/src/api/client.ts apps/IrsClawTauri/vitest.config.ts apps/IrsClawTauri/src/test-setup.ts
git commit -m "feat(IrsClawTauri): add backend store + api client with tests"
```

---

## Task 6: UI store

**Files:**
- Create: `apps/IrsClawTauri/src/store/ui.ts`

- [ ] **Step 1: Implement ui store**

Create `apps/IrsClawTauri/src/store/ui.ts`:
```ts
import { create } from 'zustand'

export type Tab = 'chat' | 'sessions' | 'tools' | 'skills' | 'plugins' | 'usage' | 'agents' | 'settings'
export type Theme = 'dark' | 'light'

const THEME_KEY = 'claw-theme'

interface UiState {
  theme: Theme
  selectedTab: Tab
  mobileDrawerOpen: boolean
  sidebarCollapsed: boolean
  setTheme: (t: Theme) => void
  toggleTheme: () => void
  setSelectedTab: (t: Tab) => void
  setMobileDrawerOpen: (open: boolean) => void
  toggleSidebar: () => void
}

export const useUiStore = create<UiState>((set) => ({
  theme: (localStorage.getItem(THEME_KEY) as Theme) || 'dark',
  selectedTab: 'chat',
  mobileDrawerOpen: false,
  sidebarCollapsed: false,
  setTheme: (t) => {
    localStorage.setItem(THEME_KEY, t)
    document.documentElement.setAttribute('data-theme', t)
    set({ theme: t })
  },
  toggleTheme: () => {
    const next: Theme = (localStorage.getItem(THEME_KEY) as Theme) === 'light' ? 'dark' : 'light'
    localStorage.setItem(THEME_KEY, next)
    document.documentElement.setAttribute('data-theme', next)
    set({ theme: next })
  },
  setSelectedTab: (t) => set({ selectedTab: t }),
  setMobileDrawerOpen: (open) => set({ mobileDrawerOpen: open }),
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
}))
```

- [ ] **Step 2: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/store/ui.ts
git commit -m "feat(IrsClawTauri): add UI store (theme/tab/drawer)"
```

---

## Task 7: API resource modules

**Files:**
- Create: `apps/IrsClawTauri/src/api/sessions.ts`
- Create: `apps/IrsClawTauri/src/api/agents.ts`
- Create: `apps/IrsClawTauri/src/api/stats.ts`
- Create: `apps/IrsClawTauri/src/api/chat.ts`

- [ ] **Step 1: sessions.ts**

Create `apps/IrsClawTauri/src/api/sessions.ts`. Adapt the session/chat functions from `crates/claw/dashboard-ui/src/api.ts` (lines 142-222) but use `apiGet`/`apiJson` from `./client` instead of `authFetch` directly. Include: `listSessions`, `getCurrentSession`, `createSession`, `switchSession`, `getSession`, `deleteSession`, `postFeedback`, `sendMessage`.

```ts
import { apiGet, apiJson } from './client'
import type { ApiResponse, SessionMeta, CurrentSession } from './types'

export const listSessions = () => apiGet<SessionMeta[]>('/sessions')
export const getCurrentSession = () => apiGet<CurrentSession>('/sessions/current')
export const createSession = (agentId?: string) =>
  apiJson<{ id: string; title: string; message_count: number; agent_id: string }>('/sessions', 'POST', agentId ? { agent_id: agentId } : {})
export const switchSession = (id: string) =>
  apiJson<{ id: string; title: string | null; message_count: number }>(`/sessions/${encodeURIComponent(id)}/switch`, 'POST')
export const getSession = (id: string) =>
  apiGet<{ id: string; title: string; messages: { role: string; content: string; reasoning?: string }[]; agent_id?: string }>(`/sessions/${encodeURIComponent(id)}`)
export const deleteSession = (id: string) => apiJson<string>(`/sessions/${encodeURIComponent(id)}`, 'DELETE')
export const postFeedback = (sessionId: string, positive: boolean, message?: string) =>
  apiJson<string>(`/sessions/${encodeURIComponent(sessionId)}/feedback`, 'POST', { positive, ...(message ? { message } : {}) })
export const sendMessage = (message: string, agentId?: string) =>
  apiJson<{ session_id: string; status: string }>('/chat', 'POST', { message, ...(agentId ? { agent_id: agentId } : {}) })
```

- [ ] **Step 2: agents.ts**

Create `apps/IrsClawTauri/src/api/agents.ts`. Adapt from dashboard-ui api.ts lines 226-259:
```ts
import { apiGet, apiJson } from './client'
import type { ApiResponse, AgentInfo } from './types'

export const listAgents = () => apiGet<AgentInfo[]>('/agents')
export const getAgentConfig = (id: string) => apiGet<AgentInfo>(`/agents/${encodeURIComponent(id)}`)
export const createAgent = (body: Record<string, unknown>) => apiJson<{ id: string; status: string }>('/agents', 'POST', body)
export const updateAgent = (id: string, body: Record<string, unknown>) => apiJson<{ id: string; status: string }>(`/agents/${encodeURIComponent(id)}`, 'PUT', body)
export const deleteAgent = (id: string) => apiJson<{ id: string; status: string }>(`/agents/${encodeURIComponent(id)}`, 'DELETE')
```

- [ ] **Step 3: stats.ts**

Create `apps/IrsClawTauri/src/api/stats.ts`:
```ts
import { apiGet } from './client'
import type { ApiResponse, StatsResponse } from './types'

export const getStats = (period: string = 'all') =>
  apiGet<StatsResponse>(`/stats?period=${encodeURIComponent(period)}`)
```

- [ ] **Step 4: chat.ts (SSE stream parser)**

Create `apps/IrsClawTauri/src/api/chat.ts`. Adapt the `streamChat` function from dashboard-ui api.ts lines 354-441, but use `authFetch` from `./client` (dynamic baseUrl) instead of the hardcoded `BASE`:
```ts
import { authFetch } from './client'
import type { SseEventHandler } from './types'

export function streamChat(sessionId: string, handlers: SseEventHandler): AbortController {
  const controller = new AbortController()
  authFetch(`/chat/stream/${encodeURIComponent(sessionId)}`, { signal: controller.signal })
    .then(async (response) => {
      const reader = response.body?.getReader()
      if (!reader) return
      const decoder = new TextDecoder()
      let buffer = ''
      let currentEvent = ''
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() || ''
        for (const line of lines) {
          const trimmed = line.trim()
          if (trimmed.startsWith('event: ')) { currentEvent = trimmed.slice(7).trim(); continue }
          if (!trimmed.startsWith('data: ')) continue
          const data = trimmed.slice(6)
          switch (currentEvent) {
            case 'token': handlers.onToken?.(data); break
            case 'reasoning': handlers.onReasoning?.(data); break
            case 'status': handlers.onStatus?.(data); break
            case 'error': handlers.onError?.(data); break
            case 'new_round': handlers.onNewRound?.(); break
            case 'tool_executed': try { handlers.onToolExecuted?.(JSON.parse(data)) } catch { /* ignore */ } break
            case 'image_generated': try { handlers.onImageGenerated?.(JSON.parse(data)) } catch { /* ignore */ } break
            case 'done': try { const p = JSON.parse(data); handlers.onDone?.(p.usage, p.quality) } catch { /* ignore */ } break
            case 'evaluation': try { handlers.onEvaluation?.(JSON.parse(data)) } catch { /* ignore */ } break
            case 'quality_score': try { handlers.onQualityScore?.(JSON.parse(data)) } catch { /* ignore */ } break
          }
        }
      }
    })
    .catch((err) => { if (err.name !== 'AbortError') handlers.onError?.(String(err)) })
  return controller
}
```

- [ ] **Step 5: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/api/
git commit -m "feat(IrsClawTauri): add API resource modules (sessions/agents/stats/chat SSE)"
```

---

## Task 8: Hooks (useMediaQuery, useTheme, useBackendHealth)

**Files:**
- Create: `apps/IrsClawTauri/src/hooks/useMediaQuery.ts`
- Create: `apps/IrsClawTauri/src/hooks/useTheme.ts`
- Create: `apps/IrsClawTauri/src/hooks/useBackendHealth.ts`

- [ ] **Step 1: useMediaQuery**

Create `apps/IrsClawTauri/src/hooks/useMediaQuery.ts`:
```ts
import { useState, useEffect } from 'react'

export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() => window.matchMedia(query).matches)
  useEffect(() => {
    const mql = window.matchMedia(query)
    const handler = (e: MediaQueryListEvent) => setMatches(e.matches)
    mql.addEventListener('change', handler)
    setMatches(mql.matches)
    return () => mql.removeEventListener('change', handler)
  }, [query])
  return matches
}

export const useIsDesktop = () => useMediaQuery('(min-width: 768px)')
```

- [ ] **Step 2: useTheme**

Create `apps/IrsClawTauri/src/hooks/useTheme.ts`:
```ts
import { useEffect } from 'react'
import { useUiStore } from '../store/ui'

export function useTheme() {
  const theme = useUiStore((s) => s.theme)
  const toggleTheme = useUiStore((s) => s.toggleTheme)
  useEffect(() => { document.documentElement.setAttribute('data-theme', theme) }, [theme])
  return { theme, toggleTheme }
}
```

- [ ] **Step 3: useBackendHealth**

Create `apps/IrsClawTauri/src/hooks/useBackendHealth.ts`:
```ts
import { useEffect, useRef } from 'react'
import { useBackendStore } from '../store/backend'
import { baseUrl } from '../api/client'

export function useBackendHealth(intervalMs: number = 15000) {
  const setConnecting = useBackendStore((s) => s.setConnecting)
  const setConnected = useBackendStore((s) => s.setConnected)
  const setFailed = useBackendStore((s) => s.setFailed)
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null)

  useEffect(() => {
    let cancelled = false
    const check = async () => {
      setConnecting()
      try {
        const res = await fetch(`${baseUrl()}/api/health`)
        if (cancelled) return
        if (res.ok) setConnected()
        else setFailed(`HTTP ${res.status}`)
      } catch (e) { if (!cancelled) setFailed(String(e)) }
    }
    check()
    timerRef.current = setInterval(check, intervalMs)
    return () => { cancelled = true; if (timerRef.current) clearInterval(timerRef.current) }
  }, [intervalMs, setConnecting, setConnected, setFailed])
}
```

- [ ] **Step 4: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/hooks/
git commit -m "feat(IrsClawTauri): add useMediaQuery/useTheme/useBackendHealth hooks"
```

---

## Task 9: useChatStream + useSessionLoader hooks

**Files:**
- Create: `apps/IrsClawTauri/src/hooks/useChatStream.ts`
- Create: `apps/IrsClawTauri/src/hooks/useSessionLoader.ts`

- [ ] **Step 1: useChatStream**

Create `apps/IrsClawTauri/src/hooks/useChatStream.ts`. Port `crates/claw/dashboard-ui/src/hooks/useChatStream.ts` verbatim, with two changes: (1) replace the hardcoded `const BASE = window.location.pathname...` and `authHeaders()` with imports from `../api/client` (`baseUrl`, `authHeaders`); (2) import types from `../api/types` instead of `../api`. Keep `processSse` using `${baseUrl()}/api/chat`. Drop `resumeStream`/`lastCursor` (YAGNI for first phase) — return only `{ streaming, state, sendMessage, abort }`.

- [ ] **Step 2: useSessionLoader**

Create `apps/IrsClawTauri/src/hooks/useSessionLoader.ts`. Port `crates/claw/dashboard-ui/src/hooks/useSessionLoader.ts` verbatim, changing imports: `getCurrentSession`/`createSession`/`listSessions`/`switchSession` from `../api/sessions`, types from `../api/types`. Keep `deserializeMessages` and the full `load`/`newChat` logic. Return the same shape (minus `setSessionId` if unused — keep it for parity).

- [ ] **Step 3: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/hooks/useChatStream.ts apps/IrsClawTauri/src/hooks/useSessionLoader.ts
git commit -m "feat(IrsClawTauri): add useChatStream + useSessionLoader hooks"
```

---

## Task 10: Common components

**Files:**
- Create: `apps/IrsClawTauri/src/components/common/AgentChip.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/components/common/EmptyState.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/components/common/PulsingDot.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/components/common/Toast.tsx` + `.module.css`

- [ ] **Step 1: AgentChip**

Create `apps/IrsClawTauri/src/components/common/AgentChip.module.css`:
```css
.chip { display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 8px; cursor: pointer; border: 1px solid var(--border); font-family: inherit; font-size: 12px; font-weight: 600; background: var(--bg-tertiary); color: var(--text-secondary); transition: all var(--duration-fast) var(--ease-out); }
.chip:hover { border-color: var(--border-light); }
.chip.active { background: var(--accent); color: #000; border-color: var(--accent); }
.icon { display: flex; align-items: center; justify-content: center; }
```
Create `apps/IrsClawTauri/src/components/common/AgentChip.tsx`:
```tsx
import { Bot, Star } from 'lucide-react'
import styles from './AgentChip.module.css'

interface Props { id: string; active: boolean; onClick: () => void }

export default function AgentChip({ id, active, onClick }: Props) {
  return (
    <button className={`${styles.chip} ${active ? styles.active : ''}`} onClick={onClick}>
      <span className={styles.icon}>{id === 'default' ? <Star size={11} /> : <Bot size={11} />}</span>
      {id}
    </button>
  )
}
```

- [ ] **Step 2: EmptyState**

Create `apps/IrsClawTauri/src/components/common/EmptyState.module.css`:
```css
.empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 60px 24px; text-align: center; animation: fadeIn var(--duration-slow) var(--ease-out); }
.icon { width: 64px; height: 64px; border-radius: 20px; background: linear-gradient(135deg, var(--ai-muted), var(--accent-muted)); display: flex; align-items: center; justify-content: center; color: var(--ai-primary); margin-bottom: 24px; animation: float 3s ease-in-out infinite; }
.title { font-size: 22px; font-weight: 700; letter-spacing: -0.03em; margin-bottom: 8px; }
.desc { color: var(--text-secondary); margin-bottom: 24px; font-size: 14px; }
```
Create `apps/IrsClawTauri/src/components/common/EmptyState.tsx`:
```tsx
import type { ReactNode } from 'react'
import styles from './EmptyState.module.css'

interface Props { icon: ReactNode; title: string; description?: string; action?: ReactNode }

export default function EmptyState({ icon, title, description, action }: Props) {
  return (
    <div className={styles.empty}>
      <div className={styles.icon}>{icon}</div>
      <h3 className={styles.title}>{title}</h3>
      {description && <p className={styles.desc}>{description}</p>}
      {action}
    </div>
  )
}
```

- [ ] **Step 3: PulsingDot**

Create `apps/IrsClawTauri/src/components/common/PulsingDot.module.css`:
```css
.wrap { position: relative; width: 14px; height: 14px; }
.halo { position: absolute; inset: 0; border-radius: 50%; background: var(--error); opacity: 0.2; animation: pulse 1s ease-in-out infinite; }
.dot { position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 6px; height: 6px; border-radius: 50%; background: var(--error); }
```
Create `apps/IrsClawTauri/src/components/common/PulsingDot.tsx`:
```tsx
import styles from './PulsingDot.module.css'
export default function PulsingDot() {
  return <span className={styles.wrap}><span className={styles.halo} /><span className={styles.dot} /></span>
}
```

- [ ] **Step 4: Toast**

Create `apps/IrsClawTauri/src/components/common/Toast.module.css`:
```css
.toast { position: fixed; bottom: 24px; left: 50%; transform: translateX(-50%); padding: 12px 18px; border-radius: var(--radius); font-size: 13px; font-weight: 500; z-index: 1000; animation: slideUp var(--duration-normal) var(--ease-out); box-shadow: var(--shadow-lg); max-width: 90vw; }
.error { background: var(--error-bg); color: var(--error); border: 1px solid rgba(248,113,113,0.3); }
.success { background: var(--success-bg); color: var(--success); border: 1px solid rgba(74,222,128,0.3); }
```
Create `apps/IrsClawTauri/src/components/common/Toast.tsx`:
```tsx
import { useEffect, useState } from 'react'
import styles from './Toast.module.css'

type ToastType = 'error' | 'success'
interface ToastItem { id: number; message: string; type: ToastType }

let counter = 0
const listeners = new Set<(items: ToastItem[]) => void>()
let items: ToastItem[] = []

export function toast(message: string, type: ToastType = 'error') {
  const id = ++counter
  items = [...items, { id, message, type }]
  listeners.forEach((l) => l(items))
  setTimeout(() => { items = items.filter((i) => i.id !== id); listeners.forEach((l) => l(items)) }, 3500)
}

export function ToastContainer() {
  const [current, setCurrent] = useState<ToastItem[]>([])
  useEffect(() => {
    const l = (i: ToastItem[]) => setCurrent(i)
    listeners.add(l)
    return () => { listeners.delete(l) }
  }, [])
  return <>{current.map((t) => <div key={t.id} className={`${styles.toast} ${styles[t.type]}`}>{t.message}</div>)}</>
}
```

- [ ] **Step 5: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/components/common/
git commit -m "feat(IrsClawTauri): add common components (AgentChip/EmptyState/PulsingDot/Toast)"
```

---

## Task 11: Chat components (port from dashboard-ui)

**Files:**
- Create: `apps/IrsClawTauri/src/components/chat/MarkdownRenderer.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/components/chat/ToolCallCard.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/components/chat/MessageBubble.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/components/chat/StreamingBubble.tsx` + `.module.css`

- [ ] **Step 1: Copy module CSS from dashboard-ui**

Copy verbatim from `crates/claw/dashboard-ui/src/components/` to `apps/IrsClawTauri/src/components/chat/`:
```bash
cp crates/claw/dashboard-ui/src/components/MarkdownRenderer.module.css apps/IrsClawTauri/src/components/chat/
cp crates/claw/dashboard-ui/src/components/ToolCallCard.module.css apps/IrsClawTauri/src/components/chat/
cp crates/claw/dashboard-ui/src/components/MessageBubble.module.css apps/IrsClawTauri/src/components/chat/
cp crates/claw/dashboard-ui/src/components/StreamingBubble.module.css apps/IrsClawTauri/src/components/chat/
```

- [ ] **Step 2: Port the 4 .tsx components**

Port each from `crates/claw/dashboard-ui/src/components/` to `apps/IrsClawTauri/src/components/chat/`. The only change in each: replace `import type { ... } from '../api'` with `import type { ... } from '../../api/types'`, and `import ToolCallCard from './ToolCallCard'` / `import MarkdownRenderer from './MarkdownRenderer'` stay the same (now local). Keep all JSX, props, and logic identical.

Source files to port (read, change import path, write to new location):
- `MarkdownRenderer.tsx` (65 lines)
- `ToolCallCard.tsx` (55 lines)
- `MessageBubble.tsx` (163 lines)
- `StreamingBubble.tsx` (58 lines)

- [ ] **Step 3: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/components/chat/
git commit -m "feat(IrsClawTauri): port chat components (Markdown/ToolCall/MessageBubble/Streaming)"
```

---

## Task 12: FloatingInput + MessageList

**Files:**
- Create: `apps/IrsClawTauri/src/components/chat/FloatingInput.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/components/chat/MessageList.tsx`

- [ ] **Step 1: FloatingInput CSS**

Create `apps/IrsClawTauri/src/components/chat/FloatingInput.module.css`:
```css
.wrap { padding: 8px 12px 12px; }
.bar { display: flex; align-items: flex-end; gap: 8px; padding: 8px 14px; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 28px; box-shadow: var(--shadow-md); transition: border-color var(--duration-fast) var(--ease-out); }
.bar:focus-within { border-color: var(--accent); }
.mic { flex-shrink: 0; width: 36px; height: 36px; border-radius: 50%; border: none; background: var(--bg-tertiary); color: var(--text-secondary); cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all var(--duration-fast); }
.mic:hover { color: var(--text-primary); }
.mic.recording { color: var(--error); }
.input { flex: 1; border: none; outline: none; background: none; color: var(--text-primary); font-family: inherit; font-size: 14px; line-height: 1.5; resize: none; max-height: 140px; padding: 8px 0; }
.input::placeholder { color: var(--text-muted); }
.send { flex-shrink: 0; width: 36px; height: 36px; border-radius: 50%; border: none; background: var(--bg-tertiary); color: var(--text-muted); cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all var(--duration-fast) var(--ease-spring); }
.send.active { background: var(--accent); color: #000; }
.send:disabled { opacity: 0.5; cursor: not-allowed; }
```

- [ ] **Step 2: FloatingInput component**

Create `apps/IrsClawTauri/src/components/chat/FloatingInput.tsx`:
```tsx
import { useRef, useState, useEffect } from 'react'
import { Mic, ArrowUp } from 'lucide-react'
import styles from './FloatingInput.module.css'

interface Props {
  onSend: (text: string) => void
  disabled: boolean
  onMic?: () => void
  recording?: boolean
}

export default function FloatingInput({ onSend, disabled, onMic, recording }: Props) {
  const [value, setValue] = useState('')
  const taRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    const ta = taRef.current
    if (!ta) return
    ta.style.height = 'auto'
    ta.style.height = `${Math.min(ta.scrollHeight, 140)}px`
  }, [value])

  const handleSend = () => {
    const text = value.trim()
    if (!text || disabled) return
    onSend(text)
    setValue('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSend() }
  }

  const hasContent = value.trim().length > 0

  return (
    <div className={styles.wrap}>
      <div className={styles.bar}>
        {onMic && (
          <button className={`${styles.mic} ${recording ? styles.recording : ''}`} onClick={onMic} disabled={disabled} title="Voice input">
            <Mic size={16} />
          </button>
        )}
        <textarea
          ref={taRef}
          className={styles.input}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Message i-rs-claw..."
          rows={1}
          disabled={disabled}
        />
        <button className={`${styles.send} ${hasContent ? styles.active : ''}`} onClick={handleSend} disabled={disabled || !hasContent} title="Send (Enter)">
          <ArrowUp size={16} />
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 3: MessageList**

Create `apps/IrsClawTauri/src/components/chat/MessageList.tsx`:
```tsx
import { useEffect, useRef } from 'react'
import type { ChatMessage } from '../../api/types'
import MessageBubble from './MessageBubble'
import StreamingBubble from './StreamingBubble'

interface Props {
  messages: ChatMessage[]
  sessionId: string | null
  feedbackSubmitted: Set<number>
  onFeedback: (i: number, p: boolean) => void
  streaming: boolean
  streamDisplay: { content: string; reasoning: string; toolCalls: ChatMessage['toolCalls'] }
}

export default function MessageList({ messages, sessionId, feedbackSubmitted, onFeedback, streaming, streamDisplay }: Props) {
  const endRef = useRef<HTMLDivElement>(null)
  const hasStreaming = streaming && (streamDisplay.content || streamDisplay.reasoning || (streamDisplay.toolCalls && streamDisplay.toolCalls.length > 0))

  useEffect(() => {
    if (streaming) endRef.current?.scrollIntoView({ behavior: 'instant' })
  }, [streamDisplay?.content, streamDisplay?.reasoning, streamDisplay?.toolCalls?.length, streaming])

  useEffect(() => {
    if (messages.length > 0) endRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  return (
    <div style={{ flex: 1, overflowY: 'auto', padding: '16px', display: 'flex', flexDirection: 'column', gap: '12px' }}>
      {messages.map((msg, i) => (
        <MessageBubble key={i} message={msg} index={i} onFeedback={onFeedback} hasFeedback={feedbackSubmitted.has(i)} sessionId={sessionId} />
      ))}
      {hasStreaming && streamDisplay && <StreamingBubble display={streamDisplay as any} />}
      {streaming && !hasStreaming && (
        <div className="message status"><div className="typing-dots"><span></span><span></span><span></span></div>Thinking...</div>
      )}
      <div ref={endRef} />
    </div>
  )
}
```

- [ ] **Step 4: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/components/chat/FloatingInput.* apps/IrsClawTauri/src/components/chat/MessageList.tsx
git commit -m "feat(IrsClawTauri): add FloatingInput + MessageList"
```

---

## Task 13: ConnectScreen (disconnected view)

**Files:**
- Create: `apps/IrsClawTauri/src/components/shell/ConnectScreen.tsx` + `.module.css`

- [ ] **Step 1: ConnectScreen CSS**

Create `apps/IrsClawTauri/src/components/shell/ConnectScreen.module.css`:
```css
.screen { display: flex; align-items: center; justify-content: center; min-height: 100vh; background: var(--bg-primary); padding: 24px; }
.card { width: 100%; max-width: 420px; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius-xl); padding: 40px 32px; text-align: center; animation: scaleIn var(--duration-normal) var(--ease-out); }
.icon { width: 56px; height: 56px; margin: 0 auto 20px; border-radius: 16px; background: linear-gradient(135deg, var(--accent-muted), var(--ai-muted)); display: flex; align-items: center; justify-content: center; color: var(--accent); }
.title { font-size: 20px; font-weight: 700; margin-bottom: 8px; }
.desc { color: var(--text-secondary); font-size: 14px; margin-bottom: 24px; line-height: 1.6; }
.field { width: 100%; padding: 12px 16px; background: var(--bg-tertiary); border: 1px solid var(--border); border-radius: var(--radius); color: var(--text-primary); font-size: 14px; margin-bottom: 12px; font-family: inherit; transition: all var(--duration-fast); }
.field:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-muted); }
.err { color: var(--error); font-size: 13px; margin-bottom: 12px; }
.btn { width: 100%; }
.spinner { width: 16px; height: 16px; border: 2px solid var(--border); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.6s linear infinite; }
.status { display: flex; align-items: center; justify-content: center; gap: 10px; color: var(--text-muted); margin-top: 16px; font-size: 13px; }
```

- [ ] **Step 2: ConnectScreen component**

Create `apps/IrsClawTauri/src/components/shell/ConnectScreen.tsx`:
```tsx
import { useState } from 'react'
import { Plug } from 'lucide-react'
import { useBackendStore } from '../../store/backend'
import styles from './ConnectScreen.module.css'

export default function ConnectScreen() {
  const { baseUrl: storedUrl, token, setBaseUrl, setToken, connectionState, errorMessage, setConnecting, setConnected, setFailed } = useBackendStore()
  const [url, setUrl] = useState(storedUrl)
  const [tok, setTok] = useState(token)
  const isConnecting = connectionState === 'connecting'

  const handleConnect = async () => {
    const cleanUrl = url.trim().replace(/\/$/, '')
    setBaseUrl(cleanUrl)
    setToken(tok.trim())
    setConnecting()
    try {
      const res = await fetch(`${cleanUrl}/api/health`)
      if (res.ok) setConnected()
      else setFailed(`HTTP ${res.status}`)
    } catch (e) { setFailed(String(e)) }
  }

  return (
    <div className={styles.screen}>
      <div className={styles.card}>
        <div className={styles.icon}><Plug size={28} /></div>
        <h2 className={styles.title}>Connect to i-rs-claw Backend</h2>
        <p className={styles.desc}>Enter the URL of a running <code>claw serve</code> instance and its auth token.</p>
        <input className={styles.field} value={url} onChange={(e) => setUrl(e.target.value)} placeholder="http://localhost:3000" autoFocus />
        <input className={styles.field} type="password" value={tok} onChange={(e) => setTok(e.target.value)} placeholder="Auth token" onKeyDown={(e) => { if (e.key === 'Enter') handleConnect() }} />
        {errorMessage && <div className={styles.err}>{errorMessage}</div>}
        <button className={`btn btn-primary ${styles.btn}`} onClick={handleConnect} disabled={isConnecting || !url.trim()}>
          {isConnecting ? 'Connecting...' : 'Connect'}
        </button>
        {isConnecting && <div className={styles.status}><span className={styles.spinner} />Checking health...</div>}
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/components/shell/ConnectScreen.*
git commit -m "feat(IrsClawTauri): add ConnectScreen (disconnected backend view)"
```

---

## Task 14: Sidebar (desktop)

**Files:**
- Create: `apps/IrsClawTauri/src/components/shell/Sidebar.tsx` + `.module.css`

- [ ] **Step 1: Sidebar CSS**

Create `apps/IrsClawTauri/src/components/shell/Sidebar.module.css`:
```css
.sidebar { width: var(--sidebar-width); min-width: var(--sidebar-width); height: 100vh; background: var(--bg-secondary); border-right: 1px solid var(--border); display: flex; flex-direction: column; overflow: hidden; }
.header { padding: 20px 20px 16px; }
.header h1 { font-size: 17px; font-weight: 700; letter-spacing: -0.03em; background: linear-gradient(135deg, var(--text-primary), var(--accent)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
.subtitle { font-size: 11px; color: var(--text-muted); margin-top: 4px; }
.agentRow { padding: 0 12px 12px; display: flex; gap: 6px; flex-wrap: wrap; }
.nav { flex: 1; padding: 8px 12px; overflow-y: auto; }
.label { padding: 10px 12px 4px; font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.08em; color: var(--text-tertiary); }
.item { display: flex; align-items: center; gap: 10px; padding: 9px 12px; border-radius: var(--radius); cursor: pointer; color: var(--text-secondary); font-size: 13px; font-weight: 500; border: none; background: none; width: 100%; text-align: left; transition: all var(--duration-fast) var(--ease-out); }
.item:hover { background: var(--bg-hover); color: var(--text-primary); }
.item.active { background: var(--accent-muted); color: var(--accent); }
.icon { width: 20px; height: 20px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
.sessionItem { display: flex; align-items: center; gap: 8px; padding: 7px 12px; border-radius: var(--radius); cursor: pointer; font-size: 13px; color: var(--text-secondary); border: none; background: none; width: 100%; text-align: left; transition: all var(--duration-fast); }
.sessionItem:hover { background: var(--bg-hover); color: var(--text-primary); }
.sessionItem.active { background: var(--accent-muted); color: var(--accent); }
.sessionTitle { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.footer { padding: 10px 12px; border-top: 1px solid var(--border); display: flex; justify-content: flex-end; align-items: center; }
.themeBtn { width: 34px; height: 34px; border-radius: var(--radius); border: none; background: var(--bg-tertiary); color: var(--text-secondary); cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all var(--duration-fast); }
.themeBtn:hover { color: var(--accent); }
.delBtn { opacity: 0; background: none; border: none; color: var(--text-muted); cursor: pointer; padding: 2px; transition: opacity var(--duration-fast); }
.sessionItem:hover .delBtn { opacity: 1; }
.delBtn:hover { color: var(--error); }
```

- [ ] **Step 2: Sidebar component**

Create `apps/IrsClawTauri/src/components/shell/Sidebar.tsx`:
```tsx
import { useEffect, useState } from 'react'
import { useLocation } from 'wouter'
import { Sun, Moon, Trash2, Plus, MessageSquare, Wrench, BookOpen, Puzzle, BarChart3, Bot, Settings } from 'lucide-react'
import { useUiStore, type Tab } from '../../store/ui'
import { useBackendStore } from '../../store/backend'
import { listSessions, createSession, deleteSession, switchSession } from '../../api/sessions'
import { listAgents } from '../../api/agents'
import type { SessionMeta, AgentInfo } from '../../api/types'
import AgentChip from '../common/AgentChip'
import styles from './Sidebar.module.css'

const NAV: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'sessions', label: 'Sessions', icon: <MessageSquare size={16} /> },
  { id: 'agents', label: 'Agents', icon: <Bot size={16} /> },
  { id: 'usage', label: 'Usage', icon: <BarChart3 size={16} /> },
  { id: 'tools', label: 'Tools', icon: <Wrench size={16} /> },
  { id: 'skills', label: 'Skills', icon: <BookOpen size={16} /> },
  { id: 'plugins', label: 'Plugins', icon: <Puzzle size={16} /> },
  { id: 'settings', label: 'Settings', icon: <Settings size={16} /> },
]

export default function Sidebar() {
  const [location, setLocation] = useLocation()
  const { theme, toggleTheme, setSelectedTab } = useUiStore()
  const connectionState = useBackendStore((s) => s.connectionState)
  const [sessions, setSessions] = useState<SessionMeta[]>([])
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const [selectedAgent, setSelectedAgent] = useState('default')
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null)

  const refresh = async () => {
    if (connectionState !== 'connected') return
    const [s, a] = await Promise.all([listSessions(), listAgents()])
    if (s.success && s.data) setSessions(s.data)
    if (a.success && a.data) setAgents(a.data.filter((x) => !x.is_sub_agent))
  }

  useEffect(() => { refresh() }, [connectionState])

  const go = (tab: Tab) => { setSelectedTab(tab); setLocation(tab === 'chat' ? '/' : `/${tab}`) }

  const handleNewChat = async () => {
    const r = await createSession(selectedAgent !== 'default' ? selectedAgent : undefined)
    if (r.success && r.data) { await refresh(); go('chat') }
  }

  const handleSwitch = async (id: string) => { await switchSession(id); setCurrentSessionId(id); go('chat') }

  const handleDelete = async (id: string) => {
    await deleteSession(id)
    if (currentSessionId === id) setCurrentSessionId(null)
    await refresh()
  }

  const currentTab: Tab = location === '/' ? 'chat' : (location.slice(1) as Tab) || 'chat'

  return (
    <aside className={styles.sidebar}>
      <div className={styles.header}>
        <h1>i-rs-claw</h1>
        <div className={styles.subtitle}>AI Personal Assistant</div>
      </div>
      <div className={styles.agentRow}>
        {agents.length <= 1 ? (
          <AgentChip id={agents[0]?.id || 'default'} active onClick={() => {}} />
        ) : (
          agents.map((a) => <AgentChip key={a.id} id={a.id} active={a.id === selectedAgent} onClick={() => setSelectedAgent(a.id)} />)
        )}
      </div>
      <nav className={styles.nav}>
        <div className={styles.label}>Sessions</div>
        <button className={styles.item} onClick={handleNewChat} disabled={connectionState !== 'connected'} style={{ marginBottom: 4 }}>
          <span className={styles.icon}><Plus size={15} /></span>New Chat
        </button>
        {sessions.slice(0, 12).map((s) => (
          <div key={s.id} className={`${styles.sessionItem} ${s.id === currentSessionId ? styles.active : ''}`} onClick={() => handleSwitch(s.id)}>
            <MessageSquare size={13} />
            <span className={styles.sessionTitle}>{s.title}</span>
            <button className={styles.delBtn} onClick={(e) => { e.stopPropagation(); handleDelete(s.id) }}><Trash2 size={12} /></button>
          </div>
        ))}
        <div className={styles.label}>Manage</div>
        {NAV.map((n) => (
          <button key={n.id} className={`${styles.item} ${currentTab === n.id ? styles.active : ''}`} onClick={() => go(n.id)}>
            <span className={styles.icon}>{n.icon}</span>{n.label}
          </button>
        ))}
      </nav>
      <div className={styles.footer}>
        <button className={styles.themeBtn} onClick={toggleTheme} title="Toggle theme">
          {theme === 'dark' ? <Sun size={15} /> : <Moon size={15} />}
        </button>
      </div>
    </aside>
  )
}
```

- [ ] **Step 3: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/components/shell/Sidebar.*
git commit -m "feat(IrsClawTauri): add desktop Sidebar"
```

---

## Task 15: DesktopShell + MobileShell + BottomTabBar + Drawer

**Files:**
- Create: `apps/IrsClawTauri/src/components/shell/DesktopShell.tsx`
- Create: `apps/IrsClawTauri/src/components/shell/MobileShell.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/components/shell/BottomTabBar.tsx`
- Create: `apps/IrsClawTauri/src/components/shell/Drawer.tsx` + `.module.css`

- [ ] **Step 1: DesktopShell**

Create `apps/IrsClawTauri/src/components/shell/DesktopShell.tsx`:
```tsx
import type { ReactNode } from 'react'
import Sidebar from './Sidebar'
import '../../styles/responsive.css'

interface Props { children: ReactNode }

export default function DesktopShell({ children }: Props) {
  return (
    <div className="app-shell">
      <Sidebar />
      <main className="app-detail">{children}</main>
    </div>
  )
}
```

- [ ] **Step 2: MobileShell CSS**

Create `apps/IrsClawTauri/src/components/shell/MobileShell.module.css`:
```css
.shell { display: flex; flex-direction: column; width: 100%; height: 100vh; background: var(--bg-primary); }
.topbar { display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; border-bottom: 1px solid var(--border); background: var(--bg-secondary); min-height: 50px; }
.title { font-size: 15px; font-weight: 600; }
.btn { background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; align-items: center; justify-content: center; border-radius: 8px; }
.btn:hover { background: var(--bg-hover); color: var(--text-primary); }
.body { flex: 1; display: flex; flex-direction: column; overflow: hidden; min-height: 0; }
.tabbar { display: flex; justify-content: space-around; padding: 8px 0 calc(8px + env(safe-area-inset-bottom)); border-top: 1px solid var(--border); background: var(--bg-secondary); }
.tab { background: none; border: none; color: var(--text-muted); cursor: pointer; display: flex; flex-direction: column; align-items: center; gap: 3px; font-size: 10px; font-family: inherit; padding: 4px 8px; border-radius: 8px; transition: color var(--duration-fast); }
.tab.active { color: var(--accent); }
.overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.4); z-index: 50; animation: fadeIn var(--duration-fast); }
```

- [ ] **Step 3: BottomTabBar**

Create `apps/IrsClawTauri/src/components/shell/BottomTabBar.tsx`:
```tsx
import { MessageSquare, History, MoreHorizontal, BarChart3, Settings } from 'lucide-react'
import { useUiStore, type Tab } from '../../store/ui'
import styles from './MobileShell.module.css'

interface Props { onNavigate: (t: Tab) => void }

const TABS: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'chat', label: 'Chat', icon: <MessageSquare size={20} /> },
  { id: 'sessions', label: 'Sessions', icon: <History size={20} /> },
  { id: 'agents', label: 'More', icon: <MoreHorizontal size={20} /> },
  { id: 'usage', label: 'Usage', icon: <BarChart3 size={20} /> },
  { id: 'settings', label: 'Settings', icon: <Settings size={20} /> },
]

export default function BottomTabBar({ onNavigate }: Props) {
  const selectedTab = useUiStore((s) => s.selectedTab)
  return (
    <nav className={styles.tabbar}>
      {TABS.map((t) => (
        <button key={t.id} className={`${styles.tab} ${selectedTab === t.id ? styles.active : ''}`} onClick={() => onNavigate(t.id)}>
          {t.icon}<span>{t.label}</span>
        </button>
      ))}
    </nav>
  )
}
```

- [ ] **Step 4: Drawer CSS + component**

Create `apps/IrsClawTauri/src/components/shell/Drawer.module.css`:
```css
.drawer { position: fixed; top: 0; left: 0; bottom: 0; width: 80vw; max-width: 320px; background: var(--bg-secondary); border-right: 1px solid var(--border); z-index: 60; display: flex; flex-direction: column; animation: slideInLeft var(--duration-normal) var(--ease-out); padding: 16px; overflow-y: auto; }
.section { margin-bottom: 20px; }
.label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); margin-bottom: 8px; }
.grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.card { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 16px 8px; background: var(--bg-tertiary); border: 1px solid var(--border); border-radius: var(--radius-lg); cursor: pointer; transition: all var(--duration-fast); }
.card:hover { border-color: var(--accent); }
.cardIcon { width: 40px; height: 40px; border-radius: 10px; background: var(--accent-muted); display: flex; align-items: center; justify-content: center; color: var(--accent); }
.cardLabel { font-size: 12px; font-weight: 500; }
.chips { display: flex; gap: 6px; flex-wrap: wrap; }
```
Create `apps/IrsClawTauri/src/components/shell/Drawer.tsx`:
```tsx
import { useEffect, useState } from 'react'
import { Wrench, BookOpen, Puzzle, Bot } from 'lucide-react'
import { useUiStore, type Tab } from '../../store/ui'
import { listAgents } from '../../api/agents'
import type { AgentInfo } from '../../api/types'
import AgentChip from '../common/AgentChip'
import styles from './Drawer.module.css'

interface Props { onClose: () => void; onNavigate: (t: Tab) => void }

const MORE: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'tools', label: 'Tools', icon: <Wrench size={18} /> },
  { id: 'skills', label: 'Skills', icon: <BookOpen size={18} /> },
  { id: 'plugins', label: 'Plugins', icon: <Puzzle size={18} /> },
  { id: 'agents', label: 'Agents', icon: <Bot size={18} /> },
]

export default function Drawer({ onClose, onNavigate }: Props) {
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const setSelectedTab = useUiStore((s) => s.setSelectedTab)

  useEffect(() => { listAgents().then((r) => { if (r.success && r.data) setAgents(r.data.filter((a) => !a.is_sub_agent)) }) }, [])

  const nav = (t: Tab) => { setSelectedTab(t); onNavigate(t); onClose() }

  return (
    <div className={styles.drawer}>
      <div className={styles.section}>
        <div className={styles.label}>Agent</div>
        <div className={styles.chips}>
          {agents.length === 0 ? <AgentChip id="default" active onClick={() => nav('agents')} /> : agents.map((a) => <AgentChip key={a.id} id={a.id} active={a.id === 'default'} onClick={() => nav('agents')} />)}
        </div>
      </div>
      <div className={styles.section}>
        <div className={styles.label}>Browse</div>
        <div className={styles.grid}>
          {MORE.map((m) => (
            <div key={m.id} className={styles.card} onClick={() => nav(m.id)}>
              <div className={styles.cardIcon}>{m.icon}</div>
              <div className={styles.cardLabel}>{m.label}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 5: MobileShell**

Create `apps/IrsClawTauri/src/components/shell/MobileShell.tsx`:
```tsx
import type { ReactNode } from 'react'
import { Menu, Plus } from 'lucide-react'
import { useLocation } from 'wouter'
import { useUiStore, type Tab } from '../../store/ui'
import { useBackendStore } from '../../store/backend'
import { createSession } from '../../api/sessions'
import BottomTabBar from './BottomTabBar'
import Drawer from './Drawer'
import styles from './MobileShell.module.css'
import '../../styles/responsive.css'

interface Props { children: ReactNode }

export default function MobileShell({ children }: Props) {
  const [, setLocation] = useLocation()
  const { mobileDrawerOpen, setMobileDrawerOpen, setSelectedTab } = useUiStore()
  const connectionState = useBackendStore((s) => s.connectionState)

  const navigate = (t: Tab) => { setSelectedTab(t); setLocation(t === 'chat' ? '/' : `/${t}`) }

  const handleNew = async () => {
    if (connectionState !== 'connected') return
    await createSession()
    navigate('chat')
  }

  return (
    <div className={styles.shell}>
      <header className={styles.topbar}>
        <button className={styles.btn} onClick={() => setMobileDrawerOpen(true)}><Menu size={20} /></button>
        <span className={styles.title}>i-rs-claw</span>
        <button className={styles.btn} onClick={handleNew} disabled={connectionState !== 'connected'}><Plus size={20} /></button>
      </header>
      <div className={styles.body}>{children}</div>
      <BottomTabBar onNavigate={navigate} />
      {mobileDrawerOpen && (
        <>
          <div className={styles.overlay} onClick={() => setMobileDrawerOpen(false)} />
          <Drawer onClose={() => setMobileDrawerOpen(false)} onNavigate={navigate} />
        </>
      )}
    </div>
  )
}
```

- [ ] **Step 6: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/components/shell/DesktopShell.tsx apps/IrsClawTauri/src/components/shell/MobileShell.* apps/IrsClawTauri/src/components/shell/BottomTabBar.tsx apps/IrsClawTauri/src/components/shell/Drawer.*
git commit -m "feat(IrsClawTauri): add DesktopShell + MobileShell + BottomTabBar + Drawer"
```

---

## Task 16: Chat page

**Files:**
- Create: `apps/IrsClawTauri/src/pages/Chat.tsx` + `.module.css`

- [ ] **Step 1: Chat page CSS**

Create `apps/IrsClawTauri/src/pages/Chat.module.css`:
```css
.page { display: flex; flex-direction: column; height: 100%; }
.header { padding: 12px 20px; border-bottom: 1px solid var(--border); display: flex; align-items: center; justify-content: space-between; background: var(--bg-secondary); min-height: 56px; }
.left { display: flex; align-items: center; gap: 10px; min-width: 0; }
.title { font-size: 15px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
```

- [ ] **Step 2: Chat page**

Create `apps/IrsClawTauri/src/pages/Chat.tsx`. Adapt `crates/claw/dashboard-ui/src/pages/Chat.tsx` — same SSE handler structure (handleCommit/handleImageGenerated/handleError/handleDone/handleEvaluation/handleQualityScore), same `useChatStream` + `useSessionLoader` wiring, same `handleSend`/`handleNewChat`/`handleFeedback`. Replace the page header + ChatInput with `styles.header` + `FloatingInput`, and the messages area with `MessageList` + `EmptyState`. Use the code in the spec's Task 16 as the template (the full Chat.tsx was written in the spec design).

Key differences from dashboard-ui Chat.tsx:
- Import `FloatingInput` instead of `ChatInput`
- Import `MessageList` and `EmptyState` from local components
- Use `<FloatingInput onSend={handleSend} disabled={streaming} />` (no mic in first phase — pass no `onMic` prop)
- Wrap in `<div className={styles.page}>` with `styles.header` top bar

- [ ] **Step 3: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/pages/Chat.*
git commit -m "feat(IrsClawTauri): add Chat page with SSE streaming"
```

---

## Task 17: Sessions, Usage, Settings, Agents, Placeholder pages

**Files:**
- Create: `apps/IrsClawTauri/src/pages/Sessions.tsx`
- Create: `apps/IrsClawTauri/src/pages/Usage.tsx`
- Create: `apps/IrsClawTauri/src/pages/Settings.tsx` + `.module.css`
- Create: `apps/IrsClawTauri/src/pages/Agents.tsx`
- Create: `apps/IrsClawTauri/src/pages/Placeholder.tsx`

- [ ] **Step 1: Sessions page**

Create `apps/IrsClawTauri/src/pages/Sessions.tsx`. List sessions with `listSessions`, create/switch/delete via `api/sessions`. Use `.session-card` / `.sessions-list` classes from theme.css. Include loading spinner, empty state with "New Chat" button, and per-card switch + delete actions.

- [ ] **Step 2: Usage page**

Create `apps/IrsClawTauri/src/pages/Usage.tsx`. Fetch `getStats(period)` with period selector (today/week/month/all). Use `.period-selector`, `.usage-cards`, `.usage-card`, `.today-stats` classes from theme.css. Show total requests/tokens/cost + today breakdown.

- [ ] **Step 3: Settings page CSS + component**

Create `apps/IrsClawTauri/src/pages/Settings.module.css` with `.page`, `.title`, `.section`, `.sectionHeader`, `.sectionBody`, `.field`, `.label`, `.input`, `.hint`, `.row`, `.toggle` classes (see spec Task 17 Step 3-4 for full CSS).

Create `apps/IrsClawTauri/src/pages/Settings.tsx` — three sections: Backend Connection (URL + token inputs, Save button calls `setBaseUrl`/`setToken`), Appearance (dark mode toggle via `useUiStore`), System (autostart toggle calling `invoke('set_autostart')` / `invoke('is_autostart_enabled')`). Import `toast` for feedback.

- [ ] **Step 4: Agents page**

Create `apps/IrsClawTauri/src/pages/Agents.tsx`. List agents with `listAgents`, create (id + model form) with `createAgent`, delete with `deleteAgent`. Show each agent as a `.card` with id/provider/model/tool_count. Hide delete for `default` and sub-agents.

- [ ] **Step 5: Placeholder page**

Create `apps/IrsClawTauri/src/pages/Placeholder.tsx`:
```tsx
import type { ReactNode } from 'react'
import EmptyState from '../components/common/EmptyState'

export default function PlaceholderPage({ icon, title }: { icon: ReactNode; title: string }) {
  return <EmptyState icon={icon} title={title} description="This page is coming soon. Use the dashboard-ui web interface for full management in the meantime." />
}
```

- [ ] **Step 6: Typecheck + commit**
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
git add apps/IrsClawTauri/src/pages/Sessions.tsx apps/IrsClawTauri/src/pages/Usage.tsx apps/IrsClawTauri/src/pages/Settings.tsx apps/IrsClawTauri/src/pages/Settings.module.css apps/IrsClawTauri/src/pages/Agents.tsx apps/IrsClawTauri/src/pages/Placeholder.tsx
git commit -m "feat(IrsClawTauri): add Sessions/Usage/Settings/Agents/Placeholder pages"
```

---

## Task 18: App.tsx + main.tsx wiring

**Files:**
- Modify: `apps/IrsClawTauri/src/App.tsx`
- Modify: `apps/IrsClawTauri/src/main.tsx`
- Delete: `apps/IrsClawTauri/src/App.css`, `apps/IrsClawTauri/src/assets/`

- [ ] **Step 1: Replace App.tsx**

Create `apps/IrsClawTauri/src/App.tsx`:
```tsx
import { useState } from 'react'
import { Route, Switch } from 'wouter'
import { Wrench, BookOpen, Puzzle } from 'lucide-react'
import { useIsDesktop } from './hooks/useMediaQuery'
import { useTheme } from './hooks/useTheme'
import { useBackendHealth } from './hooks/useBackendHealth'
import { useBackendStore } from './store/backend'
import DesktopShell from './components/shell/DesktopShell'
import MobileShell from './components/shell/MobileShell'
import ConnectScreen from './components/shell/ConnectScreen'
import { ToastContainer } from './components/common/Toast'
import ChatPage from './pages/Chat'
import SessionsPage from './pages/Sessions'
import AgentsPage from './pages/Agents'
import UsagePage from './pages/Usage'
import SettingsPage from './pages/Settings'
import PlaceholderPage from './pages/Placeholder'

export default function App() {
  useTheme()
  useBackendHealth()
  const isDesktop = useIsDesktop()
  const connectionState = useBackendStore((s) => s.connectionState)
  const [selectedAgent] = useState('default')

  if (connectionState !== 'connected') {
    return (<><ConnectScreen /><ToastContainer /></>)
  }

  const Shell = isDesktop ? DesktopShell : MobileShell

  return (
    <>
      <Shell>
        <Switch>
          <Route path="/"><ChatPage selectedAgent={selectedAgent} /></Route>
          <Route path="/sessions"><SessionsPage /></Route>
          <Route path="/agents"><AgentsPage /></Route>
          <Route path="/usage"><UsagePage /></Route>
          <Route path="/settings"><SettingsPage /></Route>
          <Route path="/tools"><PlaceholderPage icon={<Wrench size={28} />} title="Tools" /></Route>
          <Route path="/skills"><PlaceholderPage icon={<BookOpen size={28} />} title="Skills" /></Route>
          <Route path="/plugins"><PlaceholderPage icon={<Puzzle size={28} />} title="Plugins" /></Route>
        </Switch>
      </Shell>
      <ToastContainer />
    </>
  )
}
```

- [ ] **Step 2: Replace main.tsx**

Create `apps/IrsClawTauri/src/main.tsx`:
```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import './styles/theme.css'
import './styles/responsive.css'

createRoot(document.getElementById('root') as HTMLElement).render(
  <StrictMode><App /></StrictMode>,
)
```

- [ ] **Step 3: Remove scaffold leftovers**
```bash
rm -f apps/IrsClawTauri/src/App.css
rm -rf apps/IrsClawTauri/src/assets
```

- [ ] **Step 4: Typecheck + build**

Run: `pnpm --filter irsclawtauri exec tsc --noEmit`
Expected: 0 errors. Fix any unused-import errors.

Run: `pnpm --filter irsclawtauri build`
Expected: Vite builds `dist/` successfully.

- [ ] **Step 5: Commit**
```bash
git add apps/IrsClawTauri/src/App.tsx apps/IrsClawTauri/src/main.tsx
git rm -r apps/IrsClawTauri/src/App.css apps/IrsClawTauri/src/assets 2>/dev/null || true
git commit -m "feat(IrsClawTauri): wire App root (shell switch + router + connect gate)"
```

---

## Task 19: Rust commands

**Files:**
- Create: `apps/IrsClawTauri/src-tauri/src/commands.rs`

- [ ] **Step 1: Write commands module**

Create `apps/IrsClawTauri/src-tauri/src/commands.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, Manager};

const URL_FILE: &str = "backend-url";
const TOKEN_FILE: &str = "backend-token";

fn app_dir(app: &AppHandle) -> std::path::PathBuf {
    app.path().app_config_dir().expect("app config dir")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[tauri::command]
pub fn get_backend_url(app: AppHandle) -> String {
    fs::read_to_string(app_dir(&app).join(URL_FILE)).unwrap_or_default()
}

#[tauri::command]
pub fn set_backend_url(app: AppHandle, url: String) -> Result<(), String> {
    let dir = app_dir(&app);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(URL_FILE), url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_token(app: AppHandle) -> String {
    fs::read_to_string(app_dir(&app).join(TOKEN_FILE)).unwrap_or_default()
}

#[tauri::command]
pub fn set_token(app: AppHandle, token: String) -> Result<(), String> {
    let dir = app_dir(&app);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(TOKEN_FILE), token).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_backend_health(url: String) -> Result<bool, String> {
    let target = format!("{}/api/health", url.trim_end_matches('/'));
    match reqwest::get(&target).await {
        Ok(r) => Ok(r.status().is_success()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn open_file_dialog(app: AppHandle, filters: Option<Vec<Filter>>) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let mut builder = app.dialog().file();
    if let Some(filters) = filters {
        for f in filters {
            builder = builder.add_filter(f.name, f.extensions.as_slice());
        }
    }
    match builder.blocking_pick_file() {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
    let cmd = if cfg!(target_os = "macos") { "open" } else if cfg!(target_os = "windows") { "explorer" } else { "xdg-open" };
    std::env::Command::new(cmd).arg(&path).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let autostart = app.state::<tauri_plugin_autostart::AutoLaunchManager>();
    if enabled { autostart.enable().map_err(|e| e.to_string()) }
    else { autostart.disable().map_err(|e| e.to_string()) }
}

#[tauri::command]
pub fn is_autostart_enabled(app: AppHandle) -> bool {
    let autostart = app.state::<tauri_plugin_autostart::AutoLaunchManager>();
    autostart.is_enabled().unwrap_or(false)
}
```

- [ ] **Step 2: Commit (compiles after Task 21 wiring)**
```bash
git add apps/IrsClawTauri/src-tauri/src/commands.rs
git commit -m "feat(IrsClawTauri): add Rust commands module"
```

---

## Task 20: System tray

**Files:**
- Create: `apps/IrsClawTauri/src-tauri/src/tray.rs`

- [ ] **Step 1: Write tray module**

Create `apps/IrsClawTauri/src-tauri/src/tray.rs`:
```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let new_chat = MenuItem::with_id(app, "new_chat", "New Chat", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&new_chat, &show, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().expect("window icon").clone())
        .tooltip("i-rs-claw")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => app.exit(0),
            "new_chat" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.emit("tray-new-chat", ());
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}
```

- [ ] **Step 2: Commit**
```bash
git add apps/IrsClawTauri/src-tauri/src/tray.rs
git commit -m "feat(IrsClawTauri): add system tray with menu"
```

---

## Task 21: lib.rs wiring + tauri.conf.json + capabilities

**Files:**
- Modify: `apps/IrsClawTauri/src-tauri/src/lib.rs`
- Modify: `apps/IrsClawTauri/src-tauri/tauri.conf.json`
- Modify: `apps/IrsClawTauri/src-tauri/capabilities/default.json`

- [ ] **Step 1: Replace lib.rs**

Create `apps/IrsClawTauri/src-tauri/src/lib.rs`:
```rust
mod commands;
mod tray;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::{AutoLaunchManager, MacosLauncher};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .manage(AutoLaunchManager::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_backend_url,
            commands::set_backend_url,
            commands::get_token,
            commands::set_token,
            commands::check_backend_health,
            commands::open_file_dialog,
            commands::show_in_folder,
            commands::set_autostart,
            commands::is_autostart_enabled,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                #[cfg(desktop)]
                {
                    if window.label() == "main" {
                        let _ = window.hide();
                        api.prevent_close();
                    }
                }
                let _ = window;
            }
        });

    builder
        .setup(|app| {
            #[cfg(desktop)]
            {
                tray::setup_tray(app.handle())?;
            }
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Note: `AutoLaunchManager::new()` — check the tauri-plugin-autostart API. If the plugin's `init()` already manages state, remove the `.manage(AutoLaunchManager::new())` line. Verify against the plugin's docs at `node_modules/@tauri-apps/plugin-autostern` or the Cargo docs. If unsure, keep `.plugin(...autostart::init(...))` only and remove `.manage(...)`.

- [ ] **Step 2: Update tauri.conf.json**

Edit `apps/IrsClawTauri/src-tauri/tauri.conf.json` — update `productName` to `i-rs-claw`, window title to `i-rs-claw`, size to 1100x720, and add deep-link config. Replace the `app` block:
```json
{
  "app": {
    "windows": [
      { "title": "i-rs-claw", "width": 1100, "height": 720, "minWidth": 768, "minHeight": 600 }
    ],
    "security": { "csp": null }
  }
}
```
Add a top-level deep-link block:
```json
{
  "plugins": {
    "deep-link": {
      "desktop": { "schemes": ["irsclaw"] }
    }
  }
}
```

- [ ] **Step 3: Update capabilities/default.json**

Replace `apps/IrsClawTauri/src-tauri/capabilities/default.json`:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "notification:default",
    "dialog:default",
    "clipboard-manager:default",
    "deep-link:default",
    "autostart:default"
  ]
}
```

- [ ] **Step 4: Compile Rust**

Run: `cargo check -p irsclawtauri`
Expected: 0 errors. If autostart plugin API differs, adjust `.manage(...)` / `init(...)` call per the compiler's suggestion. The `show_in_folder` command may have an unused `OpenerExt` warning — ensure no unused imports.

- [ ] **Step 5: Commit**
```bash
git add apps/IrsClawTauri/src-tauri/src/lib.rs apps/IrsClawTauri/src-tauri/tauri.conf.json apps/IrsClawTauri/src-tauri/capabilities/default.json
git commit -m "feat(IrsClawTauri): wire lib.rs (plugins + commands + tray) + deep-link config"
```

---

## Task 22: Final verification

**Files:** none (verification only)

- [ ] **Step 1: Frontend typecheck + tests + build**

Run:
```bash
pnpm --filter irsclawtauri exec tsc --noEmit
pnpm --filter irsclawtauri test
pnpm --filter irsclawtauri build
```
Expected: tsc 0 errors; vitest passes; vite builds `dist/`.

- [ ] **Step 2: Rust check**

Run: `cargo check -p irsclawtauri`
Expected: 0 errors, 0 warnings.

- [ ] **Step 3: Desktop dev launch**

Run: `pnpm --filter irsclawtauri tauri dev`
Expected: app window opens showing ConnectScreen (no backend running). Enter `http://localhost:3000` + any token → shows "failed to connect" (expected if no claw serve running). Start `claw serve` in another terminal → app connects → shows Chat page with empty state.

- [ ] **Step 4: Mobile layout check**

With the dev app running, resize the window narrower than 768px. Expected: layout switches to MobileShell (top bar + bottom tab bar). Click ☰ → drawer slides in. Click More → navigates to agents.

- [ ] **Step 5: Commit final state**
```bash
git add -A
git commit -m "feat(IrsClawTauri): complete first-phase client (core loop + native shell)" --allow-empty
```

---

## Verification Summary

After all tasks complete, verify against the spec:
- [x] Backend connection: ConnectScreen → health check → connected (Task 13, 18, 8)
- [x] Chat: SSE streaming + tool calls + feedback (Task 9, 11, 12, 16)
- [x] Sessions: list/create/switch/delete (Task 7, 17)
- [x] Agents: list/create/delete (Task 7, 17)
- [x] Usage: stats with period selector (Task 7, 17)
- [x] Settings: backend URL/token/theme/autostart (Task 17)
- [x] Native shell: tray + dialog + autostart commands (Task 19, 20, 21)
- [x] Responsive: desktop sidebar / mobile tab+drawer (Task 14, 15, 18)
- [x] Theme: CSS variables + data-theme (Task 3, 6)
- [x] Placeholder pages: Tools/Skills/Plugins (Task 17, 18)
