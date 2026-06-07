use std::path::Path;

pub fn claw_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".i-rs").join("claw"))
}

use std::sync::{Arc, LazyLock};
static SHARED_RUNTIME: LazyLock<Arc<tokio::runtime::Runtime>> = LazyLock::new(|| {
    Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("sync_block_on: failed to create shared runtime"),
    )
});

/// Returns a clone of the shared tokio runtime (ref-counted `Arc`).
/// Intended for components that need their own handle to the runtime,
/// such as `McpRegistry`, to avoid creating additional runtimes.
pub fn shared_runtime() -> Arc<tokio::runtime::Runtime> {
    Arc::clone(&SHARED_RUNTIME)
}

/// Block the current thread on a future by spawning a dedicated scope thread
/// that drives the future on `SHARED_RUNTIME`.
///
/// This is a **bridge** from sync code to async storage backends. It is the
/// only sanctioned way to call async storage code from sync context in this
/// crate.
///
/// # Panics
///
/// Panics if the spawned scope thread panics (only happens if `f` itself
/// panics). The scope thread is joined with `unwrap()`, so a panic in `f`
/// propagates to the caller.
///
/// # Runtime nesting
///
/// If called from inside a tokio runtime context, a warning is emitted but
/// execution continues via `std::thread::scope`. The scope thread runs on a
/// fresh OS thread with no tokio context, which safely avoids runtime
/// nesting. However, callers should prefer `.await` over `sync_block_on`
/// whenever possible.
///
/// # When to use
///
/// - Inside sync constructors like `SessionManager::with_storage`
/// - Inside TUI event handlers (which run on the main thread, not in an async runtime)
///
/// # When NOT to use
///
/// - Inside `async fn` — just `.await` the future directly
/// - Inside axum handlers — they're already async
/// - Inside tokio tasks — they're already async
pub fn sync_block_on<F: std::future::Future + Send>(f: F) -> F::Output
where
    F::Output: Send,
{
    // If already inside a tokio runtime, emit a warning so we can find
    // and fix these call sites. The scope thread still runs safely because
    // it spawns on a fresh OS thread without tokio context.
    if tokio::runtime::Handle::try_current().is_ok() {
        tracing::warn!(
            "sync_block_on called from inside a tokio runtime context — \
             prefer .await; falling back to scope thread"
        );
    }

    // Always run on a dedicated scope thread using SHARED_RUNTIME.
    // This avoids ALL runtime nesting issues — including the
    // "Cannot drop a runtime in a context where blocking is not allowed"
    // panic when a `reqwest::Client` internal runtime is dropped within
    // an async context.
    std::thread::scope(|s| s.spawn(|| SHARED_RUNTIME.block_on(f)).join().unwrap())
}

// ── Timezone ──

/// Get the system's local timezone offset.
pub fn system_tz_offset() -> chrono::FixedOffset {
    let local = chrono::Local::now();
    *local.offset()
}

/// Parse a timezone string into a FixedOffset.
/// Supports: "UTC", "+08:00", "-05:00", "+8", "-5", "8", None (system local).
pub fn parse_timezone(tz: Option<&str>) -> chrono::FixedOffset {
    let tz = match tz {
        Some(t) if !t.trim().is_empty() => t.trim(),
        _ => return system_tz_offset(),
    };

    if tz.eq_ignore_ascii_case("utc") {
        return chrono::FixedOffset::east_opt(0).unwrap_or_else(system_tz_offset);
    }

    // UTC+8, UTC-5
    if let Some(rest) = tz.to_uppercase().strip_prefix("UTC")
        && let Ok(hours) = rest.parse::<i32>()
    {
        return chrono::FixedOffset::east_opt(hours * 3600).unwrap_or_else(system_tz_offset);
    }

    // +08:00, -05:00
    if let Ok(offset) = tz.parse::<chrono::FixedOffset>() {
        return offset;
    }

    // +8, -5, 8 (bare hours)
    if let Ok(hours) = tz.parse::<i32>() {
        return chrono::FixedOffset::east_opt(hours * 3600).unwrap_or_else(system_tz_offset);
    }

    tracing::warn!("无法解析时区配置 '{}', 回退到系统本地时区", tz);
    system_tz_offset()
}

/// Get current datetime in the configured timezone.
pub fn now_in_tz(offset: chrono::FixedOffset) -> chrono::DateTime<chrono::FixedOffset> {
    chrono::Utc::now().with_timezone(&offset)
}

/// Format the timezone label for display (e.g., "+08:00", "UTC").
pub fn tz_label(offset: chrono::FixedOffset) -> String {
    let total_secs = offset.local_minus_utc();
    if total_secs == 0 {
        return "UTC".to_string();
    }
    let sign = if total_secs >= 0 { "+" } else { "-" };
    let abs_secs = total_secs.abs();
    format!(
        "{}{:02}:{:02}",
        sign,
        abs_secs / 3600,
        (abs_secs % 3600) / 60
    )
}

/// Run a CLI command with timeout, returning stdout on success or an error string.
/// Provides a unified subprocess invocation pattern across all tools.
pub fn run_cli_command(binary: &str, args: &[&str], timeout_secs: u64) -> Result<String, String> {
    let mut child = std::process::Command::new(binary)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(std::env::temp_dir())
        .spawn()
        .map_err(|e| format!("执行 {} 失败: {}", binary, e))?;

    let start = std::time::Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|e| format!("读取命令输出失败: {}", e))?;

                let max_output = 10_000;
                if status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let trimmed = stdout.trim();
                    if trimmed.is_empty() {
                        return Ok(r#"{"success":true}"#.to_string());
                    } else if trimmed.len() > max_output {
                        let preview: String = trimmed.chars().take(max_output).collect();
                        return Ok(format!(
                            "{}...\n[输出截断: 共 {} 字符，仅显示前 {} 字符]",
                            preview,
                            trimmed.len(),
                            max_output
                        ));
                    } else {
                        return Ok(trimmed.to_string());
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let combined = if stderr.trim().is_empty() {
                        stdout.trim().to_string()
                    } else {
                        stderr.trim().to_string()
                    };
                    if combined.len() > max_output {
                        let preview: String = combined.chars().take(max_output).collect();
                        return Err(format!(
                            "{}...\n[输出截断: 共 {} 字符，仅显示前 {} 字符]",
                            preview,
                            combined.len(),
                            max_output
                        ));
                    }
                    return Err(combined);
                }
            }
            Ok(None) => {
                if start.elapsed() > std::time::Duration::from_secs(timeout_secs) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "命令执行超时 ({}s): {} {}",
                        timeout_secs,
                        binary,
                        args.join(" ")
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => return Err(format!("等待命令完成失败: {}", e)),
        }
    }
}

/// Run an i-rs CLI command via the meta binary and parse JSON output.
/// Useful for chart tools and other consumers that need structured data.
pub fn run_i_rs_json(
    tool: &str,
    command: &str,
    extra_args: &[&str],
    timeout_secs: u64,
) -> Result<serde_json::Value, String> {
    let mut args: Vec<&str> = vec![tool, command, "--json"];
    args.extend_from_slice(extra_args);
    let output = run_cli_command("i-rs", &args, timeout_secs)?;
    serde_json::from_str(&output).map_err(|e| format!("解析 JSON 输出失败: {}", e))
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

/// Check if a message content appears to be a user correction or negation.
pub fn is_correction_message(text: &str) -> bool {
    let lower = text.to_lowercase();
    let corrections = [
        "不对",
        "不是",
        "错了",
        "错误",
        "更正",
        "重新",
        "重试",
        "no,",
        "not that",
        "wrong",
        "incorrect",
        "correction",
        "redo",
        "我说的不是",
        "我要的是",
        "改一下",
        "修正",
    ];
    corrections.iter().any(|&k| lower.contains(k))
}

/// Check if a message content appears to be a decision or confirmation.
pub fn is_decision_message(text: &str) -> bool {
    let lower = text.to_lowercase();
    let decisions = [
        "确认",
        "确定",
        "就这样",
        "可以了",
        "同意",
        "批准",
        "confirm",
        "yes",
        "agreed",
        "approved",
        "that's correct",
        "没问题",
        "就这么办",
        "好的",
    ];
    decisions.iter().any(|&k| lower.contains(k))
}

/// Compact structured tool results for LLM context injection.
///
/// For JSON array results from i-rs tools, produces a compact summary
/// instead of raw truncation. Falls back to smart_truncate for non-JSON output.
pub fn compact_tool_result(tool_name: &str, result: &str, max_chars: usize) -> String {
    let stripped = strip_ansi(result);
    if stripped.chars().count() <= max_chars {
        return stripped;
    }

    let trimmed = stripped.trim();
    if !trimmed.starts_with('[') && !trimmed.starts_with('{') {
        return smart_truncate(result, max_chars);
    }

    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return smart_truncate(result, max_chars);
    };

    match &parsed {
        serde_json::Value::Array(arr) if !arr.is_empty() => {
            let mut summary = format!("[{} 共 {} 条结果] ", tool_name, arr.len());

            let mut sample_count = 0;
            let mut sample_str = String::new();

            for item in arr.iter().take(5) {
                if let serde_json::Value::Object(obj) = item {
                    sample_count += 1;
                    sample_str.push_str(&format!("{}: ", sample_count));

                    for (field_count, (k, v)) in obj.into_iter().enumerate() {
                        if field_count >= 4 {
                            sample_str.push_str("...");
                            break;
                        }
                        if let Some(s) = v.as_str() {
                            sample_str.push_str(&format!("{}={} ", k, s));
                        } else if let Some(n) = v.as_f64() {
                            sample_str.push_str(&format!("{}={} ", k, n));
                        } else if let Some(b) = v.as_bool() {
                            sample_str.push_str(&format!("{}={} ", k, b));
                        }
                    }
                    sample_str.push_str("; ");
                }
            }

            if sample_count > 0 {
                summary.push_str(&sample_str);
            }

            if arr.len() > 5 {
                summary.push_str(&format!("... 还有 {} 条", arr.len() - 5));
            }

            if summary.len() > max_chars {
                let s: String = summary.chars().take(max_chars - 3).collect();
                format!("{}...", s)
            } else {
                summary
            }
        }
        serde_json::Value::Array(_) => smart_truncate(result, max_chars),
        _ => smart_truncate(result, max_chars),
    }
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
            && s.len() < stripped.len()
        {
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
        assert_eq!(
            find_json_prefix(r#"{"a":1,"b":2}"#).as_deref(),
            Some(r#"{"a":1,"b":2}"#)
        );
    }

    #[test]
    fn test_find_json_prefix_nested() {
        let s = r#"{"a":{"b":[1,2]},"c":3}extra"#;
        assert_eq!(
            find_json_prefix(s).as_deref(),
            Some(r#"{"a":{"b":[1,2]},"c":3}"#)
        );
    }

    #[test]
    fn test_find_json_prefix_array() {
        assert_eq!(find_json_prefix("[1,2,3]").as_deref(), Some("[1,2,3]"));
    }

    #[test]
    fn test_find_json_prefix_string() {
        assert_eq!(
            find_json_prefix(r#""hello"more"#).as_deref(),
            Some(r#""hello""#)
        );
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
