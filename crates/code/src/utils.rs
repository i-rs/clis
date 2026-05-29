/// Truncate tool output at a byte limit, breaking at the last newline before the limit.
pub fn truncate_output(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    if let Some(pos) = s[..end].rfind('\n') {
        end = pos + 1;
    }
    let truncated = &s[..end];
    format!(
        "{}\n\n[output truncated: {} bytes omitted, total {} bytes]",
        truncated,
        s.len() - truncated.len(),
        s.len()
    )
}
