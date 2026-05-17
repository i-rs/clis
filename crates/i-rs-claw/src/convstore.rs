use serde_json::Value;
use std::path::PathBuf;

/// A single search result from a conversation session.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub session_id: String,
    pub session_title: String,
    pub message_type: String,
    pub excerpt: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    #[allow(dead_code)]
    pub updated_at: i64,
}

/// Full-text search across all conversation session JSONL files.
///
/// Sessions are stored in:
///   ~/.i-rs-claw/claw/
///   ├── index.json              # Session list (Vec<SessionMeta>)
///   └── sessions/
///       └── {session_id}.jsonl  # App messages in JSONL format
pub struct ConvStore {
    /// Base directory where index.json and sessions/ live.
    claw_dir: PathBuf,
}

impl ConvStore {
    pub fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }

    /// Search message text across all sessions.
    ///
    /// `query` is matched case-insensitively against user, assistant, and error
    /// message text, as well as tool_call names (without searching full args/results).
    /// Returns up to `max_results` results, ordered by session update time (newest first).
    pub fn search(&self, query: &str, max_results: usize) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();

        // Load session index
        let sessions = self.load_index();
        let sessions_dir = self.claw_dir.join("sessions");

        let mut results: Vec<SearchResult> = Vec::new();

        for meta in &sessions {
            let jsonl_path = sessions_dir.join(format!("{}.jsonl", meta.id));
            if !jsonl_path.exists() {
                continue;
            }
            let content = match std::fs::read_to_string(&jsonl_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let lines: Vec<Value> = content
                .lines()
                .filter_map(|line| {
                    if line.trim().is_empty() {
                        None
                    } else {
                        serde_json::from_str(line).ok()
                    }
                })
                .collect();

            for (i, msg) in lines.iter().enumerate() {
                let msg_type = msg.get("type").and_then(|t| t.as_str()).unwrap_or("");
                let text = msg.get("text").and_then(|t| t.as_str()).unwrap_or("");
                let name = msg.get("name").and_then(|n| n.as_str()).unwrap_or("");

                // Determine searchable content per message type
                let searchable = match msg_type {
                    "user" | "assistant" | "error" => text.to_lowercase(),
                    // For tool calls, only search by tool name (not full args/results)
                    "tool_call" => format!("[tool: {}]", name).to_lowercase(),
                    _ => continue,
                };

                if !searchable.contains(&query_lower) {
                    continue;
                }

                // Build excerpt
                let excerpt = match msg_type {
                    "tool_call" => format!("[工具调用: {}]", name),
                    _ => {
                        let t: String = text.chars().take(200).collect();
                        if text.len() > 200 {
                            format!("{}...", t)
                        } else {
                            t
                        }
                    }
                };

                // Context: up to 2 messages before
                let context_before: Vec<String> = lines
                    [i.saturating_sub(2)..i]
                    .iter()
                    .filter_map(|m| {
                        let t = m.get("text").and_then(|v| v.as_str())?;
                        let s: String = t.chars().take(100).collect();
                        Some(s)
                    })
                    .collect();

                // Context: up to 1 message after (only if there is a next message)
                let context_after: Vec<String> = if i + 1 < lines.len() {
                    let end = std::cmp::min(i + 1, lines.len() - 1);
                    lines[i + 1..=end]
                        .iter()
                        .filter_map(|m| {
                            let t = m.get("text").and_then(|v| v.as_str())?;
                            let s: String = t.chars().take(100).collect();
                            Some(s)
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                results.push(SearchResult {
                    session_id: meta.id.clone(),
                    session_title: meta.title.clone(),
                    message_type: msg_type.to_string(),
                    excerpt,
                    context_before,
                    context_after,
                    updated_at: meta.updated_at,
                });

                if results.len() >= max_results {
                    return results;
                }
            }
        }

        results
    }

    fn load_index(&self) -> Vec<crate::session::SessionMeta> {
        let path = self.claw_dir.join("index.json");
        if !path.exists() {
            return Vec::new();
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(sessions) = serde_json::from_str(&content) {
                return sessions;
            }
        }
        Vec::new()
    }
}
