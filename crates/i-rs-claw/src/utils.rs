/// Smart truncation for LLM tool results.
///
/// 1. Strip ANSI color codes (useless for LLM consumption)
/// 2. If result is JSON, prefer truncating at a complete JSON object boundary
/// 3. Otherwise truncate at char boundary with "...(truncated)" suffix
pub fn smart_truncate(s: &str, max_chars: usize) -> String {
    // Step 1: Strip ANSI escape sequences
    let stripped = strip_ansi(s);

    if stripped.chars().count() <= max_chars {
        return stripped;
    }

    // Step 2: If it looks like JSON, try to find a complete object boundary
    let trimmed: &str = stripped.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') || trimmed.starts_with('"') {
        // Try to parse as JSON, truncate at the last complete value within limit
        let truncated: String = stripped.chars().take(max_chars).collect();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&truncated) {
            if let Ok(s) = serde_json::to_string(&v) {
                if s.len() < stripped.len() {
                    return format!("{}...(truncated)", s);
                }
            }
        }
        // Fallback: try to find last complete object by counting braces
        if let Some(complete) = find_json_prefix(&truncated) {
            return format!("{}...(truncated)", complete);
        }
    }

    // Default: truncate at char boundary
    let truncated: String = stripped.chars().take(max_chars).collect();
    format!("{}...(truncated)", truncated)
}

/// Strip ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until the letter that terminates the escape sequence
            while let Some(esc) = chars.next() {
                if esc.is_ascii_alphabetic() || esc == '~' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Find the longest complete JSON prefix within the given string.
fn find_json_prefix(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let first = s.chars().next()?;
    let (open, close) = match first {
        '{' => ('{', '}'),
        '[' => ('[', ']'),
        '"' => ('"', '"'),
        _ => return None,
    };

    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut last_complete = 0;

    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && in_string {
            escaped = true;
            continue;
        }
        if c == '"' && first != '"' {
            in_string = !in_string;
        }
        if !in_string {
            if c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
                if depth == 0 {
                    last_complete = i + c.len_utf8();
                }
            }
        }
    }

    if last_complete > 0 {
        Some(s[..last_complete].to_string())
    } else {
        None
    }
}
