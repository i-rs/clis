pub struct InputState {
    pub content: String,
    pub cursor_pos: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    draft: String,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_pos: 0,
            history: Vec::new(),
            history_index: None,
            draft: String::new(),
        }
    }

    pub fn push_history(&mut self, text: &str) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.history.push(trimmed.to_string());
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }
        self.history_index = None;
        self.draft.clear();
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_index.is_none() {
            self.draft = self.content.clone();
            self.history_index = Some(self.history.len() - 1);
        } else if let Some(i) = self.history_index {
            if i > 0 {
                self.history_index = Some(i - 1);
            } else {
                return;
            }
        }
        let idx = self.history_index.unwrap();
        self.content = self.history[idx].clone();
        self.cursor_pos = self.content.len();
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            Some(i) if i + 1 < self.history.len() => {
                self.history_index = Some(i + 1);
                self.content = self.history[i + 1].clone();
                self.cursor_pos = self.content.len();
            }
            Some(_) => {
                self.history_index = None;
                self.content = std::mem::take(&mut self.draft);
                self.cursor_pos = self.content.len();
            }
            None => {}
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.history_index.is_some() {
            self.history_index = None;
            self.draft.clear();
        }
        self.content.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let len = self.content[..self.cursor_pos]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.cursor_pos -= len;
            self.content.remove(self.cursor_pos);
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor_pos < self.content.len() {
            let len = self.content[self.cursor_pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.content.drain(self.cursor_pos..self.cursor_pos + len);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            let len = self.content[..self.cursor_pos]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.cursor_pos -= len;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_pos < self.content.len() {
            let len = self.content[self.cursor_pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.cursor_pos += len;
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_pos = 0;
        self.history_index = None;
        self.draft.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_char_basic() {
        let mut input = InputState::new();
        input.insert_char('h');
        input.insert_char('i');
        assert_eq!(input.content, "hi");
        assert_eq!(input.cursor_pos, 2);
    }

    #[test]
    fn test_insert_char_middle() {
        let mut input = InputState::new();
        input.content = "ac".to_string();
        input.cursor_pos = 1;
        input.insert_char('b');
        assert_eq!(input.content, "abc");
        assert_eq!(input.cursor_pos, 2);
    }

    #[test]
    fn test_insert_char_multibyte() {
        let mut input = InputState::new();
        input.insert_char('中');
        input.insert_char('文');
        assert_eq!(input.content, "中文");
        assert_eq!(input.cursor_pos, 6);
    }

    #[test]
    fn test_delete_char_basic() {
        let mut input = InputState::new();
        input.content = "hello".to_string();
        input.cursor_pos = 5;
        input.delete_char();
        assert_eq!(input.content, "hell");
        assert_eq!(input.cursor_pos, 4);
    }

    #[test]
    fn test_delete_char_at_start_does_nothing() {
        let mut input = InputState::new();
        input.content = "hello".to_string();
        input.cursor_pos = 0;
        input.delete_char();
        assert_eq!(input.content, "hello");
        assert_eq!(input.cursor_pos, 0);
    }

    #[test]
    fn test_delete_char_multibyte() {
        let mut input = InputState::new();
        input.content = "中文".to_string();
        input.cursor_pos = 6;
        input.delete_char();
        assert_eq!(input.content, "中");
        assert_eq!(input.cursor_pos, 3);
    }

    #[test]
    fn test_delete_forward() {
        let mut input = InputState::new();
        input.content = "abc".to_string();
        input.cursor_pos = 1;
        input.delete_forward();
        assert_eq!(input.content, "ac");
        assert_eq!(input.cursor_pos, 1);
    }

    #[test]
    fn test_delete_forward_at_end_does_nothing() {
        let mut input = InputState::new();
        input.content = "abc".to_string();
        input.cursor_pos = 3;
        input.delete_forward();
        assert_eq!(input.content, "abc");
    }

    #[test]
    fn test_move_left_right() {
        let mut input = InputState::new();
        input.content = "abc".to_string();
        input.cursor_pos = 3;
        input.move_left();
        assert_eq!(input.cursor_pos, 2);
        input.move_left();
        assert_eq!(input.cursor_pos, 1);
        input.move_right();
        assert_eq!(input.cursor_pos, 2);
    }

    #[test]
    fn test_move_left_at_start_does_nothing() {
        let mut input = InputState::new();
        input.move_left();
        assert_eq!(input.cursor_pos, 0);
    }

    #[test]
    fn test_move_right_at_end_does_nothing() {
        let mut input = InputState::new();
        input.content = "x".to_string();
        input.cursor_pos = 1;
        input.move_right();
        assert_eq!(input.cursor_pos, 1);
    }

    #[test]
    fn test_move_left_multibyte() {
        let mut input = InputState::new();
        input.content = "中文".to_string();
        input.cursor_pos = 6;
        input.move_left();
        assert_eq!(input.cursor_pos, 3);
        input.move_left();
        assert_eq!(input.cursor_pos, 0);
    }

    #[test]
    fn test_clear() {
        let mut input = InputState::new();
        input.content = "hello".to_string();
        input.cursor_pos = 5;
        input.clear();
        assert_eq!(input.content, "");
        assert_eq!(input.cursor_pos, 0);
    }

    #[test]
    fn test_insert_after_delete() {
        let mut input = InputState::new();
        input.content = "abc".to_string();
        input.cursor_pos = 3;
        input.delete_char();
        assert_eq!(input.content, "ab");
        input.insert_char('d');
        assert_eq!(input.content, "abd");
        assert_eq!(input.cursor_pos, 3);
    }

    #[test]
    fn test_history_simple() {
        let mut input = InputState::new();
        input.push_history("hello");
        input.push_history("world");
        assert_eq!(input.history.len(), 2);
        input.history_up();
        assert_eq!(input.content, "world");
        input.history_up();
        assert_eq!(input.content, "hello");
        input.history_down();
        assert_eq!(input.content, "world");
        input.history_down();
        assert_eq!(input.content, "");
    }

    #[test]
    fn test_history_saves_draft() {
        let mut input = InputState::new();
        input.content = "current draft".to_string();
        input.cursor_pos = 13;
        input.push_history("previous prompt");
        input.history_up();
        assert_eq!(input.content, "previous prompt");
        input.history_down();
        assert_eq!(input.content, "current draft");
    }
}
