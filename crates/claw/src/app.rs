pub use i_rs_claw_core::app::*;

use i_rs_claw_core::config::Config;
use i_rs_claw_core::stats::TodaySummary;
use crate::ui::chat_api::{ClickRegionRegistry, ComponentCell, ComponentOp, Scroller, build_component_for};
use chrono::NaiveDateTime;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::hash::Hash;
use std::rc::Rc;
use std::time::Instant;


#[derive(Clone)]
pub struct InputState {
    pub text: String,
    pub cursor: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    last_change: Option<Instant>,
    pub(crate) draft: String,
}

pub const MAX_INPUT_LEN: usize = 64 * 1024;

impl InputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_change: None,
            draft: String::new(),
        }
    }

    /// 当前剩余可写入字节数。
    pub fn remaining_capacity(&self) -> usize {
        MAX_INPUT_LEN.saturating_sub(self.text.len())
    }

    /// 若 `text.len()` 超过 `MAX_INPUT_LEN`，按字符边界在 cursor 处截断，
    /// 返回被丢弃的字节数（0 表示完整写入）。
    pub fn insert_text_at_cursor(&mut self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        self.push_undo(Instant::now());
        let cap = self.remaining_capacity();
        let to_insert = if text.len() <= cap {
            text
        } else {
            let mut end = cap;
            while end > 0 && !text.is_char_boundary(end) {
                end -= 1;
            }
            &text[..end]
        };
        let inserted_bytes = to_insert.len();
        let dropped = text.len() - inserted_bytes;
        if inserted_bytes == 0 {
            return dropped;
        }
        self.text.insert_str(self.cursor, to_insert);
        self.cursor += inserted_bytes;
        dropped
    }

    /// Push current text to undo stack. Coalesces with the previous push
    /// if less than 500ms have elapsed (logical-operation undo).
    pub fn push_undo(&mut self, now: Instant) {
        let coalesce = self
            .last_change
            .map(|t| now.duration_since(t).as_millis() < 500)
            .unwrap_or(false);
        if coalesce {
            self.undo_stack.pop();
        }
        self.undo_stack.push(self.text.clone());
        if self.undo_stack.len() > 100 {
            self.undo_stack.drain(..50);
        }
        self.redo_stack.clear();
        self.last_change = Some(now);
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.text.clone());
            self.text = prev;
            self.cursor = self.cursor.min(self.text.len());
            while self.cursor > 0 && !self.text.is_char_boundary(self.cursor) {
                self.cursor -= 1;
            }
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.text.clone());
            self.text = next;
            self.cursor = self.cursor.min(self.text.len());
            while self.cursor > 0 && !self.text.is_char_boundary(self.cursor) {
                self.cursor -= 1;
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if c.len_utf8() > self.remaining_capacity() {
            return;
        }
        self.push_undo(Instant::now());
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn delete_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.push_undo(Instant::now());
        let prev = self.text[..self.cursor].char_indices().next_back();
        if let Some((idx, _)) = prev {
            self.text.drain(idx..self.cursor);
            self.cursor = idx;
        }
    }

    pub fn delete_word_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.push_undo(Instant::now());
        let before = &self.text[..self.cursor];
        let trimmed = before.trim_end_matches(|c: char| c.is_whitespace());
        let word_start = if trimmed.is_empty() {
            0
        } else {
            let chars: Vec<(usize, char)> = trimmed.char_indices().collect();
            let mut i = chars.len();
            while i > 0 && (chars[i - 1].1.is_alphanumeric() || chars[i - 1].1 == '_') {
                i -= 1;
            }
            if i < chars.len() {
                chars[i].0
            } else {
                trimmed.len()
            }
        };
        self.text.drain(word_start..self.cursor);
        self.cursor = word_start;
    }

    pub fn delete_to_line_start(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.push_undo(Instant::now());
        let line_start = self.text[..self.cursor]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        self.text.drain(line_start..self.cursor);
        self.cursor = line_start;
    }

    pub fn delete_to_line_end(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        self.push_undo(Instant::now());
        let line_end = self.text[self.cursor..]
            .find('\n')
            .map(|i| self.cursor + i)
            .unwrap_or(self.text.len());
        self.text.drain(self.cursor..line_end);
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let prev = self.text[..self.cursor].char_indices().next_back();
        if let Some((idx, _)) = prev {
            self.cursor = idx;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let next = self.text[self.cursor..].char_indices().nth(1);
        if let Some((offset, _)) = next {
            self.cursor += offset;
        } else {
            self.cursor = self.text.len();
        }
    }

    pub fn move_cursor_word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let before = &self.text[..self.cursor];
        let trimmed = before.trim_end_matches(|c: char| c.is_whitespace());
        if trimmed.is_empty() {
            self.cursor = 0;
            return;
        }
        let chars: Vec<(usize, char)> = trimmed.char_indices().collect();
        let mut i = chars.len();
        while i > 0 && (chars[i - 1].1.is_alphanumeric() || chars[i - 1].1 == '_') {
            i -= 1;
        }
        self.cursor = if i < chars.len() {
            chars[i].0
        } else {
            trimmed.len()
        };
    }

    pub fn move_cursor_word_right(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let after = &self.text[self.cursor..];
        let trimmed = after.trim_start_matches(|c: char| c.is_whitespace());
        if trimmed.is_empty() {
            self.cursor = self.text.len();
            return;
        }
        let skip_ws = after.len() - trimmed.len();
        let word_end = trimmed
            .char_indices()
            .position(|(_, c)| !c.is_alphanumeric() && c != '_')
            .unwrap_or(trimmed.len());
        self.cursor = self.cursor + skip_ws + word_end;
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.text.len();
    }

    pub fn commit_to_history(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.history.last().map(|s| s.as_str()) != Some(text) {
            self.history.push(text.to_string());
            if self.history.len() > 50 {
                self.history.drain(0..1);
            }
        }
        self.history_index = None;
    }

    pub fn navigate_up(&mut self) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }
        let new_idx = match self.history_index {
            Some(i) if i > 0 => i - 1,
            None => self.history.len().saturating_sub(1),
            _ => return None,
        };
        self.history_index = Some(new_idx);
        Some(self.history[new_idx].clone())
    }

    pub fn navigate_down(&mut self) -> Option<String> {
        match self.history_index {
            Some(i) if i + 1 < self.history.len() => {
                self.history_index = Some(i + 1);
                Some(self.history[i + 1].clone())
            }
            Some(_) => {
                self.history_index = None;
                None
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashAction {
    Help,
    Sessions,
    New,
    Agent,
    Agents,
    Tools,
    Sidebar,
    Stats,
    Plugins,
    Config,
    Export,
    Feedback,
    Info,
    Select,
    Clear,
    Compact,
    Theme,
}

pub struct SlashCommand {
    pub name: &'static str,
    pub desc: &'static str,
    pub shortcut: &'static str,
    pub action: SlashAction,
}

pub static SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        name: "/help",
        desc: "快捷键帮助",
        shortcut: "Ctrl+H",
        action: SlashAction::Help,
    },
    SlashCommand {
        name: "/sessions",
        desc: "会话列表",
        shortcut: "Ctrl+L",
        action: SlashAction::Sessions,
    },
    SlashCommand {
        name: "/new",
        desc: "新建会话",
        shortcut: "Ctrl+N",
        action: SlashAction::New,
    },
    SlashCommand {
        name: "/agent",
        desc: "切换 Agent",
        shortcut: "Ctrl+P",
        action: SlashAction::Agent,
    },
    SlashCommand {
        name: "/agents",
        desc: "Agent 管理",
        shortcut: "Ctrl+A",
        action: SlashAction::Agents,
    },
    SlashCommand {
        name: "/tools",
        desc: "工具列表",
        shortcut: "Ctrl+T",
        action: SlashAction::Tools,
    },
    SlashCommand {
        name: "/sidebar",
        desc: "HTTP 调试面板",
        shortcut: "Ctrl+R",
        action: SlashAction::Sidebar,
    },
    SlashCommand {
        name: "/stats",
        desc: "Token 用量统计",
        shortcut: "Ctrl+Shift+U",
        action: SlashAction::Stats,
    },
    SlashCommand {
        name: "/plugins",
        desc: "插件 & 技能",
        shortcut: "Ctrl+Shift+P",
        action: SlashAction::Plugins,
    },
    SlashCommand {
        name: "/config",
        desc: "配置信息",
        shortcut: "Ctrl+I",
        action: SlashAction::Config,
    },
    SlashCommand {
        name: "/export",
        desc: "导出会话",
        shortcut: "Ctrl+E",
        action: SlashAction::Export,
    },
    SlashCommand {
        name: "/feedback",
        desc: "发送反馈",
        shortcut: "Ctrl+F",
        action: SlashAction::Feedback,
    },
    SlashCommand {
        name: "/info",
        desc: "状态仪表盘",
        shortcut: "Ctrl+Shift+I",
        action: SlashAction::Info,
    },
    SlashCommand {
        name: "/select",
        desc: "选择模式",
        shortcut: "Ctrl+S",
        action: SlashAction::Select,
    },
    SlashCommand {
        name: "/clear",
        desc: "清空当前会话",
        shortcut: "",
        action: SlashAction::Clear,
    },
    SlashCommand {
        name: "/compact",
        desc: "压缩上下文",
        shortcut: "",
        action: SlashAction::Compact,
    },
    SlashCommand {
        name: "/theme",
        desc: "切换主题配色",
        shortcut: "",
        action: SlashAction::Theme,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    SessionList,
    Sidebar,
    AgentPicker,
    Help,
    Config,
    Feedback,
    ToolList,
    AgentList,
    StatsHistory,
    PluginList,
    InfoPanel,
    ThemePicker,
}

pub struct OverlayState {
    pub current: Option<Overlay>,
    pub selection_mode: bool,
    pub selected_message: Option<usize>,
    pub session_list_index: usize,
    pub session_list: Vec<i_rs_claw_core::session::SessionMeta>,
    pub session_search: String,
    pub session_search_mode: bool,
    pub session_rename_buf: String,
    pub session_confirm_delete: bool,
    pub sidebar_selected: usize,
    pub sidebar_body_idx: Option<usize>,
    pub sidebar_body_scroll: usize,
    pub sidebar_formatted_json: Option<String>,
    pub agent_picker_index: usize,
    pub agent_list: Vec<String>,
    pub copy_feedback: Option<(String, std::time::Instant)>,
    pub tab_completions: Vec<String>,
    pub tab_completion_index: usize,
    pub tab_completion_prefix: String,
    pub tab_completion_cursor: usize,
    pub slash_visible: bool,
    pub slash_index: usize,
    pub theme_index: usize,
    pub cached_filtered_sessions: Option<Vec<i_rs_claw_core::session::SessionMeta>>,
    pub cached_search_hash: u64,
}

impl OverlayState {
    pub fn filtered_sessions(&self) -> Vec<i_rs_claw_core::session::SessionMeta> {
        let q = self.session_search.to_lowercase();
        if q.is_empty() {
            self.session_list.clone()
        } else {
            self.session_list
                .iter()
                .filter(|s| s.title.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
    }

    pub fn filtered_sessions_cached(&mut self) -> Vec<i_rs_claw_core::session::SessionMeta> {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.session_search.hash(&mut hasher);
        self.session_list.len().hash(&mut hasher);
        for s in &self.session_list {
            s.title.hash(&mut hasher);
        }
        let hash = hasher.finish();
        if self.cached_search_hash == hash
            && let Some(ref cached) = self.cached_filtered_sessions
        {
            return cached.clone();
        }
        let filtered = self.filtered_sessions();
        self.cached_search_hash = hash;
        self.cached_filtered_sessions = Some(filtered.clone());
        filtered
    }

    pub fn get_filtered_sessions(&self) -> &[i_rs_claw_core::session::SessionMeta] {
        if let Some(ref cached) = self.cached_filtered_sessions {
            cached
        } else if self.session_search.is_empty() {
            &self.session_list
        } else {
            &[]
        }
    }

    pub fn invalidate_session_cache(&mut self) {
        self.cached_filtered_sessions = None;
        self.cached_search_hash = 0;
    }

    pub fn new(agent_list: Vec<String>) -> Self {
        Self {
            current: None,
            selection_mode: false,
            selected_message: None,
            session_list_index: 0,
            session_list: Vec::new(),
            session_search: String::new(),
            session_search_mode: false,
            session_rename_buf: String::new(),
            session_confirm_delete: false,
            sidebar_selected: 0,
            sidebar_body_idx: None,
            sidebar_body_scroll: 0,
            sidebar_formatted_json: None,
            agent_picker_index: 0,
            agent_list,
            copy_feedback: None,
            tab_completions: Vec::new(),
            tab_completion_index: 0,
            tab_completion_prefix: String::new(),
            tab_completion_cursor: 0,
            slash_visible: false,
            slash_index: 0,
            theme_index: 0,
            cached_filtered_sessions: None,
            cached_search_hash: 0,
        }
    }

    pub fn is_overlay(&self, kind: Overlay) -> bool {
        self.current == Some(kind)
    }

    pub fn show(&mut self, kind: Overlay) {
        if self.current != Some(kind) {
            self.current = Some(kind);
        }
    }

    pub fn toggle(&mut self, kind: Overlay) {
        if self.current == Some(kind) {
            self.current = None;
        } else {
            self.current = Some(kind);
        }
    }

    pub fn close(&mut self) {
        self.current = None;
    }

    #[allow(dead_code)]
    pub fn has_overlay(&self) -> bool {
        self.current.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle,
    Processing,
}

pub struct RenderState {
    pub heights: Vec<usize>,
    pub cached_width: usize,
    pub chat_height: u16,
    pub dirty: bool,
    pub last_drawn_at: Option<Instant>,
    pub component_version: u64,
    pub cached_scroller: Option<Scroller>,
    pub scroller_version: u64,
    pub cached_input_height: Option<(String, u16, u16)>,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            heights: Vec::new(),
            cached_width: 0,
            chat_height: 0,
            dirty: true,
            last_drawn_at: None,
            component_version: 0,
            cached_scroller: None,
            scroller_version: 0,
            cached_input_height: None,
        }
    }

    pub fn invalidate(&mut self) {
        self.heights.clear();
        self.dirty = true;
        self.component_version = self.component_version.wrapping_add(1);
    }

    pub fn invalidate_last(&mut self) {
        if !self.heights.is_empty() {
            self.heights[0] = 0;
        } else {
            self.heights.clear();
        }
        self.dirty = true;
    }

    /// 渲染成功完成后调用：清脏位并记录时间戳。
    pub fn mark_rendered(&mut self) {
        self.dirty = false;
        self.last_drawn_at = Some(Instant::now());
    }
}

pub fn spinner_char(spinner_start: Instant) -> char {
    const SPINNERS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];
    let elapsed = Instant::now().duration_since(spinner_start);
    let idx = (elapsed.as_millis() / 80) as usize % SPINNERS.len();
    SPINNERS[idx]
}

pub fn spinner_char_alt(spinner_start: Instant, chars: &[char]) -> char {
    let elapsed = Instant::now().duration_since(spinner_start);
    let idx = (elapsed.as_millis() / 100) as usize % chars.len();
    chars[idx]
}

pub struct ChatState {
    pub messages: Vec<Message>,
    pub message_timestamps: Vec<NaiveDateTime>,
    pub components: Vec<ComponentCell>,
    pub current_reasoning: String,
    pub api_messages: Option<Vec<Value>>,
    pub tool_call_count: usize,
    pub plan_steps: Vec<PlanStep>,
}

pub struct ScrollState {
    pub scroll_lines: usize,
    pub max_scroll: usize,
    pub stick_to_bottom: bool,
}

pub struct LlmState {
    pub state: AppState,
    pub status_text: String,
    pub token_usage: Option<i_rs_claw_core::llm::TokenUsage>,
}

pub struct App {
    pub chat: ChatState,
    pub scroll: ScrollState,
    pub llm: LlmState,
    pub input: InputState,
    pub overlay: OverlayState,
    pub config: Config,
    pub hit_regions: ClickRegionRegistry<usize>,
    pub http_logs: VecDeque<HttpLog>,
    pub reminder_text: Option<String>,
    pub current_agent: String,
    pub today_stats: TodaySummary,
    pub stats_history: Vec<i_rs_claw_core::stats::DailyStats>,
    pub skill_list: Vec<i_rs_claw_core::skill_store::SkillEntry>,
    pub plugin_list: Vec<PluginEntry>,
    pub spinner_start: Instant,
    pub render_state: RenderState,
}

impl App {
    pub fn new(config: Config) -> Self {
        let agent_list = config.agent_ids();

        Self {
            chat: ChatState {
                messages: Vec::new(),
                message_timestamps: Vec::new(),
                components: Vec::new(),
                current_reasoning: String::new(),
                api_messages: None,
                tool_call_count: 0,
                plan_steps: Vec::new(),
            },
            scroll: ScrollState {
                scroll_lines: 0,
                max_scroll: 0,
                stick_to_bottom: true,
            },
            llm: LlmState {
                state: AppState::Idle,
                status_text: String::new(),
                token_usage: None,
            },
            input: InputState::new(),
            overlay: OverlayState::new(agent_list),
            config,
            hit_regions: ClickRegionRegistry::new(),
            http_logs: VecDeque::new(),
            reminder_text: None,
            current_agent: "default".to_string(),
            today_stats: TodaySummary::default(),
            stats_history: Vec::new(),
            skill_list: Vec::new(),
            plugin_list: Vec::new(),
            spinner_start: Instant::now(),
            render_state: RenderState::new(),
        }
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.llm.state, AppState::Processing)
    }

    /// Rebuild the entire `components` Vec from `messages`. Use this
    /// after a wholesale replace (session load, reset). For every
    /// other code path prefer `push_component_for` /
    /// `pop_last_component` so component state survives.
    pub fn rebuild_components(&mut self) {
        self.chat.components = self
            .chat
            .messages
            .iter()
            .map(|m| Rc::new(RefCell::new(build_component_for(m))) as ComponentCell)
            .collect();
    }

    /// Append the component that corresponds to `msg` at the tail of
    /// the `components` Vec. Must be called right after pushing the
    /// message into `self.chat.messages`.
    pub fn push_component_for(&mut self, msg: &Message) {
        self.chat.components
            .push(Rc::new(RefCell::new(build_component_for(msg))) as ComponentCell);
    }

    /// Apply an op to the component at `idx`. Returns `true` if the
    /// component was actually mutated (i.e. the op landed in an arm
    /// that changes state). No-op if `idx` is out of range.
    pub fn apply_to_component(&mut self, idx: usize, op: ComponentOp) -> bool {
        if let Some(c) = self.chat.components.get(idx) {
            // The default `apply` is a no-op; we still mark dirty
            // because the clickable region is what changed.
            c.borrow_mut().apply(op);
            true
        } else {
            false
        }
    }

    /// Convenience: apply to the trailing component.
    pub fn apply_to_last_component(&mut self, op: ComponentOp) {
        if let Some(c) = self.chat.components.last() {
            c.borrow_mut().apply(op);
        }
    }

    /// Toggle the expand/collapse state of component `idx` (e.g.
    /// from a click or Space-key event). Marks the chat cache dirty
    /// so the next render reflects the new height.
    pub fn toggle_component_at(&mut self, idx: usize) {
        if self.apply_to_component(idx, ComponentOp::Toggle) {
            self.mark_dirty();
        }
    }

    pub fn mark_dirty(&mut self) {
        self.render_state.invalidate();
    }

    /// 仅 UI 浮层状态变更（overlay 选择、侧边栏滚动等），不需重算聊天布局缓存。
    pub fn mark_overlay_dirty(&mut self) {
        self.render_state.dirty = true;
    }

    pub fn add_user_message(&mut self, text: &str) {
        self.overlay.copy_feedback.take();
        let msg = Message::User {
            text: text.to_string(),
        };
        self.push_component_for(&msg);
        self.chat.messages.push(msg);
        self.chat.message_timestamps
            .push(chrono::Local::now().naive_local());
        self.llm.state = AppState::Processing;
        // Snap to the live tail: render reads `stick_to_bottom` and
        // overrides `scroll_lines` with `max_scroll` for us.
        self.scroll.stick_to_bottom = true;
        self.chat.plan_steps.clear();
        self.mark_dirty();
    }

    pub fn insert_char(&mut self, c: char) {
        self.overlay.copy_feedback.take();
        self.input.insert_char(c);
    }

    pub fn delete_before_cursor(&mut self) {
        self.input.delete_before_cursor();
    }

    pub fn move_cursor_left(&mut self) {
        self.input.move_cursor_left();
    }

    pub fn move_cursor_right(&mut self) {
        self.input.move_cursor_right();
    }

    #[allow(dead_code)]
    pub fn move_cursor_home(&mut self) {
        self.input.move_cursor_home();
    }

    #[allow(dead_code)]
    pub fn move_cursor_end(&mut self) {
        self.input.move_cursor_end();
    }

    pub fn commit_input_to_history(&mut self, text: &str) {
        self.input.commit_to_history(text);
        // Snap to the live tail when the user sends a message.
        // Render reads `stick_to_bottom` and overrides `scroll_lines`
        // with `max_scroll`, so we don't need an exact value here.
        self.scroll.stick_to_bottom = true;
    }

    #[allow(dead_code)]
    pub fn navigate_history_up(&mut self) -> Option<String> {
        self.input.navigate_up()
    }

    #[allow(dead_code)]
    pub fn navigate_history_down(&mut self) -> Option<String> {
        self.input.navigate_down()
    }

    /// Trackpad-optimized scroll: 3 lines per event for smooth macOS two-finger scrolling.
    /// "Up" = toward the top of the content (older messages), so scroll_lines DECREASES.
    pub fn scroll_up(&mut self) {
        // Capture the effective position before disengaging stickiness,
        // otherwise the user's first scroll-up from "follow bottom" mode
        // would jump to row 0 instead of stepping back from the bottom.
        if self.scroll.stick_to_bottom {
            self.scroll.scroll_lines = self.scroll.max_scroll;
            self.scroll.stick_to_bottom = false;
        }
        self.scroll.scroll_lines = self.scroll.scroll_lines.saturating_sub(3);
    }

    /// Trackpad-optimized scroll: 3 lines per event for smooth macOS two-finger scrolling.
    /// "Down" = toward the bottom of the content (newer messages), so scroll_lines INCREASES.
    pub fn scroll_down(&mut self) {
        if self.scroll.stick_to_bottom {
            return;
        }
        self.scroll.scroll_lines = self.scroll.scroll_lines.saturating_add(3);
        if self.scroll.scroll_lines >= self.scroll.max_scroll {
            self.scroll.scroll_lines = self.scroll.max_scroll;
            self.scroll.stick_to_bottom = true;
        }
    }

    /// Scroll one line at a time — used by arrow keys for precise navigation.
    pub fn scroll_up_one(&mut self) {
        if self.scroll.stick_to_bottom {
            self.scroll.scroll_lines = self.scroll.max_scroll;
            self.scroll.stick_to_bottom = false;
        }
        self.scroll.scroll_lines = self.scroll.scroll_lines.saturating_sub(1);
    }

    /// Scroll one line at a time — used by arrow keys for precise navigation.
    pub fn scroll_down_one(&mut self) {
        if self.scroll.stick_to_bottom {
            return;
        }
        self.scroll.scroll_lines = self.scroll.scroll_lines.saturating_add(1);
        if self.scroll.scroll_lines >= self.scroll.max_scroll {
            self.scroll.scroll_lines = self.scroll.max_scroll;
            self.scroll.stick_to_bottom = true;
        }
    }

    /// Page-scroll helper used by PageUp / PageDown.
    /// Keeps a 2-row overlap so the user retains visual context
    /// between pages. `dir > 0` = PageUp (toward top), `dir < 0` = PageDown.
    pub fn scroll_page(&mut self, dir: i32) {
        let area_lines = (self.render_state.chat_height as usize)
            .saturating_sub(2)
            .max(1);
        if dir > 0 {
            if self.scroll.stick_to_bottom {
                self.scroll.scroll_lines = self.scroll.max_scroll;
                self.scroll.stick_to_bottom = false;
            }
            self.scroll.scroll_lines = self.scroll.scroll_lines.saturating_sub(area_lines);
        } else if dir < 0 {
            if self.scroll.stick_to_bottom {
                return;
            }
            self.scroll.scroll_lines = self.scroll.scroll_lines.saturating_add(area_lines);
            if self.scroll.scroll_lines >= self.scroll.max_scroll {
                self.scroll.scroll_lines = self.scroll.max_scroll;
                self.scroll.stick_to_bottom = true;
            }
        }
    }

    pub fn scroll_to_bottom_if_stuck(&mut self) {}

    /// 让选中的消息滚入视口。若已在视口内则保持滚动位置不变。
    /// 依据 render_state.heights 估算每个消息行高；若缓存为空（如首屏未渲染）则放弃调整。
    ///
    /// heights 约定：heights[0] = 最旧消息，heights[len-1] = 最新消息（与 render_chat 一致）。
    /// scroll_lines 从顶部计数：0 = 顶部（最旧），max_scroll = 底部（最新）。
    pub fn scroll_to_selected(&mut self) {
        let Some(idx) = self.overlay.selected_message else {
            return;
        };
        let heights = &self.render_state.heights;
        if heights.is_empty() || idx >= heights.len() {
            return;
        }
        // sel_top = 选中消息顶端距顶部的行数（即所有比它更旧的消息总高度）
        let sel_top: usize = heights.iter().take(idx).sum();
        let sel_height = heights[idx];
        let sel_bottom = sel_top + sel_height;
        // 用渲染时回填的 chat 高度算视口行数，避免依赖 max_scroll（展开/折叠后滞后）
        let area_lines = (self.render_state.chat_height as usize)
            .saturating_sub(1)
            .max(1);
        // max_scroll 也要用最新的 heights 重新计算（max_scroll 是渲染时存的，旧值会错）
        let total: usize = heights.iter().sum();
        let new_max_scroll = total.saturating_sub(area_lines);
        // stick_to_bottom 时实际视口在底部，所以从 max_scroll 起算
        let effective_scroll = if self.scroll.stick_to_bottom {
            new_max_scroll
        } else {
            self.scroll.scroll_lines
        };
        let viewport_top = effective_scroll;
        let viewport_bottom = effective_scroll.saturating_add(area_lines);

        if sel_top < viewport_top || sel_bottom > viewport_bottom {
            // 选中的不在视口内 → 居中对齐：让选中条落在视口中央，
            // 既保证选中条可见，又让用户能感知上下文。
            self.scroll.scroll_lines = sel_top.saturating_sub(area_lines / 2);
            self.scroll.stick_to_bottom = false;
        }
        // 夹到合法范围
        self.scroll.scroll_lines = self.scroll.scroll_lines.min(new_max_scroll);
        self.scroll.max_scroll = new_max_scroll;
    }

    /// 事件处理中调用：在 mark_dirty 清空 heights 后，用组件自身的
    /// `height()` 精确重算每个消息行高，让 scroll_to_selected 在
    /// 展开/折叠后不必等下一次渲染就能算出正确的视口位置。
    /// 渲染时会基于 chat.rs 的真实 layout 重新精修 heights。
    ///
    /// heights 约定：时间顺序存储（heights[0] = 最旧消息），与 render_chat 一致。
    pub fn rebuild_heights_approx(&mut self) {
        let text_width = self.render_state.cached_width.max(20);
        let mut heights: Vec<usize> = Vec::with_capacity(self.chat.components.len());
        // 时间顺序遍历以匹配 render_chat 的 heights 约定
        for comp in self.chat.components.iter() {
            let h = comp.borrow().height(text_width as u16);
            heights.push((h as usize).max(1));
        }
        self.render_state.heights = heights;
        // 同步 max_scroll 给主渲染用，避免短暂不一致
        let area_lines = (self.render_state.chat_height as usize)
            .saturating_sub(1)
            .max(1);
        let total: usize = self.render_state.heights.iter().sum();
        self.scroll.max_scroll = total.saturating_sub(area_lines);
    }

    pub fn set_status(&mut self, text: &str) {
        self.llm.status_text = text.to_string();
    }

    pub fn start_assistant_message(&mut self) {
        // Move any reasoning that streamed in *between* rounds (e.g. while we
        // were waiting for a tool result) onto the Assistant message we are
        // about to create. Previously the reasoning was only flushed when
        // the previous message was already an Assistant, so it was silently
        // dropped every time the last message was a `ToolCall` — which is
        // the common case mid-conversation.
        let carried_reasoning = std::mem::take(&mut self.chat.current_reasoning);
        let is_empty_assistant = matches!(
            self.chat.messages.last(),
            Some(Message::Assistant { text, .. }) if text.is_empty()
        );
        if !is_empty_assistant {
            let msg = Message::Assistant {
                text: String::new(),
                reasoning: carried_reasoning,
                token_usage: None,
            };
            self.push_component_for(&msg);
            self.chat.messages.push(msg);
            self.chat.message_timestamps
                .push(chrono::Local::now().naive_local());
            self.mark_dirty();
        } else if let Some(Message::Assistant { reasoning, .. }) = self.chat.messages.last_mut()
            && !carried_reasoning.is_empty()
        {
            // Reuse the trailing empty Assistant so we don't end up with
            // two adjacent empty Assistant blocks. Also push the new
            // reasoning into the matching component so the trailing
            // assistant's `reasoning` and the component's `reasoning`
            // stay in sync.
            reasoning.push_str(&carried_reasoning);
            self.apply_to_last_component(ComponentOp::AppendReasoning(carried_reasoning));
            self.render_state.invalidate_last();
        }
    }

    pub fn append_assistant_text(&mut self, text: &str) {
        let last_is_assistant = matches!(self.chat.messages.last_mut(), Some(Message::Assistant { .. }));
        if !last_is_assistant {
            self.start_assistant_message();
        }
        // First text on a fresh Assistant message — flush any reasoning
        // that streamed in *before* the first token. Without this the
        // reasoning would stay in `current_reasoning` and be discarded at
        // the next round boundary (when the last message is a ToolCall).
        if let Some(Message::Assistant {
            text: t, reasoning, ..
        }) = self.chat.messages.last_mut()
            && t.is_empty()
            && !self.chat.current_reasoning.is_empty()
        {
            let pending = std::mem::take(&mut self.chat.current_reasoning);
            reasoning.push_str(&pending);
            // Sync the trailing component too.
            self.apply_to_last_component(ComponentOp::AppendReasoning(pending));
        }
        if let Some(Message::Assistant { text: t, .. }) = self.chat.messages.last_mut() {
            t.push_str(text);
            // Mirror onto the trailing component so its `height()`,
            // body rows, and `reasoning` stay consistent with the
            // `Message` enum.
            self.apply_to_last_component(ComponentOp::AppendText(text.to_string()));
            self.render_state.invalidate_last();
        }
        // Streaming tokens: keep the viewport pinned to the bottom if the
        // user is following the live tail, but don't yank them out of a
        // back-scroll position.
        self.scroll_to_bottom_if_stuck();
    }

    pub fn add_tool_call(
        &mut self,
        name: &str,
        args: &str,
        result: &str,
        step: usize,
        total_steps: usize,
    ) {
        let msg = Message::ToolCall {
            name: name.to_string(),
            args: args.to_string(),
            result: result.to_string(),
            step,
            total_steps,
        };
        self.push_component_for(&msg);
        self.chat.messages.push(msg);
        self.chat.message_timestamps
            .push(chrono::Local::now().naive_local());
        self.chat.tool_call_count += 1;
        self.scroll_to_bottom_if_stuck();
        self.mark_dirty();
    }

    pub fn add_http_log(&mut self, log: HttpLog) {
        self.http_logs.push_front(log);
        while self.http_logs.len() > 50 {
            self.http_logs.pop_back();
        }
        self.mark_overlay_dirty();
    }

    pub fn add_error(&mut self, text: &str) {
        if !self.chat.current_reasoning.is_empty()
            && let Some(Message::Assistant { reasoning, .. }) = self.chat.messages.last_mut()
        {
            reasoning.push_str(&self.chat.current_reasoning);
        }
        self.chat.current_reasoning.clear();
        // Only discard a trailing Assistant placeholder if it is *truly*
        // empty (no text, no reasoning). Otherwise the error banner would
        // eat the user's thinking content as well.
        if let Some(Message::Assistant {
            text: t, reasoning, ..
        }) = self.chat.messages.last()
            && t.is_empty()
            && reasoning.is_empty()
        {
            self.chat.messages.pop();
            self.chat.message_timestamps.pop();
            self.chat.components.pop();
        }
        let msg = Message::Error {
            text: text.to_string(),
        };
        self.push_component_for(&msg);
        self.chat.messages.push(msg);
        self.chat.message_timestamps
            .push(chrono::Local::now().naive_local());
        self.chat.api_messages = None;
        self.llm.state = AppState::Idle;
        self.llm.status_text.clear();
        self.scroll_to_bottom_if_stuck();
        self.mark_dirty();
    }

    pub fn finish_processing(&mut self, api_messages: Option<Vec<Value>>) {
        if !self.chat.current_reasoning.is_empty()
            && let Some(Message::Assistant { reasoning, .. }) = self.chat.messages.last_mut()
        {
            reasoning.push_str(&self.chat.current_reasoning);
        }
        self.chat.current_reasoning.clear();
        // Only pop the trailing Assistant placeholder if it has nothing to
        // show. Previously we dropped it whenever `text` was empty, which
        // threw away any reasoning that had been streamed in.
        if let Some(Message::Assistant {
            text: t, reasoning, ..
        }) = self.chat.messages.last()
            && t.is_empty()
            && reasoning.is_empty()
        {
            self.chat.messages.pop();
            self.chat.message_timestamps.pop();
            self.chat.components.pop();
        }
        self.chat.api_messages = api_messages;
        self.llm.state = AppState::Idle;
        self.llm.status_text.clear();
        self.scroll_to_bottom_if_stuck();
        self.mark_dirty();
    }

    pub fn detect_plan(&mut self, text: &str) {
        if !self.is_processing() {
            return;
        }
        let new_steps: Vec<PlanStep> = text
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                let rest = trimmed
                    .strip_prefix(|c: char| c.is_ascii_digit())
                    .unwrap_or("");
                let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
                rest.strip_prefix(". ")
                    .map(|r| r.trim_end_matches(['.', '，', ',']))
                    .filter(|r| !r.is_empty())
                    .map(|r| PlanStep {
                        description: r.to_string(),
                        done: false,
                    })
            })
            .collect();

        if new_steps.len() >= self.chat.plan_steps.len() {
            let done_count = self.chat.plan_steps.iter().take_while(|s| s.done).count();
            self.chat.plan_steps = new_steps;
            for step in self.chat.plan_steps.iter_mut().take(done_count) {
                step.done = true;
            }
        }
    }

    pub fn mark_next_plan_step_done(&mut self) {
        for step in &mut self.chat.plan_steps {
            if !step.done {
                step.done = true;
                break;
            }
        }
    }

    pub fn reset_for_new_session(&mut self) {
        self.chat.messages.clear();
        self.chat.message_timestamps.clear();
        self.chat.components.clear();
        self.chat.api_messages = None;
        self.llm.state = AppState::Idle;
        self.chat.tool_call_count = 0;
        self.llm.status_text.clear();
        self.llm.token_usage = None;
        self.input = InputState::new();
        self.overlay.current = None;
        self.http_logs.clear();
        self.overlay.sidebar_selected = 0;
        self.overlay.sidebar_body_idx = None;
        self.overlay.sidebar_body_scroll = 0;
        self.chat.plan_steps.clear();
        self.overlay.session_search.clear();
        self.overlay.session_search_mode = false;
        self.overlay.selected_message = None;
        self.overlay.selection_mode = false;
        // Reset viewport to "follow live tail" so the next message the
        // user sends is visible without manual scrolling.
        self.scroll.scroll_lines = 0;
        self.scroll.max_scroll = 0;
        self.scroll.stick_to_bottom = true;
        self.mark_dirty();
    }

    pub fn sync_message_timestamps(&mut self) {
        let now = chrono::Local::now().naive_local();
        while self.chat.message_timestamps.len() < self.chat.messages.len() {
            self.chat.message_timestamps.push(now);
        }
        if self.chat.message_timestamps.len() > self.chat.messages.len() {
            self.chat.message_timestamps.truncate(self.chat.messages.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        let mut c = Config::new();
        c.api_key = "test".to_string();
        c
    }

    #[test]
    fn test_app_new() {
        let app = App::new(test_config());
        assert!(app.chat.messages.is_empty());
        assert_eq!(app.llm.state, AppState::Idle);
        assert!(!app.is_processing());
        assert!(app.http_logs.is_empty());
        assert_eq!(app.current_agent, "default");
        assert_eq!(app.chat.tool_call_count, 0);
    }

    #[test]
    fn test_add_user_message_sets_processing() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        assert_eq!(app.chat.messages.len(), 1);
        assert!(matches!(app.chat.messages[0], Message::User { ref text } if text == "hello"));
        assert!(app.is_processing());
        assert!(app.scroll.stick_to_bottom, "sending snaps viewport to live tail");
    }

    #[test]
    fn test_finish_processing_clears_state() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        assert!(app.is_processing());

        app.finish_processing(Some(vec![
            serde_json::json!({"role": "assistant", "content": "hi"}),
        ]));
        assert!(!app.is_processing());
        assert_eq!(app.llm.state, AppState::Idle);
        assert!(app.llm.status_text.is_empty());
        assert!(app.chat.api_messages.is_some());
    }

    #[test]
    fn test_finish_processing_removes_empty_assistant() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();
        assert_eq!(app.chat.messages.len(), 2);
        app.finish_processing(None);
        assert_eq!(app.chat.messages.len(), 1);
        assert!(!app.is_processing());
    }

    #[test]
    fn test_finish_processing_preserves_assistant_with_reasoning_only() {
        // Reasoning streamed in but no visible text was emitted. The
        // placeholder Assistant must be kept so the user can see the
        // thinking content; previously it was dropped along with the text
        // check.
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();
        app.chat.current_reasoning.push_str("thinking hard");
        app.finish_processing(None);
        assert_eq!(app.chat.messages.len(), 2);
        if let Message::Assistant {
            text, reasoning, ..
        } = &app.chat.messages[1]
        {
            assert!(text.is_empty());
            assert_eq!(reasoning, "thinking hard");
        } else {
            panic!("expected Assistant, got other variant");
        }
    }

    #[test]
    fn test_start_assistant_carries_reasoning_across_tool_call() {
        // Reasoning emitted *between* rounds (when the last message is a
        // ToolCall) must be moved onto the new Assistant placeholder,
        // otherwise it would be silently dropped.
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();
        app.append_assistant_text("thinking out loud");
        app.add_tool_call("i_rs", "{}", "{}", 1, 1);
        // Simulate the LLM streaming reasoning for the next round.
        app.chat.current_reasoning.push_str("between-rounds thought");
        app.start_assistant_message();
        if let Message::Assistant {
            text, reasoning, ..
        } = app.chat.messages.last().unwrap()
        {
            assert!(text.is_empty());
            assert_eq!(reasoning, "between-rounds thought");
        } else {
            panic!("expected Assistant, got other variant");
        }
    }

    #[test]
    fn test_append_assistant_text_flushes_pending_reasoning() {
        // Reasoning that streamed in *before* the first visible token must
        // be moved onto the Assistant message as soon as text arrives.
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();
        app.chat.current_reasoning.push_str("planning");
        app.append_assistant_text("hi");
        if let Message::Assistant {
            text, reasoning, ..
        } = &app.chat.messages[1]
        {
            assert_eq!(text, "hi");
            assert_eq!(reasoning, "planning");
        } else {
            panic!("expected Assistant, got other variant");
        }
        assert!(app.chat.current_reasoning.is_empty());
    }

    #[test]
    fn test_add_error_sets_idle() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        assert!(app.is_processing());

        app.add_error("something went wrong");
        assert!(!app.is_processing());
        assert_eq!(app.chat.messages.len(), 2);
        assert!(
            matches!(app.chat.messages[1], Message::Error { ref text } if text == "something went wrong")
        );
        assert!(app.chat.api_messages.is_none());
    }

    #[test]
    fn test_add_error_removes_empty_assistant() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();
        assert_eq!(app.chat.messages.len(), 2);
        app.add_error("err");
        assert_eq!(app.chat.messages.len(), 2);
        assert!(matches!(app.chat.messages[1], Message::Error { .. }));
    }

    #[test]
    fn test_assistant_message_append() {
        let mut app = App::new(test_config());
        app.start_assistant_message();
        assert_eq!(app.chat.messages.len(), 1);
        assert!(matches!(app.chat.messages[0], Message::Assistant { ref text, .. } if text.is_empty()));

        app.append_assistant_text("hello ");
        app.append_assistant_text("world");
        assert!(
            matches!(app.chat.messages[0], Message::Assistant { ref text, .. } if text == "hello world")
        );
    }

    #[test]
    fn test_append_assistant_reuses_empty() {
        let mut app = App::new(test_config());
        app.append_assistant_text("direct");
        assert_eq!(app.chat.messages.len(), 1);
        assert!(matches!(app.chat.messages[0], Message::Assistant { ref text, .. } if text == "direct"));
    }

    #[test]
    fn test_add_tool_call() {
        let mut app = App::new(test_config());
        app.add_tool_call("weight", r#"{"action":"list"}"#, "OK", 1, 2);
        assert_eq!(app.chat.messages.len(), 1);
        assert!(
            matches!(&app.chat.messages[0], Message::ToolCall { name, step: 1, total_steps: 2, .. } if name == "weight")
        );
        assert_eq!(app.chat.tool_call_count, 1);
    }

    /// Regression: older session records save tool_call records without
    /// `step`/`total_steps`. Deserialization must tolerate this, otherwise
    /// tool_call messages silently vanish on session reload (the mini
    /// program and dashboard-ui both lose them).
    #[test]
    fn test_message_from_jsonl_tool_call_without_step() {
        let v = serde_json::json!({
            "type": "tool_call",
            "name": "weight",
            "args": "{\"command\":\"list\"}",
            "result": "ok"
        });
        let msg =
            message_from_jsonl(v).expect("tool_call without step/total_steps must deserialize");
        match msg {
            Message::ToolCall {
                name,
                args,
                result,
                step,
                total_steps,
            } => {
                assert_eq!(name, "weight");
                assert_eq!(args, "{\"command\":\"list\"}");
                assert_eq!(result, "ok");
                assert_eq!(step, 0, "missing step defaults to 0");
                assert_eq!(total_steps, 0, "missing total_steps defaults to 0");
            }
            _ => panic!("expected ToolCall"),
        }
    }

    #[test]
    fn test_message_from_jsonl_tool_call_with_step() {
        let v = serde_json::json!({
            "type": "tool_call",
            "name": "water",
            "args": "{}",
            "result": "ok",
            "step": 2,
            "total_steps": 3
        });
        let msg = message_from_jsonl(v).expect("tool_call with step/total_steps must deserialize");
        match msg {
            Message::ToolCall {
                name,
                step,
                total_steps,
                ..
            } => {
                assert_eq!(name, "water");
                assert_eq!(step, 2);
                assert_eq!(total_steps, 3);
            }
            _ => panic!("expected ToolCall"),
        }
    }

    #[test]
    fn test_scroll() {
        // Convention: scroll_lines counts rows FROM TOP (0 = oldest/top,
        // max_scroll = newest/bottom). scroll_up subtracts, scroll_down adds.
        let mut app = App::new(test_config());
        app.scroll.max_scroll = 30;
        assert_eq!(app.scroll.scroll_lines, 0);
        assert!(app.scroll.stick_to_bottom);

        // scroll_up from sticky: capture max (30), subtract 3 → 27
        app.scroll_up();
        assert_eq!(app.scroll.scroll_lines, 27);
        assert!(!app.scroll.stick_to_bottom);

        app.scroll_up();
        assert_eq!(app.scroll.scroll_lines, 24);

        // scroll_down: 24 + 3 = 27 (still < max)
        app.scroll_down();
        assert_eq!(app.scroll.scroll_lines, 27);
        assert!(!app.scroll.stick_to_bottom);

        // scroll_down hits max → re-engage sticky
        app.scroll_down();
        assert_eq!(app.scroll.scroll_lines, 30);
        assert!(app.scroll.stick_to_bottom);

        // scroll_down at sticky bottom is a no-op
        app.scroll_down();
        assert_eq!(app.scroll.scroll_lines, 30);

        // scroll_up captures and steps back
        app.scroll_up();
        assert_eq!(app.scroll.scroll_lines, 27);

        // Scroll all the way to the top with trackpad-sized steps
        app.scroll_up();
        app.scroll_up();
        app.scroll_up();
        app.scroll_up();
        app.scroll_up();
        app.scroll_up();
        app.scroll_up();
        app.scroll_up();
        app.scroll_up();
        // 27 - 9*3 = 0 (saturating)
        assert_eq!(app.scroll.scroll_lines, 0);
        assert!(!app.scroll.stick_to_bottom);

        // Beyond top: saturates at 0
        app.scroll_up();
        assert_eq!(app.scroll.scroll_lines, 0);
    }

    #[test]
    fn test_scroll_to_selected_newest_visible_at_bottom() {
        // 10 条消息各占 3 行，总 30 行，area_lines = 10，max_scroll = 20
        // heights 时间顺序：heights[0]=最旧(m0), heights[9]=最新(m9)
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.scroll.max_scroll = 20;
        // stick_to_bottom=true（底部），选中 idx=9（最新）→ sel_top=27, viewport=[20,30) → 可见 → 不变
        app.overlay.selected_message = Some(9);
        app.scroll_to_selected();
        assert_eq!(app.scroll.scroll_lines, 0);
        assert!(
            app.scroll.stick_to_bottom,
            "sticky preserved when selected already visible"
        );
    }

    #[test]
    fn test_scroll_to_selected_oldest_not_visible_at_bottom() {
        // stick_to_bottom=true（底部），选中 idx=0（最旧）→ 需要向上滚到顶部
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.scroll.max_scroll = 20;
        app.overlay.selected_message = Some(0);
        app.scroll_to_selected();
        // sel_top=0, 居中: 0-5=0 (saturating)
        assert_eq!(app.scroll.scroll_lines, 0);
        assert!(!app.scroll.stick_to_bottom, "moved away from bottom");
    }

    #[test]
    fn test_scroll_to_selected_oldest_visible_at_top() {
        // scroll_lines=0（顶部），选中 idx=0（最旧）→ 在视口内 → 不变
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.scroll.max_scroll = 20;
        app.scroll.scroll_lines = 0;
        app.scroll.stick_to_bottom = false;
        app.overlay.selected_message = Some(0);
        app.scroll_to_selected();
        // sel_top=0, sel_bottom=3, viewport=[0,10) → 可见 → 不变
        assert_eq!(app.scroll.scroll_lines, 0);
        assert!(!app.scroll.stick_to_bottom);
    }

    #[test]
    fn test_scroll_to_selected_newest_not_visible_at_top() {
        // scroll_lines=0（顶部），选中 idx=9（最新）→ 需要向下滚动到底
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.scroll.max_scroll = 20;
        app.scroll.scroll_lines = 0;
        app.scroll.stick_to_bottom = false;
        app.overlay.selected_message = Some(9);
        app.scroll_to_selected();
        // sel_top=27, 居中: 27-5=22, clamp 到 max=20
        assert_eq!(app.scroll.scroll_lines, 20);
        assert_eq!(app.scroll.max_scroll, 20);
        assert!(!app.scroll.stick_to_bottom);
    }

    #[test]
    fn test_scroll_to_selected_mid_visible_no_change() {
        // scroll_lines=10，选中 idx=5 → 检查是否在视口内
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.scroll.max_scroll = 20;
        app.scroll.scroll_lines = 10;
        app.scroll.stick_to_bottom = false;
        app.overlay.selected_message = Some(5);
        app.scroll_to_selected();
        // sel_top=sum(heights[0..5])=15, sel_bottom=18, viewport=[10,20) → 可见
        assert_eq!(app.scroll.scroll_lines, 10);
    }

    #[test]
    fn test_scroll_to_selected_empty_heights() {
        let mut app = App::new(test_config());
        app.overlay.selected_message = Some(0);
        app.scroll_to_selected();
        assert_eq!(app.scroll.scroll_lines, 0);
    }

    #[test]
    fn test_scroll_to_selected_none_selected() {
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.scroll.scroll_lines = 5;
        app.scroll_to_selected();
        assert_eq!(app.scroll.scroll_lines, 5);
    }

    #[test]
    fn test_set_status() {
        let mut app = App::new(test_config());
        assert!(app.llm.status_text.is_empty());
        app.set_status("thinking…");
        assert_eq!(app.llm.status_text, "thinking…");
    }

    #[test]
    fn test_reset_for_new_session() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.add_tool_call("test", "{}", "ok", 1, 1);
        app.set_status("done");
        app.reset_for_new_session();
        assert!(app.chat.messages.is_empty());
        assert!(app.chat.message_timestamps.is_empty());
        assert!(app.chat.api_messages.is_none());
        assert_eq!(app.chat.tool_call_count, 0);
        assert!(app.llm.status_text.is_empty());
        assert!(app.http_logs.is_empty());
        assert!(app.chat.plan_steps.is_empty());
    }

    #[test]
    fn test_stick_to_bottom_tracks_scroll_position() {
        // Sticky-bottom state should toggle correctly as the user moves
        // up and down the chat history. New messages preserve the
        // viewport when the user is reading older content (stick=false),
        // and the renderer pins to the bottom when stick=true.
        let mut app = App::new(test_config());
        app.scroll.max_scroll = 30;
        assert!(app.scroll.stick_to_bottom, "starts at the bottom");

        // scroll_up from sticky: capture 30, step back → 27
        app.scroll_up();
        assert!(!app.scroll.stick_to_bottom);

        app.scroll_up();
        assert!(!app.scroll.stick_to_bottom);

        // scroll_down_one: 24 + 1 = 25
        app.scroll_down_one();
        assert!(!app.scroll.stick_to_bottom, "still above the bottom");

        // Walk back to the bottom with 1-line steps; stick_to_bottom
        // re-engages when scroll_lines reaches max_scroll.
        while app.scroll.scroll_lines < app.scroll.max_scroll {
            app.scroll_down_one();
        }
        assert!(app.scroll.stick_to_bottom, "re-engaged at the bottom");

        // Scrolling away disengages; manually re-engage.
        app.scroll_up();
        assert!(!app.scroll.stick_to_bottom);
        app.scroll.stick_to_bottom = true;

        // New tool call preserves stickiness (scroll_to_bottom_if_stuck is
        // a no-op; the renderer reads stick_to_bottom directly).
        let prev_count = app.chat.tool_call_count;
        app.add_tool_call("weight", "{}", "ok", 0, 1);
        assert!(app.scroll.stick_to_bottom, "stuck at bottom stays stuck");
        assert_eq!(app.chat.tool_call_count, prev_count + 1);

        // When not stuck, new content must not yank the user.
        app.scroll_up();
        app.scroll_up();
        let pinned = app.scroll.scroll_lines;
        app.add_tool_call("weight2", "{}", "ok", 0, 1);
        assert_eq!(app.scroll.scroll_lines, pinned, "back-scroll position preserved");
    }

    #[test]
    fn test_scroll_page_uses_viewport_with_overlap() {
        let mut app = App::new(test_config());
        app.render_state.chat_height = 20;
        app.scroll.max_scroll = 200;

        // PageUp (dir>0): from sticky, capture 200 then subtract 18 → 182
        app.scroll_page(1);
        // 20 - 2 = 18
        assert_eq!(app.scroll.scroll_lines, 182);
        assert!(!app.scroll.stick_to_bottom);

        app.scroll_page(1);
        assert_eq!(app.scroll.scroll_lines, 164);

        // PageDown (dir<0): 164 + 18 = 182
        app.scroll_page(-1);
        assert_eq!(app.scroll.scroll_lines, 182);

        // PageDown hits bottom → re-engage sticky
        app.scroll_page(-1);
        assert_eq!(app.scroll.scroll_lines, 200);
        assert!(app.scroll.stick_to_bottom);
    }

    #[test]
    fn test_mark_overlay_dirty_preserves_chat_cache() {
        let mut app = App::new(test_config());
        app.render_state.dirty = false;
        app.render_state.heights.push(3);

        app.mark_overlay_dirty();

        assert!(app.render_state.dirty);
        assert_eq!(app.render_state.heights.len(), 1);
    }

    #[test]
    fn test_mark_dirty_clears_heights() {
        let mut app = App::new(test_config());
        app.render_state.dirty = false;
        app.render_state.heights.push(3);

        app.mark_dirty();

        assert!(app.render_state.dirty);
        assert!(app.render_state.heights.is_empty());
    }

    #[test]
    fn test_detect_plan() {
        let mut app = App::new(test_config());
        app.add_user_message("plan something");
        app.detect_plan("1. first step.\n2. second step，\n3. third step.");
        assert_eq!(app.chat.plan_steps.len(), 3);
        assert_eq!(app.chat.plan_steps[0].description, "first step");
        assert!(!app.chat.plan_steps[0].done);
        assert_eq!(app.chat.plan_steps[2].description, "third step");
    }

    #[test]
    fn test_detect_plan_ignores_non_numbered() {
        let mut app = App::new(test_config());
        app.add_user_message("hi");
        app.detect_plan("- bullet item\nplain text\n1. actual step");
        assert_eq!(app.chat.plan_steps.len(), 1);
        assert_eq!(app.chat.plan_steps[0].description, "actual step");
    }

    #[test]
    fn test_detect_plan_only_when_processing() {
        let mut app = App::new(test_config());
        app.detect_plan("1. first step");
        assert!(app.chat.plan_steps.is_empty());
    }

    #[test]
    fn test_mark_next_plan_step_done() {
        let mut app = App::new(test_config());
        app.add_user_message("do it");
        app.detect_plan("1. step A\n2. step B");
        app.mark_next_plan_step_done();
        assert!(app.chat.plan_steps[0].done);
        assert!(!app.chat.plan_steps[1].done);
        app.mark_next_plan_step_done();
        assert!(app.chat.plan_steps[1].done);
    }

    #[test]
    fn test_sync_message_timestamps_fills_gaps() {
        let mut app = App::new(test_config());
        app.chat.messages.push(Message::User {
            text: "a".to_string(),
        });
        app.chat.messages.push(Message::User {
            text: "b".to_string(),
        });
        assert!(app.chat.message_timestamps.is_empty());
        app.sync_message_timestamps();
        assert_eq!(app.chat.message_timestamps.len(), 2);
    }

    #[test]
    fn test_sync_message_timestamps_truncates_excess() {
        let mut app = App::new(test_config());
        app.chat.messages.push(Message::User {
            text: "a".to_string(),
        });
        app.chat.message_timestamps
            .push(chrono::Local::now().naive_local());
        app.chat.message_timestamps
            .push(chrono::Local::now().naive_local());
        app.sync_message_timestamps();
        assert_eq!(app.chat.message_timestamps.len(), 1);
    }

    #[test]
    fn test_add_http_log_maintains_bound() {
        let mut app = App::new(test_config());
        for i in 0..55 {
            app.add_http_log(HttpLog {
                timestamp: "t".to_string(),
                status: 200,
                duration_ms: i,
                model: "m".to_string(),
                prompt_tokens: 0,
                completion_tokens: 0,
                error: None,
                request_body: String::new(),
                msg_count: 0,
            });
        }
        assert_eq!(app.http_logs.len(), 50);
        assert_eq!(app.http_logs[0].duration_ms, 54);
    }

    #[test]
    fn test_input_state_insert_and_delete() {
        let mut input = InputState::new();
        input.insert_char('h');
        input.insert_char('i');
        assert_eq!(input.text, "hi");
        assert_eq!(input.cursor, 2);

        input.move_cursor_left();
        assert_eq!(input.cursor, 1);
        input.insert_char('x');
        assert_eq!(input.text, "hxi");

        input.delete_before_cursor();
        assert_eq!(input.text, "hi");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn test_input_cursor_bounds() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.move_cursor_left();
        assert_eq!(input.cursor, 0);
        input.move_cursor_left();
        assert_eq!(input.cursor, 0);

        input.move_cursor_right();
        assert_eq!(input.cursor, 1);
        input.move_cursor_right();
        assert_eq!(input.cursor, 1);

        input.delete_before_cursor();
        assert_eq!(input.text, "");
        input.delete_before_cursor();
        assert_eq!(input.text, "");
    }

    #[test]
    fn test_input_history() {
        let mut input = InputState::new();
        assert!(input.navigate_up().is_none());

        input.commit_to_history("hello");
        input.commit_to_history("world");
        assert_eq!(input.history.len(), 2);

        assert_eq!(input.navigate_up().as_deref(), Some("world"));
        assert_eq!(input.navigate_up().as_deref(), Some("hello"));
        assert_eq!(input.navigate_up(), None);

        assert_eq!(input.navigate_down().as_deref(), Some("world"));
        assert_eq!(input.navigate_down(), None);
    }

    #[test]
    fn test_input_history_dedup() {
        let mut input = InputState::new();
        input.commit_to_history("same");
        input.commit_to_history("same");
        assert_eq!(input.history.len(), 1);
    }

    #[test]
    fn test_input_history_max_50() {
        let mut input = InputState::new();
        for i in 0..60 {
            input.commit_to_history(&format!("item{}", i));
        }
        assert_eq!(input.history.len(), 50);
        assert_eq!(input.history[0], "item10");
        assert_eq!(input.history[49], "item59");
    }

    #[test]
    fn test_empty_commit_does_not_add_to_history() {
        let mut input = InputState::new();
        input.commit_to_history("");
        assert!(input.history.is_empty());
    }

    #[test]
    fn test_overlay_filtered_sessions_empty() {
        let overlay = OverlayState::new(vec!["default".to_string()]);
        let result = overlay.filtered_sessions();
        assert!(result.is_empty());
    }

    #[test]
    fn test_overlay_filtered_sessions_search() {
        let mut overlay = OverlayState::new(vec!["default".to_string()]);
        overlay.session_list = vec![
            i_rs_claw_core::session::SessionMeta {
                id: "1".to_string(),
                title: "Weight tracking".to_string(),
                agent_id: "default".to_string(),
                user_id: "default".to_string(),
                state: i_rs_claw_core::session::SessionState::Active,
                created_at: 0,
                updated_at: 0,
                message_count: 0,
            },
            i_rs_claw_core::session::SessionMeta {
                id: "2".to_string(),
                title: "Mood log".to_string(),
                agent_id: "default".to_string(),
                user_id: "default".to_string(),
                state: i_rs_claw_core::session::SessionState::Active,
                created_at: 0,
                updated_at: 0,
                message_count: 0,
            },
            i_rs_claw_core::session::SessionMeta {
                id: "3".to_string(),
                title: "Weight history".to_string(),
                agent_id: "default".to_string(),
                user_id: "default".to_string(),
                state: i_rs_claw_core::session::SessionState::Active,
                created_at: 0,
                updated_at: 0,
                message_count: 0,
            },
        ];

        assert_eq!(overlay.filtered_sessions().len(), 3);

        overlay.session_search = "weight".to_string();
        let result = overlay.filtered_sessions();
        assert_eq!(result.len(), 2);
        assert!(
            result
                .iter()
                .all(|s| s.title.to_lowercase().contains("weight"))
        );

        overlay.session_search = "mood".to_string();
        assert_eq!(overlay.filtered_sessions().len(), 1);

        overlay.session_search = "nonexistent".to_string();
        assert!(overlay.filtered_sessions().is_empty());
    }

    #[test]
    fn test_overlay_filtered_sessions_case_insensitive() {
        let mut overlay = OverlayState::new(vec!["default".to_string()]);
        overlay.session_list = vec![i_rs_claw_core::session::SessionMeta {
            id: "1".to_string(),
            title: "Weight Tracking".to_string(),
            agent_id: "default".to_string(),
                user_id: "default".to_string(),
            state: i_rs_claw_core::session::SessionState::Active,
            created_at: 0,
            updated_at: 0,
            message_count: 0,
        }];
        overlay.session_search = "WEIGHT".to_string();
        assert_eq!(overlay.filtered_sessions().len(), 1);
    }

    #[test]
    fn test_overlay_enum_toggle() {
        let mut overlay = OverlayState::new(vec!["default".to_string()]);
        assert!(overlay.current.is_none());

        overlay.toggle(Overlay::Help);
        assert_eq!(overlay.current, Some(Overlay::Help));

        overlay.toggle(Overlay::Help);
        assert!(overlay.current.is_none());

        overlay.show(Overlay::SessionList);
        assert_eq!(overlay.current, Some(Overlay::SessionList));

        overlay.show(Overlay::Config);
        assert_eq!(overlay.current, Some(Overlay::Config));
    }

    #[test]
    fn test_render_state_invalidation() {
        let mut rs = RenderState::new();
        rs.heights.push(10);
        rs.invalidate();
        assert!(rs.heights.is_empty());
        assert!(rs.dirty);
    }
}
