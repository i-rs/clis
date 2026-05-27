# IrsClawMiniProgram Redesign

Date: 2026-05-26

## Overview

Complete redesign of the IrsClawMiniProgram — a WeChat Mini Program companion for the `i-rs-claw` TUI AI assistant. The current version has inconsistent theming (dark/light mixed), copy-pasted list pages, and generic visual quality. This redesign standardizes on WeChat design principles for a polished, native-feeling experience.

## Design Principles

1. **微信原生感** — Follow WeChat Mini Program design conventions: light background (#EDEDED), white cards, green brand color (#07C160), standard list styles
2. **一致性** — All pages share the same design system; no random dark/light mixing
3. **内容优先** — Chat is the hero; all secondary pages are accessed via sidebar drawer
4. **轻量流畅** — Minimal chrome, fast transitions, no unnecessary decoration

## Architecture

### Page Structure

```
App (app.js / app.json / app.wxss)
├── pages/index/          ← Chat (MAIN, entry page)
│   ├── Sidebar Drawer    ← Overlay menu (sessions/tools/skills/plugins/agents/settings)
│   └── Message List + Input Bar
├── pages/sessions/       ← Conversation history
├── pages/tools/          ← Tool registry
├── pages/skills/         ← Skill registry
├── pages/plugins/        ← Plugin registry
├── pages/agents/         ← Agent selection
└── pages/settings/       ← Server config
```

All secondary pages (sessions/tools/skills/plugins/agents/settings) are navigated to via `wx.navigateTo` from the sidebar drawer.

## Design System

### Colors

| Token | Value | Usage |
|-------|-------|-------|
| `--page-bg` | `#EDEDED` | Page background |
| `--card-bg` | `#FFFFFF` | Card / list item background |
| `--brand` | `#07C160` | Primary action, send button, active states |
| `--brand-press` | `#06AD56` | Press state for brand |
| `--text-primary` | `#000000` / `#1A1A1A` | Primary text |
| `--text-secondary` | `#666666` | Secondary / description text |
| `--text-tertiary` | `#999999` | Placeholder / timestamp |
| `--divider` | `#EBEBEB` | Hairline separators |
| `--mask` | `rgba(0,0,0,0.5)` | Drawer / modal overlay |
| `--user-bubble` | `#07C160` | User message bubble |
| `--user-bubble-text` | `#FFFFFF` | User message text |
| `--ai-bubble` | `#FFFFFF` | AI message bubble |
| `--ai-bubble-border` | `#E5E5E5` | AI bubble border |
| `--danger` | `#FA5151` | Delete actions |

### Typography

WeChat uses the system font stack by default. No custom fonts needed.

| Level | Size | Weight | Color |
|-------|------|--------|-------|
| Navbar title | 18px | 600 | Primary |
| Section header | 14px | 400 | Tertiary |
| List item title | 16px | 400 | Primary |
| List item subtitle | 14px | 400 | Secondary |
| Message bubble text | 16px | 400 | Primary / White |
| Timestamp | 12px | 400 | Tertiary |
| Input text | 17px | 400 | Primary |

### Spacing

- Page padding: 0 (cards edge-to-edge, standard WeChat list style)
- Card padding: 16px left/right, 12px top/bottom
- Between items: 0 (standard list with bottom border)
- Between sections: 10px (standard WeChat section gap)
- Bubble padding: 12px 16px
- Input area height: 52px (standard WeChat input bar)

### Border Radius

| Component | Radius |
|-----------|--------|
| Avatar | 50% (circle) |
| Message bubble | 10px |
| Cards | 0px (standard list) |
| Input bar | 8px |
| Action button | 6px |

## Components

### 1. NavBar

WeChat-style navigation bar. Custom component via `navigationStyle: custom` for the chat page.

```
[☰]          [Agent Name]          [+]
```

- Left: hamburger menu button (opens sidebar drawer)
- Center: current agent name (tap to show agent switcher dropdown?)
- Right: new chat button (green `+` icon)

### 2. Sidebar Drawer

Sliding panel from left edge, 75% screen width, with overlay mask.

**Menu groups:**

```
IrsClaw                    ← App name / branding header
─────────────────────
💬  Session History        → wx.navigateTo pages/sessions
🔧  Tools                  → wx.navigateTo pages/tools
📚  Skills                 → wx.navigateTo pages/skills
🧩  Plugins                → wx.navigateTo pages/plugins
─────────────────────
👤  Agent                  → wx.navigateTo pages/agents
⚙️  Settings               → wx.navigateTo pages/settings
```

- Icons: Use clean SVG-like Unicode characters or simple emoji
- Close: tap mask or swipe right-to-left

### 3. Message Bubbles

```
User (right-aligned):
┌──────────────────────┐
│  Message content     │
│  here                │
│             12:30 PM │
└──────────────────────┘
        [circle avatar]

AI (left-aligned):
  [circle avatar]
┌──────────────────────┐
│  Message content     │
│  here                │
│             12:30 PM │
└──────────────────────┘
```

- User avatar: initials or simple icon, #07C160 green background
- AI avatar: WeChat-style robot icon or "AI" text, #666 gray background
- Timestamp: bottom-right of bubble, tertiary color, 12px
- Consecutive messages from same sender: hide avatar, tighter spacing

**Collapsible reasoning block (in AI messages):**

```
┌──────────────────────┐
│ 🤔 Thinking...  ▶    │  ← Orange header, tappable
│ (collapsed)          │
└──────────────────────┘
```

When expanded:
```
┌──────────────────────┐
│ 🤔 Thinking...  ▼    │
│                      │
│ Full reasoning text  │
│ in lighter gray bg   │
│                      │
└──────────────────────┘
```

**Collapsible tool call block (in AI messages):**

```
┌──────────────────────┐
│ 🔧 Tool: get_weather │
│ ▶  Show arguments    │  ← Purple header
└──────────────────────┘
```

### 4. Input Bar

Fixed at bottom, standard WeChat style:

```
┌─────────────────────────────────┐
│ [Text Input (flex: 1)]  [Send] │
└─────────────────────────────────┘
```

- Input: 17px font, placeholder text, auto-height (1-4 lines), rounded bg
- Send button: Green circle with white arrow icon, disabled (gray) when empty
- Safe area bottom padding for iPhone notch

### 5. List Items (Unified Standard)

Used across sessions, tools, skills, plugins, agents:

```
┌─────────────────────────────────────┐
│ [icon]  Title                        │
│          Description / subtitle      │
│                         [action]     │
├─────────────────────────────────────┤  ← hairline divider
```

- Icon: 44x44px rounded square container with gradient/hue background
- Title: 16px, primary
- Subtitle: 14px, secondary
- Action: `>` arrow for navigation, or custom (checkmark for agents, delete for sessions)
- Full row tappable with press highlight

### 6. Empty States

Centered layout:

```
        [icon 64px]
    [Title 18px bold]
  [Subtitle 14px gray]
```

### 7. Settings Page

Grouped form with section headers:

```
CONNECTION
┌─────────────────────────────┐
│ Status: ● Connected         │
│ Server: http://...          │
├─────────────────────────────┤
│ Server URL                  │
│ [input field]               │
├─────────────────────────────┤
│ Auth Token                  │
│ [input field]               │
├─────────────────────────────┤
│ [Save & Reconnect]          │
└─────────────────────────────┘

ABOUT
┌─────────────────────────────┐
│ Provider: openai            │
│ Model: gpt-4o-mini          │
└─────────────────────────────┘
```

## Data Flow

### Chat Flow
1. User types message → taps send
2. `app.js` calls `POST /api/chat { content, agentId, sessionId }`
3. On response: fetch full session via `GET /api/sessions/:id`
4. Update message list
5. Auto-scroll to bottom

### Session Management
- Auto-load latest session on connect
- New chat button → `POST /api/sessions` → switch to new session
- Sessions page → tap session → `wx.navigateBack` with session ID

### Theme
- Fixed light theme throughout. No dark mode switching.
- All pages use `#EDEDED` background with white cards.

## Implementation Plan

### Phase 1: Foundation
- Overwrite `app.wxss` with new design tokens
- Rewrite `app.js` to keep API client but clean up global data
- Rewrite `pages/index/` (the chat page) with new layout

### Phase 2: Sidebar Drawer
- Implement sidebar as an overlay component within `pages/index/`
- Wire up navigation to all secondary pages

### Phase 3: List Pages
- Rewrite `pages/sessions/` with new unified list style
- Rewrite `pages/tools/`, `pages/skills/`, `pages/plugins/` (unified list component)
- Rewrite `pages/agents/` with checkmark selection
- Rewrite `pages/settings/` with grouped form

### Phase 4: Polish
- Animations: message fade-in, drawer slide, list tap feedback
- Safe area handling for all devices
- Loading states for API calls
- Scroll-to-bottom on new messages
