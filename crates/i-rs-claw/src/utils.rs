/// Safe UTF-8 truncation: cut string at a char boundary, max `max_bytes` bytes.
pub fn truncate(s: &str, max_bytes: usize) -> &str {
    let max = max_bytes.min(s.len());
    let bound = s
        .char_indices()
        .take_while(|(i, _)| *i < max)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    &s[..bound]
}
