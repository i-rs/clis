use std::path::Path;

pub fn claw_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".i-rs").join("claw"))
}

/// Atomic file write: write to a temp file first, then atomically rename.
/// This prevents data corruption if the process crashes mid-write.
/// Returns `Ok(())` on success, `Err` with a description on failure.
pub fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write to a temporary file next to the target
    let tmp_path = path.with_extension(format!(
        "tmp.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    std::fs::write(&tmp_path, content.as_bytes())?;

    // Set restrictive permissions (0600) on the temp file before renaming
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tmp_path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&tmp_path, perms)?;
    }

    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

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
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&truncated)
            && let Ok(s) = serde_json::to_string(&v)
                && s.len() < stripped.len() {
                    return format!("{}...(truncated)", s);
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
            for esc in chars.by_ref() {
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

/// Find complete JSON string enclosed in quotes.
fn find_json_string(s: &str) -> Option<String> {
    let s = s.trim();
    let mut chars = s.char_indices();
    // Skip leading quote
    let _ = chars.next();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            // Skip escaped character
            let _ = chars.next();
        } else if c == '"' {
            return Some(s[..=i].to_string());
        }
    }
    None
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
        '"' => return find_json_string(s), // special case: string
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_no_ansi() {
        assert_eq!(strip_ansi("hello world"), "hello world");
    }

    #[test]
    fn test_strip_ansi_sgr() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn test_strip_ansi_multiple() {
        assert_eq!(strip_ansi("\x1b[1m\x1b[32mbold green\x1b[0m"), "bold green");
    }

    #[test]
    fn test_strip_ansi_cursor() {
        assert_eq!(strip_ansi("line1\x1b[K\nline2"), "line1\nline2");
    }

    #[test]
    fn test_strip_ansi_empty() {
        assert_eq!(strip_ansi(""), "");
    }

    #[test]
    fn test_find_json_prefix_object() {
        assert_eq!(find_json_prefix(r#"{"a":1,"b":2}"#).as_deref(), Some(r#"{"a":1,"b":2}"#));
    }

    #[test]
    fn test_find_json_prefix_nested() {
        let s = r#"{"a":{"b":[1,2]},"c":3}extra"#;
        assert_eq!(find_json_prefix(s).as_deref(), Some(r#"{"a":{"b":[1,2]},"c":3}"#));
    }

    #[test]
    fn test_find_json_prefix_array() {
        assert_eq!(find_json_prefix("[1,2,3]").as_deref(), Some("[1,2,3]"));
    }

    #[test]
    fn test_find_json_prefix_string() {
        assert_eq!(find_json_prefix(r#""hello"more"#).as_deref(), Some(r#""hello""#));
    }

    #[test]
    fn test_find_json_prefix_unbalanced() {
        assert_eq!(find_json_prefix(r#"{"a":1"#), None);
    }

    #[test]
    fn test_find_json_prefix_not_json() {
        assert_eq!(find_json_prefix("plain text"), None);
    }

    #[test]
    fn test_find_json_prefix_empty() {
        assert_eq!(find_json_prefix(""), None);
    }

    #[test]
    fn test_smart_truncate_plain_under_limit() {
        let result = smart_truncate("short", 100);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_smart_truncate_plain_over_limit() {
        let result = smart_truncate("this is a long string", 10);
        assert!(result.ends_with("...(truncated)"));
        assert!(result.len() <= 10 + "...(truncated)".len());
    }

    #[test]
    fn test_smart_truncate_with_ansi() {
        let result = smart_truncate("\x1b[31mhello\x1b[0m", 100);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_smart_truncate_json() {
        let s = r#"{"a":"very long string that should be truncated","b":2}"#;
        let result = smart_truncate(s, 30);
        assert!(result.contains("...(truncated)"));
    }

    #[test]
    fn test_smart_truncate_empty() {
        assert_eq!(smart_truncate("", 10), "");
    }
}
