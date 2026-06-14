use std::time::Instant;

pub const MAX_INPUT_LEN: usize = 64 * 1024;

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

    pub fn remaining_capacity(&self) -> usize {
        MAX_INPUT_LEN.saturating_sub(self.text.len())
    }

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
