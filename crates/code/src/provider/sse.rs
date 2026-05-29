/// SSE event extracted from a byte-stream chunk.
pub struct SseEvent {
    pub event_type: Option<String>,
    pub data: String,
}

/// Parse SSE lines from a chunked byte stream buffer.
///
/// Handles both formats:
/// - OpenAI/Ollama: `data: {...}` without `event:` headers
/// - Anthropic: `event: message_start` + `data: {...}`
///
/// Call repeatedly with each incoming chunk. Returns all complete
/// SSE events found in the buffer after appending the new chunk.
pub fn parse_sse(buf: &mut String, chunk: &[u8]) -> Vec<SseEvent> {
    let chunk_str = String::from_utf8_lossy(chunk);
    buf.push_str(&chunk_str);
    let mut events = Vec::new();
    let mut event_type: Option<String> = None;
    while let Some(pos) = buf.find('\n') {
        let line = buf[..pos].trim_end_matches('\r').to_string();
        *buf = buf[pos + 1..].to_string();
        if line.is_empty() {
            continue;
        }
        if let Some(evt) = line.strip_prefix("event: ") {
            event_type = Some(evt.to_string());
            continue;
        }
        if let Some(data) = line.strip_prefix("data: ") {
            events.push(SseEvent {
                event_type: event_type.take(),
                data: data.to_string(),
            });
        }
    }
    events
}
