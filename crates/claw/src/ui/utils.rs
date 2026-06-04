use std::borrow::Cow;

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

pub(super) fn truncate_str<'a>(s: &'a str, max_len: usize) -> Cow<'a, str> {
    let width = UnicodeWidthStr::width(s);
    if width <= max_len {
        Cow::Borrowed(s)
    } else {
        let mut out = String::new();
        let mut w = 0usize;
        for c in s.chars() {
            let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            if w + cw + 3 > max_len {
                break;
            }
            out.push(c);
            w += cw;
        }
        out.push_str("...");
        Cow::Owned(out)
    }
}

pub(super) fn relative_time_at(ts: i64, now: i64) -> String {
    let diff = now.saturating_sub(ts);
    if diff < 60 {
        "刚刚".to_string()
    } else if diff < 3600 {
        format!("{}分钟前", diff / 60)
    } else if diff < 86400 {
        format!("{}小时前", diff / 3600)
    } else if diff < 2592000 {
        format!("{}天前", diff / 86400)
    } else {
        format!("{}月前", diff / 2592000)
    }
}

pub(super) fn relative_time(ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    relative_time_at(ts, now)
}

pub(super) fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let line_count = text.lines().count();
    let mut lines = Vec::with_capacity(line_count);
    for line in text.lines() {
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }
        let line_width = UnicodeWidthStr::width(line);
        if line_width <= max_width {
            lines.push(line.to_string());
            continue;
        }
        let mut current = String::new();
        let mut current_w = 0;
        for word in line.split(' ') {
            let word_w = UnicodeWidthStr::width(word);
            let separator = if current.is_empty() { 0 } else { 1 };
            if current_w + separator + word_w > max_width {
                if current.is_empty() {
                    lines.extend(force_split(word, max_width));
                    continue;
                }
                lines.push(current);
                current = String::new();
                current_w = 0;
            }

            if !current.is_empty() {
                current.push(' ');
                current_w += 1;
            }

            if word_w > max_width {
                let forced = force_split(word, max_width.saturating_sub(current_w));
                let mut first = true;
                let mut remaining_is_long = false;
                for part in &forced {
                    if first {
                        first = false;
                        current.push_str(part);
                        current_w += UnicodeWidthStr::width(part.as_str());
                        if UnicodeWidthStr::width(part.as_str())
                            > max_width.saturating_sub(current_w)
                            && forced.len() > 1
                        {
                            remaining_is_long = true;
                        }
                    } else {
                        lines.push(current);
                        current = String::new();
                        current.push_str(part);
                        current_w = UnicodeWidthStr::width(part.as_str());
                    }
                }
                if remaining_is_long {
                    lines.push(current);
                    current = String::new();
                    current_w = 0;
                }
            } else {
                current.push_str(word);
                current_w += word_w;
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

fn force_split(text: &str, max_width: usize) -> Vec<String> {
    let max_width = max_width.max(1);
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut current_w = 0;
    for c in text.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
        if current_w + cw > max_width && !current.is_empty() {
            parts.push(std::mem::take(&mut current));
            current_w = 0;
        }
        current.push(c);
        current_w += cw;
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// Format an already-parsed JSON value into wrapped, styled lines.
/// Returns the lines (empty if the value renders to nothing).
///
/// Callers that already hold a `&Value` should prefer this over
/// re-parsing the JSON string.
pub(super) fn format_json_lines(val: &serde_json::Value, max_width: usize) -> Vec<Line<'static>> {
    let formatted = serde_json::to_string_pretty(val).unwrap_or_else(|_| val.to_string());
    let indent_width = max_width.saturating_sub(4);
    let mut lines = Vec::new();
    for line in formatted.lines() {
        if line.is_empty() {
            lines.push(Line::from(Span::styled(
                "  ".to_string(),
                Style::default().fg(Color::Rgb(160, 180, 160)),
            )));
            continue;
        }
        let line_w = UnicodeWidthStr::width(line);
        if line_w <= indent_width {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(Color::Rgb(160, 180, 160)),
            )));
        } else {
            for part in wrap_text(line, indent_width) {
                lines.push(Line::from(Span::styled(
                    format!("  {}", part),
                    Style::default().fg(Color::Rgb(160, 180, 160)),
                )));
            }
        }
    }
    lines
}
