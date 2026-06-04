use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::builders::indent_line;

pub(super) fn has_ansi(text: &str) -> bool {
    text.contains("\x1b[")
}

pub(super) fn ansi_to_lines(text: &str, max_width: usize) -> Vec<Line<'static>> {
    let plain = strip_ansi(text);
    let wrapped = crate::ui::utils::wrap_text(&plain, max_width.saturating_sub(3));
    let mut lines = Vec::new();
    let mut plain_offset = 0usize;
    for w in &wrapped {
        let line_end = plain_offset + w.len();
        let spans = parse_ansi_line_at(text, plain_offset, line_end);
        if spans.is_empty() {
            lines.push(indent_line(w, Style::default().fg(Color::White)));
        } else {
            let mut result = vec![Span::raw("   ")];
            result.extend(spans);
            lines.push(Line::from(result));
        }
        plain_offset = line_end;
    }
    lines
}

pub(super) fn strip_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let b = bytes[i];
                i += 1;
                if b.is_ascii_alphabetic() || b == b'~' {
                    break;
                }
            }
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != 0x1b {
                i += 1;
            }
            result.push_str(&text[start..i]);
        }
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AnsiState {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    dim: bool,
}

impl AnsiState {
    fn reset(&mut self) {
        self.fg = None;
        self.bg = None;
        self.bold = false;
        self.italic = false;
        self.underline = false;
        self.dim = false;
    }
    pub(super) fn to_style(self) -> Style {
        let mut s = Style::default();
        if self.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        if self.italic {
            s = s.add_modifier(Modifier::ITALIC);
        }
        if self.underline {
            s = s.add_modifier(Modifier::UNDERLINED);
        }
        if self.dim {
            s = s.add_modifier(Modifier::DIM);
        }
        if let Some(c) = self.fg {
            s = s.fg(c);
        }
        if let Some(c) = self.bg {
            s = s.bg(c);
        }
        s
    }
}

fn parse_ansi_line_at(raw: &str, line_start: usize, line_end: usize) -> Vec<Span<'static>> {
    let raw_bytes = raw.as_bytes();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut state = AnsiState {
        fg: None,
        bg: None,
        bold: false,
        italic: false,
        underline: false,
        dim: false,
    };
    let mut cur_text = String::new();
    let mut raw_i = 0;
    let mut plain_i = 0;

    while raw_i < raw_bytes.len() && plain_i < line_end {
        if raw_bytes[raw_i] == 0x1b && raw_i + 1 < raw_bytes.len() && raw_bytes[raw_i + 1] == b'[' {
            if !cur_text.is_empty() {
                spans.push(Span::styled(
                    std::mem::take(&mut cur_text),
                    state.to_style(),
                ));
            }
            raw_i += 2;
            let (next_raw, next_state) = parse_sgr(raw_bytes, raw_i, state);
            raw_i = next_raw;
            state = next_state;
        } else {
            let c = raw[raw_i..].chars().next().unwrap_or('\0');
            let c_len = c.len_utf8();
            raw_i += c_len;
            if plain_i >= line_start && plain_i < line_end {
                cur_text.push(c);
            }
            plain_i += c_len;
        }
    }
    if !cur_text.is_empty() {
        spans.push(Span::styled(cur_text, state.to_style()));
    }
    spans
}

fn parse_sgr(bytes: &[u8], start: usize, mut state: AnsiState) -> (usize, AnsiState) {
    let mut i = start;
    let mut params = [0i32; 8];
    let mut param_count = 0usize;
    let mut num_buf = 0i32;
    let mut has_num = false;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b';' || b == b':' {
            if has_num && param_count < params.len() {
                params[param_count] = num_buf;
                param_count += 1;
            }
            num_buf = 0;
            has_num = false;
            i += 1;
        } else if b == b'm' {
            if has_num && param_count < params.len() {
                params[param_count] = num_buf;
                param_count += 1;
            } else if param_count == 0 {
                params[0] = 0;
                param_count = 1;
            }
            i += 1;
            break;
        } else if b.is_ascii_digit() {
            num_buf = num_buf * 10 + (b - b'0') as i32;
            has_num = true;
            i += 1;
        } else {
            let mut j = i;
            while j < bytes.len() && bytes[j] != b'm' {
                j += 1;
            }
            i = if j < bytes.len() { j + 1 } else { j };
            break;
        }
    }
    apply_sgr_params(&params[..param_count], &mut state);
    (i, state)
}

fn apply_sgr_params(params: &[i32], state: &mut AnsiState) {
    let mut pi = 0;
    while pi < params.len() {
        match params[pi] {
            0 => state.reset(),
            1 => {
                state.bold = true;
                state.dim = false;
            }
            2 => state.dim = true,
            3 => state.italic = true,
            4 => state.underline = true,
            22 => {
                state.bold = false;
                state.dim = false;
            }
            23 => state.italic = false,
            24 => state.underline = false,
            30 => state.fg = Some(Color::Black),
            31 => state.fg = Some(Color::Red),
            32 => state.fg = Some(Color::Green),
            33 => state.fg = Some(Color::Yellow),
            34 => state.fg = Some(Color::Blue),
            35 => state.fg = Some(Color::Magenta),
            36 => state.fg = Some(Color::Cyan),
            37 => state.fg = Some(Color::White),
            38 if pi + 2 < params.len() => match params[pi + 1] {
                2 if pi + 4 < params.len() => {
                    state.fg = Some(Color::Rgb(
                        params[pi + 2].clamp(0, 255) as u8,
                        params[pi + 3].clamp(0, 255) as u8,
                        params[pi + 4].clamp(0, 255) as u8,
                    ));
                    pi += 4;
                }
                5 if pi + 2 < params.len() => {
                    state.fg = Some(indexed_color(params[pi + 2]));
                    pi += 2;
                }
                _ => {}
            },
            39 => state.fg = None,
            40 => state.bg = Some(Color::Black),
            41 => state.bg = Some(Color::Red),
            42 => state.bg = Some(Color::Green),
            43 => state.bg = Some(Color::Yellow),
            44 => state.bg = Some(Color::Blue),
            45 => state.bg = Some(Color::Magenta),
            46 => state.bg = Some(Color::Cyan),
            47 => state.bg = Some(Color::White),
            48 if pi + 2 < params.len() => match params[pi + 1] {
                2 if pi + 4 < params.len() => {
                    state.bg = Some(Color::Rgb(
                        params[pi + 2].clamp(0, 255) as u8,
                        params[pi + 3].clamp(0, 255) as u8,
                        params[pi + 4].clamp(0, 255) as u8,
                    ));
                    pi += 4;
                }
                5 if pi + 2 < params.len() => {
                    state.bg = Some(indexed_color(params[pi + 2]));
                    pi += 2;
                }
                _ => {}
            },
            49 => state.bg = None,
            90 => state.fg = Some(Color::Rgb(128, 128, 128)),
            91 => state.fg = Some(Color::Rgb(255, 128, 128)),
            92 => state.fg = Some(Color::Rgb(128, 255, 128)),
            93 => state.fg = Some(Color::Rgb(255, 255, 128)),
            94 => state.fg = Some(Color::Rgb(128, 128, 255)),
            95 => state.fg = Some(Color::Rgb(255, 128, 255)),
            96 => state.fg = Some(Color::Rgb(128, 255, 255)),
            97 => state.fg = Some(Color::White),
            100 => state.bg = Some(Color::Rgb(128, 128, 128)),
            101 => state.bg = Some(Color::Rgb(255, 128, 128)),
            102 => state.bg = Some(Color::Rgb(128, 255, 128)),
            103 => state.bg = Some(Color::Rgb(255, 255, 128)),
            104 => state.bg = Some(Color::Rgb(128, 128, 255)),
            105 => state.bg = Some(Color::Rgb(255, 128, 255)),
            106 => state.bg = Some(Color::Rgb(128, 255, 255)),
            107 => state.bg = Some(Color::White),
            _ => {}
        }
        pi += 1;
    }
}

fn indexed_color(n: i32) -> Color {
    let n = n.clamp(0, 255) as u8;
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        8 => Color::Rgb(128, 128, 128),
        9 => Color::Rgb(255, 128, 128),
        10 => Color::Rgb(128, 255, 128),
        11 => Color::Rgb(255, 255, 128),
        12 => Color::Rgb(128, 128, 255),
        13 => Color::Rgb(255, 128, 255),
        14 => Color::Rgb(128, 255, 255),
        15 => Color::Rgb(255, 255, 255),
        n if n < 232 => {
            let n = n as u32 - 16;
            Color::Rgb(
                ((n / 36) * 51) as u8,
                ((n % 36 / 6) * 51) as u8,
                ((n % 6) * 51) as u8,
            )
        }
        n => {
            let g = ((n as u32 - 232) * 10 + 8) as u8;
            Color::Rgb(g, g, g)
        }
    }
}

pub(super) fn ansi_line_count(text: &str, max_width: usize) -> usize {
    if max_width == 0 {
        return text.lines().count();
    }
    text.lines()
        .map(|line| {
            let w = ansi_stripped_width(line);
            if w == 0 { 1 } else { w.div_ceil(max_width) }
        })
        .sum()
}

pub(super) fn ansi_stripped_width(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut width = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() && !bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
        } else {
            let c = s[i..].chars().next().unwrap_or('\0');
            width += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
            i += c.len_utf8();
        }
    }
    width
}

pub(super) fn wrapped_line_count(text: &str, max_width: usize) -> usize {
    ansi_line_count(text, max_width)
}
