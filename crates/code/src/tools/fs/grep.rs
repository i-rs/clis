use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str { "grep" }
    fn description(&self) -> &str { "Search file contents using regex" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "grep",
                "description": "Search file contents using regex",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string"},
                        "path": {"type": "string", "description": "Root directory (default: .)"},
                        "include": {"type": "string", "description": "File glob filter e.g. *.rs, *.py"}
                    },
                    "required": ["pattern"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("pattern required"))?.to_string();
        let root = args.get("path").and_then(|v| v.as_str()).unwrap_or(".").to_string();
        let include = args.get("include").and_then(|v| v.as_str()).map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let re = regex::Regex::new(&pattern)?;
            let lines = std::sync::Mutex::new(Vec::new());
            let mut builder = ignore::WalkBuilder::new(&root);
            builder.hidden(false);
            builder.git_ignore(true);
            if let Some(ref glob_str) = include {
                let glob = globset::Glob::new(glob_str)?;
                let matcher = glob.compile_matcher();
                builder.filter_entry(move |entry| {
                    entry.path().to_str().is_none_or(|p| matcher.is_match(p))
                });
            }
            let walker = builder.build();

            for entry in walker.flatten() {
                if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    continue;
                }
                let path = entry.path().to_owned();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            let mut l = lines.lock().unwrap_or_else(|e| e.into_inner());
                            l.push(format!("{}:{}:{}", path.display(), i + 1, line));
                            if l.len() >= 200 {
                                l.push("[results truncated — use offset/limit to refine]".into());
                                break;
                            }
                        }
                    }
                }
            }
            let results = lines.into_inner().unwrap();
            if results.is_empty() {
                Ok("No matches found".into())
            } else {
                Ok(format!("Found {} matches:\n{}", results.len(), results.join("\n")))
            }
        }).await?
    }
}
