use crate::config::Config;
use crate::stats::TodaySummary;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Instant;

/// 按显示宽度估算行数（用于事件处理中粗略重算消息行高）。
/// ASCII=1，CJK/全角=2，不做 ANSI/word-break，仅作 scroll_to_selected 的兜底估算。
fn text_wrap_lines(text: &str, max_width: usize) -> usize {
    if max_width == 0 {
        return text.lines().count().max(1);
    }
    let mut total = 0usize;
    for line in text.split('\n') {
        let mut width = 0usize;
        let mut local_lines = 1;
        for c in line.chars() {
            let cw = if (c as u32) < 0x1100 || (c as u32) == 0x2E3A { 1 } else { 2 };
            if width + cw > max_width {
                local_lines += 1;
                width = cw;
            } else {
                width += cw;
            }
        }
        total += local_lines;
    }
    total.max(1)
}

pub fn message_to_jsonl(msg: &Message) -> Value {
    serde_json::to_value(msg)
        .unwrap_or_else(|_| serde_json::json!({"type": "error", "text": "serialization failed"}))
}

pub fn message_from_jsonl(v: Value) -> Option<Message> {
    serde_json::from_value(v).ok()
}

pub fn evaluate_response_heuristic(
    response_text: &str,
    tool_results: &[(&str, bool)],
    known_tools: &[&str],
) -> Message {
    let mut issues = Vec::new();
    let mut references_valid = 0u32;

    let executed_tools: HashSet<&str> = tool_results.iter().map(|(n, _)| *n).collect();
    for (name, success) in tool_results {
        if response_text.contains(*name) {
            references_valid += 1;
        }
        if !(*success || response_text.contains("错误") || response_text.contains("失败")) {
            issues.push(format!("工具 '{}' 执行失败，但回复未提及", name));
        }
    }

    for pattern in known_tools {
        if response_text.contains(*pattern)
            && !executed_tools.contains(pattern)
            && response_text.contains("i-rs")
        {
            issues.push(format!("回复提及未执行的工具: {}", pattern));
        }
    }

    let complete = !response_text.trim().is_empty();
    if !complete {
        issues.push("回复为空".to_string());
    }

    let relevance = compute_text_relevance(response_text);
    if relevance < 0.3 && !response_text.is_empty() {
        issues.push(format!(
            "回复信息密度较低 (相关度: {:.0}%)",
            relevance * 100.0
        ));
    }

    let has_errors = !issues.is_empty();
    let score = if has_errors {
        Some(1.0 - (issues.len() as f64 * 0.2).min(0.8))
    } else if complete {
        Some(1.0)
    } else {
        Some(0.0)
    };

    Message::Quality {
        score,
        complete,
        references_valid,
        issues,
    }
}

fn compute_text_relevance(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let mut meaningful = 0usize;
    let mut total_chars = 0usize;
    for c in text.chars() {
        total_chars += 1;
        if c.is_alphanumeric() || c > '\x7f' {
            meaningful += 1;
        }
    }
    if total_chars == 0 {
        return 0.0;
    }

    static STOPWORDS: &[&str] = &[
        "的", "了", "在", "是", "我", "有", "和", "就", "不", "都", "the", "a", "an", "is", "are",
        "was", "were", "be", "been", "have", "has", "had", "do", "does", "did", "will", "would",
        "could", "should", "may", "might", "can", "shall", "to", "of", "in", "for", "on", "with",
        "at", "by", "from", "as", "into", "through", "during", "before", "after", "above", "below",
        "between", "and", "but", "or", "nor", "not", "so", "yet", "both", "either", "each",
        "every", "all", "any", "few", "more", "most", "other", "some", "such", "only", "own",
        "same", "than", "too", "very", "just", "because", "about", "up", "out", "if", "then",
        "now", "it", "its", "he", "she", "they", "them", "this", "that", "these", "those", "what",
        "which", "who", "whom", "how",
    ];

    let mut stopword_count = 0usize;
    let mut total_words = 0usize;
    for w in text.split_whitespace() {
        total_words += 1;
        if w.len() <= 4 && STOPWORDS.iter().any(|&s| w.eq_ignore_ascii_case(s) || w.contains(s)) {
            stopword_count += 1;
        }
    }

    let meaningful_ratio = meaningful as f64 / total_chars as f64;
    let stopword_ratio = stopword_count as f64 / total_words.max(1) as f64;

    (meaningful_ratio * 0.6 + (1.0 - stopword_ratio) * 0.4).min(1.0)
}

#[derive(Clone)]
pub struct PluginEntry {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub description: String,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct HttpLog {
    pub timestamp: String,
    pub status: u16,
    pub duration_ms: u64,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub error: Option<String>,
    pub request_body: String,
    pub msg_count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    User {
        text: String,
    },
    Assistant {
        text: String,
        #[serde(default)]
        reasoning: String,
    },
    ToolCall {
        name: String,
        args: String,
        result: String,
        step: usize,
        total_steps: usize,
    },
    Error {
        text: String,
    },
    Evaluation {
        tool: String,
        valid: bool,
        issues: Vec<String>,
    },
    Quality {
        score: Option<f64>,
        complete: bool,
        references_valid: u32,
        issues: Vec<String>,
    },
    Feedback {
        positive: bool,
        message: Option<String>,
    },
    Image {
        path: String,
        alt_text: String,
        width: u32,
        height: u32,
        format: String,
    },
}

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
        let line_start = self.text[..self.cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
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
    SlashCommand { name: "/help",    desc: "快捷键帮助",       shortcut: "Ctrl+H",       action: SlashAction::Help },
    SlashCommand { name: "/sessions",desc: "会话列表",         shortcut: "Ctrl+L",       action: SlashAction::Sessions },
    SlashCommand { name: "/new",     desc: "新建会话",         shortcut: "Ctrl+N",       action: SlashAction::New },
    SlashCommand { name: "/agent",   desc: "切换 Agent",       shortcut: "Ctrl+P",       action: SlashAction::Agent },
    SlashCommand { name: "/agents",  desc: "Agent 管理",       shortcut: "Ctrl+A",       action: SlashAction::Agents },
    SlashCommand { name: "/tools",   desc: "工具列表",         shortcut: "Ctrl+T",       action: SlashAction::Tools },
    SlashCommand { name: "/sidebar", desc: "HTTP 调试面板",    shortcut: "Ctrl+R",       action: SlashAction::Sidebar },
    SlashCommand { name: "/stats",   desc: "Token 用量统计",   shortcut: "Ctrl+Shift+U", action: SlashAction::Stats },
    SlashCommand { name: "/plugins", desc: "插件 & 技能",      shortcut: "Ctrl+Shift+P", action: SlashAction::Plugins },
    SlashCommand { name: "/config",  desc: "配置信息",         shortcut: "Ctrl+I",       action: SlashAction::Config },
    SlashCommand { name: "/export",  desc: "导出会话",         shortcut: "Ctrl+E",       action: SlashAction::Export },
    SlashCommand { name: "/feedback",desc: "发送反馈",         shortcut: "Ctrl+F",       action: SlashAction::Feedback },
    SlashCommand { name: "/info",    desc: "状态仪表盘",       shortcut: "Ctrl+Shift+I", action: SlashAction::Info },
    SlashCommand { name: "/select",  desc: "选择模式",         shortcut: "Ctrl+S",       action: SlashAction::Select },
    SlashCommand { name: "/clear",   desc: "清空当前会话",     shortcut: "",             action: SlashAction::Clear },
    SlashCommand { name: "/compact", desc: "压缩上下文",       shortcut: "",             action: SlashAction::Compact },
    SlashCommand { name: "/theme",   desc: "切换主题配色",     shortcut: "",             action: SlashAction::Theme },
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
    pub session_list: Vec<crate::session::SessionMeta>,
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
    pub tool_call_expanded: HashSet<usize>,
    pub reasoning_expanded: HashSet<usize>,
    pub copy_feedback: Option<(String, std::time::Instant)>,
    pub tab_completions: Vec<String>,
    pub tab_completion_index: usize,
    pub slash_visible: bool,
    pub slash_index: usize,
    pub theme_index: usize,
    pub cached_filtered_sessions: Option<Vec<crate::session::SessionMeta>>,
    pub cached_search_hash: u64,
}

impl OverlayState {
    pub fn filtered_sessions(&self) -> Vec<crate::session::SessionMeta> {
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

    pub fn filtered_sessions_cached(&mut self) -> Vec<crate::session::SessionMeta> {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.session_search.hash(&mut hasher);
        self.session_list.len().hash(&mut hasher);
        for s in &self.session_list {
            s.title.hash(&mut hasher);
        }
        let hash = hasher.finish();
        if self.cached_search_hash == hash && let Some(ref cached) = self.cached_filtered_sessions {
            return cached.clone();
        }
        let filtered = self.filtered_sessions();
        self.cached_search_hash = hash;
        self.cached_filtered_sessions = Some(filtered.clone());
        filtered
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
            tool_call_expanded: HashSet::new(),
            reasoning_expanded: HashSet::new(),
            copy_feedback: None,
            tab_completions: Vec::new(),
            tab_completion_index: 0,
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
    pub format_cache: HashMap<usize, Arc<Vec<ratatui::text::Line<'static>>>>,
    pub cached_width: usize,
    pub chat_height: u16,
    pub dirty: bool,
    pub last_drawn_at: Option<Instant>,
    pub tool_call_headers: HashMap<usize, (String, Option<String>)>,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            heights: Vec::new(),
            format_cache: HashMap::new(),
            cached_width: 0,
            chat_height: 0,
            dirty: true,
            last_drawn_at: None,
            tool_call_headers: HashMap::new(),
        }
    }

    pub fn invalidate(&mut self) {
        self.heights.clear();
        self.dirty = true;
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

pub struct App {
    pub messages: Vec<Message>,
    pub message_timestamps: Vec<NaiveDateTime>,
    pub input: InputState,
    pub overlay: OverlayState,
    pub state: AppState,
    pub config: Config,
    pub tool_call_count: usize,
    pub status_text: String,
    pub api_messages: Option<Vec<Value>>,
    pub token_usage: Option<crate::llm::TokenUsage>,
    pub scroll_lines: usize,
    pub max_scroll: usize,
    pub component_offsets: Vec<u16>,
    pub component_total_height: usize,
    pub http_logs: VecDeque<HttpLog>,
    pub current_reasoning: String,
    pub reminder_text: Option<String>,
    pub plan_steps: Vec<PlanStep>,
    pub current_agent: String,
    pub today_stats: TodaySummary,
    pub stats_history: Vec<crate::stats::DailyStats>,
    pub skill_list: Vec<crate::skill_store::SkillEntry>,
    pub plugin_list: Vec<PluginEntry>,
    pub spinner_start: Instant,
    pub render_state: RenderState,
}

impl App {
    pub fn new(config: Config) -> Self {
        let agent_list = config.agent_ids();

        Self {
            messages: Vec::new(),
            message_timestamps: Vec::new(),
            input: InputState::new(),
            overlay: OverlayState::new(agent_list),
            state: AppState::Idle,
            config,
            tool_call_count: 0,
            status_text: String::new(),
            api_messages: None,
            token_usage: None,
            scroll_lines: 0,
            max_scroll: 0,
            component_offsets: Vec::new(),
            component_total_height: 0,
            http_logs: VecDeque::new(),
            current_reasoning: String::new(),
            reminder_text: None,
            plan_steps: Vec::new(),
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
        matches!(self.state, AppState::Processing)
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
        self.messages.push(Message::User {
            text: text.to_string(),
        });
        self.message_timestamps
            .push(chrono::Local::now().naive_local());
        self.state = AppState::Processing;
        self.scroll_lines = 0;
        self.plan_steps.clear();
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
        self.scroll_lines = 0;
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
    pub fn scroll_up(&mut self) {
        self.scroll_lines = self.scroll_lines.saturating_add(3);
        if self.max_scroll > 0 {
            self.scroll_lines = self.scroll_lines.min(self.max_scroll);
        }
    }

    /// Trackpad-optimized scroll: 3 lines per event for smooth macOS two-finger scrolling.
    pub fn scroll_down(&mut self) {
        self.scroll_lines = self.scroll_lines.saturating_sub(3);
    }

    /// 让选中的消息滚入视口。若已在视口内则保持滚动位置不变。
    /// 依据 render_state.heights 估算每个消息行高；
    /// 若缓存为空（如首屏未渲染）则放弃调整。
    ///
    /// heights 约定：heights[0] = 最新消息，heights[len-1] = 最旧消息（与 render_chat 一致）。
    /// scroll_lines 从底部计数：0 = 底部（最新），递增 = 向上（更旧）。
    pub fn scroll_to_selected(&mut self) {
        let Some(idx) = self.overlay.selected_message else { return };
        let heights = &self.render_state.heights;
        if heights.is_empty() || idx >= heights.len() {
            return;
        }
        // 将前向索引 (messages[idx]) 转为逆序索引 (heights 中的位置)
        let rev_idx = heights.len() - 1 - idx;
        // sel_top = 选中消息顶端距底部的行数（即所有比它更新的消息总高度）
        let sel_top: usize = heights.iter().take(rev_idx).sum();
        let sel_height = heights[rev_idx];
        let sel_bottom = sel_top + sel_height;
        // 用渲染时回填的 chat 高度算视口行数，避免依赖 max_scroll（展开/折叠后滞后）
        let area_lines = (self.render_state.chat_height as usize).saturating_sub(1).max(1);
        let viewport_top = self.scroll_lines;
        let viewport_bottom = self.scroll_lines.saturating_add(area_lines);
        // max_scroll 也要用最新的 heights 重新计算（max_scroll 是渲染时存的，旧值会错）
        let total: usize = heights.iter().sum();
        let new_max_scroll = total.saturating_sub(area_lines);

        if sel_top < viewport_top || sel_bottom > viewport_bottom {
            // 选中的不在视口内 → 居中对齐：让选中条落在视口中央，
            // 既保证选中条可见，又让用户能感知上下文。
            self.scroll_lines = sel_top.saturating_sub(area_lines / 2);
        }
        // 夹到合法范围
        self.scroll_lines = self.scroll_lines.min(new_max_scroll);
        self.max_scroll = new_max_scroll;
    }

    /// 事件处理中调用：在 mark_dirty 清空 heights 后，用消息文本长度
    /// 粗略估算每条消息行高，让 scroll_to_selected 在展开/折叠后
    /// 不必等下一次渲染就能算出正确的视口位置。
    /// 渲染时会基于 chat.rs 的真实 layout 重新精修 heights。
    ///
    /// heights 约定：逆序存储（heights[0] = 最新消息），与 render_chat 一致。
    pub fn rebuild_heights_approx(&mut self) {
        let text_width = self.render_state.cached_width.max(20);
        let n = self.messages.len();
        let mut heights: Vec<usize> = Vec::with_capacity(n);
        // 逆序遍历以匹配 render_chat 的 heights 约定
        for (rev_idx, msg) in self.messages.iter().rev().enumerate() {
            let fwd_idx = n.saturating_sub(1).saturating_sub(rev_idx);
            let body = match msg {
                Message::User { text } => 1 + text_wrap_lines(text, text_width) + 1,
                Message::Assistant { text, reasoning } => {
                    let mut extra = 0;
                    if !reasoning.is_empty() {
                        extra += 1;
                        if self.overlay.reasoning_expanded.contains(&fwd_idx) {
                            extra += reasoning.lines().count();
                        }
                    }
                    1 + text_wrap_lines(text, text_width) + 1 + extra
                }
                Message::ToolCall { result, .. } => {
                    if self.overlay.tool_call_expanded.contains(&fwd_idx) {
                        let mut lines = 1;
                        if !result.is_empty() {
                            lines += text_wrap_lines(result, text_width.saturating_sub(3));
                        }
                        lines
                    } else {
                        1
                    }
                }
                Message::Error { text } => 1 + text_wrap_lines(text, text_width) + 1,
                Message::Image { .. } => 3,
                Message::Quality { issues, .. } => 3 + issues.len(),
                Message::Feedback { .. } => 2,
                _ => 1,
            };
            heights.push(body.max(1));
        }
        self.render_state.heights = heights;
        // 同步 max_scroll 给主渲染用，避免短暂不一致
        let area_lines = (self.render_state.chat_height as usize).saturating_sub(1).max(1);
        let total: usize = self.render_state.heights.iter().sum();
        self.max_scroll = total.saturating_sub(area_lines);
    }

    pub fn set_status(&mut self, text: &str) {
        self.status_text = text.to_string();
    }

    pub fn start_assistant_message(&mut self) {
        if !self.current_reasoning.is_empty()
            && let Some(Message::Assistant { reasoning, .. }) = self.messages.last_mut()
        {
            reasoning.push_str(&self.current_reasoning);
        }
        self.current_reasoning.clear();
        let is_empty_assistant = matches!(
            self.messages.last(),
            Some(Message::Assistant { text, .. }) if text.is_empty()
        );
        if !is_empty_assistant {
            self.messages.push(Message::Assistant {
                text: String::new(),
                reasoning: String::new(),
            });
            self.message_timestamps
                .push(chrono::Local::now().naive_local());
            self.mark_dirty();
        }
    }

    pub fn append_assistant_text(&mut self, text: &str) {
        let last_is_assistant = matches!(self.messages.last_mut(), Some(Message::Assistant { .. }));
        if !last_is_assistant {
            self.start_assistant_message();
        }
        if let Some(Message::Assistant { text: t, .. }) = self.messages.last_mut() {
            t.push_str(text);
            self.render_state.format_cache.remove(&(self.messages.len() - 1));
            self.render_state.invalidate_last();
        }
    }

    pub fn add_tool_call(
        &mut self,
        name: &str,
        args: &str,
        result: &str,
        step: usize,
        total_steps: usize,
    ) {
        self.messages.push(Message::ToolCall {
            name: name.to_string(),
            args: args.to_string(),
            result: result.to_string(),
            step,
            total_steps,
        });
        self.message_timestamps
            .push(chrono::Local::now().naive_local());
        self.tool_call_count += 1;
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
        if !self.current_reasoning.is_empty()
            && let Some(Message::Assistant { reasoning, .. }) = self.messages.last_mut()
        {
            reasoning.push_str(&self.current_reasoning);
        }
        self.current_reasoning.clear();
        if let Some(Message::Assistant { text, .. }) = self.messages.last()
            && text.is_empty()
        {
            self.messages.pop();
            self.message_timestamps.pop();
        }
        self.messages.push(Message::Error {
            text: text.to_string(),
        });
        self.message_timestamps
            .push(chrono::Local::now().naive_local());
        self.api_messages = None;
        self.state = AppState::Idle;
        self.status_text.clear();
        self.mark_dirty();
    }

    pub fn finish_processing(&mut self, api_messages: Option<Vec<Value>>) {
        if !self.current_reasoning.is_empty()
            && let Some(Message::Assistant { reasoning, .. }) = self.messages.last_mut()
        {
            reasoning.push_str(&self.current_reasoning);
        }
        self.current_reasoning.clear();
        if let Some(Message::Assistant { text, .. }) = self.messages.last()
            && text.is_empty()
        {
            self.messages.pop();
            self.message_timestamps.pop();
        }
        self.api_messages = api_messages;
        self.state = AppState::Idle;
        self.status_text.clear();
        self.mark_dirty();
    }

    pub fn detect_plan(&mut self, text: &str) {
        if self.is_processing() {
            self.plan_steps.clear();
            for line in text.lines() {
                let trimmed = line.trim();
                let rest = trimmed
                    .strip_prefix(|c: char| c.is_ascii_digit())
                    .unwrap_or("");
                let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
                if let Some(rest) = rest.strip_prefix(". ") {
                    let clean = rest.trim_end_matches(['.', '，', ',']);
                    if !clean.is_empty() {
                        self.plan_steps.push(PlanStep {
                            description: clean.to_string(),
                            done: false,
                        });
                    }
                }
            }
        }
    }

    pub fn mark_next_plan_step_done(&mut self) {
        for step in &mut self.plan_steps {
            if !step.done {
                step.done = true;
                break;
            }
        }
    }

    pub fn reset_for_new_session(&mut self) {
        self.messages.clear();
        self.message_timestamps.clear();
        self.api_messages = None;
        self.state = AppState::Idle;
        self.tool_call_count = 0;
        self.status_text.clear();
        self.token_usage = None;
        self.input = InputState::new();
        self.overlay.current = None;
        self.http_logs.clear();
        self.overlay.sidebar_selected = 0;
        self.overlay.sidebar_body_idx = None;
        self.overlay.sidebar_body_scroll = 0;
        self.plan_steps.clear();
        self.overlay.session_search.clear();
        self.overlay.session_search_mode = false;
        self.overlay.selected_message = None;
        self.overlay.selection_mode = false;
        self.overlay.tool_call_expanded.clear();
        self.overlay.reasoning_expanded.clear();
        self.mark_dirty();
    }

    pub fn sync_message_timestamps(&mut self) {
        let now = chrono::Local::now().naive_local();
        while self.message_timestamps.len() < self.messages.len() {
            self.message_timestamps.push(now);
        }
        if self.message_timestamps.len() > self.messages.len() {
            self.message_timestamps.truncate(self.messages.len());
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
        assert!(app.messages.is_empty());
        assert_eq!(app.state, AppState::Idle);
        assert!(!app.is_processing());
        assert!(app.http_logs.is_empty());
        assert_eq!(app.current_agent, "default");
        assert_eq!(app.tool_call_count, 0);
    }

    #[test]
    fn test_add_user_message_sets_processing() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(app.messages[0], Message::User { ref text } if text == "hello"));
        assert!(app.is_processing());
        assert_eq!(app.scroll_lines, 0);
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
        assert_eq!(app.state, AppState::Idle);
        assert!(app.status_text.is_empty());
        assert!(app.api_messages.is_some());
    }

    #[test]
    fn test_finish_processing_removes_empty_assistant() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();
        assert_eq!(app.messages.len(), 2);
        app.finish_processing(None);
        assert_eq!(app.messages.len(), 1);
        assert!(!app.is_processing());
    }

    #[test]
    fn test_add_error_sets_idle() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        assert!(app.is_processing());

        app.add_error("something went wrong");
        assert!(!app.is_processing());
        assert_eq!(app.messages.len(), 2);
        assert!(
            matches!(app.messages[1], Message::Error { ref text } if text == "something went wrong")
        );
        assert!(app.api_messages.is_none());
    }

    #[test]
    fn test_add_error_removes_empty_assistant() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();
        assert_eq!(app.messages.len(), 2);
        app.add_error("err");
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(app.messages[1], Message::Error { .. }));
    }

    #[test]
    fn test_assistant_message_append() {
        let mut app = App::new(test_config());
        app.start_assistant_message();
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(app.messages[0], Message::Assistant { ref text, .. } if text.is_empty()));

        app.append_assistant_text("hello ");
        app.append_assistant_text("world");
        assert!(
            matches!(app.messages[0], Message::Assistant { ref text, .. } if text == "hello world")
        );
    }

    #[test]
    fn test_append_assistant_reuses_empty() {
        let mut app = App::new(test_config());
        app.append_assistant_text("direct");
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(app.messages[0], Message::Assistant { ref text, .. } if text == "direct"));
    }

    #[test]
    fn test_add_tool_call() {
        let mut app = App::new(test_config());
        app.add_tool_call("weight", r#"{"action":"list"}"#, "OK", 1, 2);
        assert_eq!(app.messages.len(), 1);
        assert!(
            matches!(&app.messages[0], Message::ToolCall { name, step: 1, total_steps: 2, .. } if name == "weight")
        );
        assert_eq!(app.tool_call_count, 1);
    }

    #[test]
     fn test_scroll() {
        let mut app = App::new(test_config());
        assert_eq!(app.scroll_lines, 0);
        app.scroll_up();
        assert_eq!(app.scroll_lines, 3);
        app.scroll_up();
        assert_eq!(app.scroll_lines, 6);
        app.scroll_down();
        assert_eq!(app.scroll_lines, 3);
        app.scroll_down();
        assert_eq!(app.scroll_lines, 0);
        app.scroll_down();
        assert_eq!(app.scroll_lines, 0);
        app.max_scroll = 5;
        app.scroll_up();
        assert_eq!(app.scroll_lines, 3);
        app.scroll_up();
        assert_eq!(app.scroll_lines, 5); // clamped to max_scroll
    }

    #[test]
    fn test_scroll_to_selected_newest_visible_at_bottom() {
        // 10 条消息各占 3 行，总 30 行，area_lines = 10，max_scroll = 20
        // heights 逆序：heights[0]=最新(m9), heights[9]=最旧(m0)
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.max_scroll = 20;
        // scroll_lines=0（底部），选中 idx=9（最新）→ sel_top=0 → 在视口内 → 不变
        app.overlay.selected_message = Some(9);
        app.scroll_to_selected();
        assert_eq!(app.scroll_lines, 0);
    }

    #[test]
    fn test_scroll_to_selected_oldest_not_visible_at_bottom() {
        // scroll_lines=0（底部），选中 idx=0（最旧）→ 需要向上滚动
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.max_scroll = 20;
        app.scroll_lines = 0;
        app.overlay.selected_message = Some(0);
        app.scroll_to_selected();
        // rev_idx=9, sel_top=27, 居中: 27-5=22, clamp 到 20
        assert_eq!(app.scroll_lines, 20);
    }

    #[test]
    fn test_scroll_to_selected_oldest_visible_at_top() {
        // scroll_lines=20（顶部），选中 idx=0（最旧）→ 在视口内 → 不变
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.max_scroll = 20;
        app.scroll_lines = 20;
        app.overlay.selected_message = Some(0);
        app.scroll_to_selected();
        // rev_idx=9, sel_top=27, sel_bottom=30, viewport=[20,30) → 可见 → 不变
        assert_eq!(app.scroll_lines, 20);
    }

    #[test]
    fn test_scroll_to_selected_newest_not_visible_at_top() {
        // scroll_lines=20（顶部），选中 idx=9（最新）→ 需要向下滚动
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.max_scroll = 20;
        app.scroll_lines = 20;
        app.overlay.selected_message = Some(9);
        app.scroll_to_selected();
        // rev_idx=0, sel_top=0, 居中: 0-5=0 (saturating)
        assert_eq!(app.scroll_lines, 0);
        assert_eq!(app.max_scroll, 20);
    }

    #[test]
    fn test_scroll_to_selected_mid_visible_no_change() {
        // scroll_lines=10，选中 idx=5 → 检查是否在视口内
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.render_state.chat_height = 11;
        app.max_scroll = 20;
        app.scroll_lines = 10;
        app.overlay.selected_message = Some(5);
        app.scroll_to_selected();
        // rev_idx=4, sel_top=12, sel_bottom=15, viewport=[10,20) → 12>=10 && 15<=20 → 可见
        assert_eq!(app.scroll_lines, 10);
    }

    #[test]
    fn test_scroll_to_selected_empty_heights() {
        let mut app = App::new(test_config());
        app.overlay.selected_message = Some(0);
        app.scroll_to_selected();
        assert_eq!(app.scroll_lines, 0);
    }

    #[test]
    fn test_scroll_to_selected_none_selected() {
        let mut app = App::new(test_config());
        app.render_state.heights = vec![3; 10];
        app.scroll_lines = 5;
        app.scroll_to_selected();
        assert_eq!(app.scroll_lines, 5);
    }

    #[test]
    fn test_set_status() {
        let mut app = App::new(test_config());
        assert!(app.status_text.is_empty());
        app.set_status("thinking…");
        assert_eq!(app.status_text, "thinking…");
    }

    #[test]
    fn test_reset_for_new_session() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.add_tool_call("test", "{}", "ok", 1, 1);
        app.set_status("done");
        app.reset_for_new_session();
        assert!(app.messages.is_empty());
        assert!(app.message_timestamps.is_empty());
        assert!(app.api_messages.is_none());
        assert_eq!(app.tool_call_count, 0);
        assert!(app.status_text.is_empty());
        assert!(app.http_logs.is_empty());
        assert!(app.plan_steps.is_empty());
    }

    #[test]
    fn test_mark_overlay_dirty_preserves_chat_cache() {
        let mut app = App::new(test_config());
        app.render_state.dirty = false;
        app.render_state.heights.push(3);
        app.render_state
            .format_cache
            .insert(0, std::sync::Arc::new(vec![]));

        app.mark_overlay_dirty();

        assert!(app.render_state.dirty);
        assert_eq!(app.render_state.heights.len(), 1);
        assert_eq!(app.render_state.format_cache.len(), 1);
    }

    #[test]
    fn test_mark_dirty_clears_heights_not_cache() {
        let mut app = App::new(test_config());
        app.render_state.dirty = false;
        app.render_state.heights.push(3);
        app.render_state
            .format_cache
            .insert(0, std::sync::Arc::new(vec![]));

        app.mark_dirty();

        assert!(app.render_state.dirty);
        assert!(app.render_state.heights.is_empty());
        assert!(!app.render_state.format_cache.is_empty());
    }

    #[test]
    fn test_detect_plan() {
        let mut app = App::new(test_config());
        app.add_user_message("plan something");
        app.detect_plan("1. first step.\n2. second step，\n3. third step.");
        assert_eq!(app.plan_steps.len(), 3);
        assert_eq!(app.plan_steps[0].description, "first step");
        assert!(!app.plan_steps[0].done);
        assert_eq!(app.plan_steps[2].description, "third step");
    }

    #[test]
    fn test_detect_plan_ignores_non_numbered() {
        let mut app = App::new(test_config());
        app.add_user_message("hi");
        app.detect_plan("- bullet item\nplain text\n1. actual step");
        assert_eq!(app.plan_steps.len(), 1);
        assert_eq!(app.plan_steps[0].description, "actual step");
    }

    #[test]
    fn test_detect_plan_only_when_processing() {
        let mut app = App::new(test_config());
        app.detect_plan("1. first step");
        assert!(app.plan_steps.is_empty());
    }

    #[test]
    fn test_mark_next_plan_step_done() {
        let mut app = App::new(test_config());
        app.add_user_message("do it");
        app.detect_plan("1. step A\n2. step B");
        app.mark_next_plan_step_done();
        assert!(app.plan_steps[0].done);
        assert!(!app.plan_steps[1].done);
        app.mark_next_plan_step_done();
        assert!(app.plan_steps[1].done);
    }

    #[test]
    fn test_sync_message_timestamps_fills_gaps() {
        let mut app = App::new(test_config());
        app.messages.push(Message::User {
            text: "a".to_string(),
        });
        app.messages.push(Message::User {
            text: "b".to_string(),
        });
        assert!(app.message_timestamps.is_empty());
        app.sync_message_timestamps();
        assert_eq!(app.message_timestamps.len(), 2);
    }

    #[test]
    fn test_sync_message_timestamps_truncates_excess() {
        let mut app = App::new(test_config());
        app.messages.push(Message::User {
            text: "a".to_string(),
        });
        app.message_timestamps
            .push(chrono::Local::now().naive_local());
        app.message_timestamps
            .push(chrono::Local::now().naive_local());
        app.sync_message_timestamps();
        assert_eq!(app.message_timestamps.len(), 1);
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
            crate::session::SessionMeta {
                id: "1".to_string(),
                title: "Weight tracking".to_string(),
                agent_id: "default".to_string(),
                state: crate::session::SessionState::Active,
                created_at: 0,
                updated_at: 0,
                message_count: 0,
            },
            crate::session::SessionMeta {
                id: "2".to_string(),
                title: "Mood log".to_string(),
                agent_id: "default".to_string(),
                state: crate::session::SessionState::Active,
                created_at: 0,
                updated_at: 0,
                message_count: 0,
            },
            crate::session::SessionMeta {
                id: "3".to_string(),
                title: "Weight history".to_string(),
                agent_id: "default".to_string(),
                state: crate::session::SessionState::Active,
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
        overlay.session_list = vec![crate::session::SessionMeta {
            id: "1".to_string(),
            title: "Weight Tracking".to_string(),
            agent_id: "default".to_string(),
            state: crate::session::SessionState::Active,
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
        rs.format_cache.insert(0, Arc::new(vec![]));
        rs.invalidate();
        assert!(rs.heights.is_empty());
        assert!(rs.dirty);
        assert!(!rs.format_cache.is_empty());
    }
}
