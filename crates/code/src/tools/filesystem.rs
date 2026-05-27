use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};
use std::path::Path;

pub struct ReadTool;
pub struct WriteTool;
pub struct EditTool;
pub struct GlobTool;
pub struct GrepTool;
pub struct LsTool;

fn check_path(path: &str) -> anyhow::Result<()> {
    let p = Path::new(path);
    if p.components().any(|c| c.as_os_str() == "..") {
        anyhow::bail!("Path traversal detected: {}", path);
    }
    Ok(())
}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str { "read" }
    fn description(&self) -> &str { "Read a file with line numbers. Use offset and limit for large files." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read a file with line numbers. Use offset and limit for large files.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string", "description": "Path to the file"},
                        "offset": {"type": "integer", "description": "Line number to start from (1-indexed, default: 1)"},
                        "limit": {"type": "integer", "description": "Max lines to read (default: 200)"}
                    },
                    "required": ["file_path"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        check_path(path)?;
        let content = std::fs::read_to_string(path)?;

        let total_lines = content.lines().count();
        let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(1).max(1) as usize;
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200) as usize;

        let lines: Vec<&str> = content.lines().collect();
        let end = (offset + limit - 1).min(lines.len());
        let selected: Vec<&str> = lines[(offset - 1)..end].to_vec();

        let max_digits = end.to_string().len();
        let numbered: Vec<String> = selected.iter().enumerate()
            .map(|(i, l)| format!("{:>width$}: {}", offset + i, l, width = max_digits))
            .collect();

        let header = if offset > 1 || end < total_lines {
            format!("{} (lines {}-{} of {})\n```\n{}\n```", path, offset, end, total_lines, numbered.join("\n"))
        } else {
            format!("{}\n```\n{}\n```", path, numbered.join("\n"))
        };
        Ok(header)
    }
}

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str { "write" }
    fn description(&self) -> &str { "Create or overwrite a file" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "write",
                "description": "Create or overwrite a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "content": {"type": "string"}
                    },
                    "required": ["file_path", "content"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        let content = args.get("content").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("content required"))?;
        check_path(path)?;

        let old_content = if Path::new(path).exists() {
            std::fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        };

        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;

        if old_content.is_empty() {
            Ok(format!("Created {} ({} bytes)\n```{}\n```", path, content.len(), content))
        } else {
            let diff = crate::diff::diff_text(&old_content, content);
            Ok(format!(
                "Modified {} (+{} -{})\n```diff\n{}\n```",
                path, diff.lines_added, diff.lines_removed, diff.patch
            ))
        }
    }
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str { "edit" }
    fn description(&self) -> &str { "Edit a file by replacing text (use replace_all for batch renames)" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "edit",
                "description": "Edit a file by replacing text",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "old_string": {"type": "string", "description": "Text to replace (must match exactly once, unless replace_all)"},
                        "new_string": {"type": "string"},
                        "replace_all": {"type": "boolean", "description": "Replace all occurrences (default: false)"}
                    },
                    "required": ["file_path", "old_string", "new_string"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        let old = args.get("old_string").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("old_string required"))?;
        let new = args.get("new_string").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("new_string required"))?;
        let replace_all = args.get("replace_all").and_then(|v| v.as_bool()).unwrap_or(false);
        check_path(path)?;
        let content = std::fs::read_to_string(path)?;
        let count = content.matches(old).count();
        if count == 0 {
            anyhow::bail!("old_string not found in {}", path);
        }
        if count > 1 && !replace_all {
            anyhow::bail!("Found {} matches for old_string. Use replace_all=true or provide more context.", count);
        }

        let new_content = if replace_all {
            content.replace(old, new)
        } else {
            content.replacen(old, new, 1)
        };

        let diff = crate::diff::diff_text(&content, &new_content);
        std::fs::write(path, &new_content)?;

        Ok(format!(
            "Edited {} (+{} -{}{})\n```diff\n{}\n```",
            path,
            diff.lines_added,
            diff.lines_removed,
            if replace_all { format!(" ({} replacements)", count) } else { String::new() },
            diff.patch
        ))
    }
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str { "glob" }
    fn description(&self) -> &str { "Find files matching a glob pattern" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "glob",
                "description": "Find files matching a glob pattern",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string", "description": "Glob pattern e.g. **/*.rs"},
                        "path": {"type": "string", "description": "Root directory (default: .)"}
                    },
                    "required": ["pattern"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("pattern required"))?;
        let root = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let glob = globset::Glob::new(pattern)?;
        let matcher = glob.compile_matcher();
        let mut results = Vec::new();
        for entry in walkdir::WalkDir::new(root).max_depth(10).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && matcher.is_match(entry.path()) {
                results.push(entry.path().to_string_lossy().to_string());
            }
        }
        results.sort();
        if results.is_empty() {
            return Ok("No files found".into());
        }
        Ok(format!("Found {} files:\n{}", results.len(), results.join("\n")))
    }
}

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
                        "include": {"type": "string", "description": "File pattern e.g. *.rs"}
                    },
                    "required": ["pattern"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("pattern required"))?;
        let root = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let re = regex::Regex::new(pattern)?;

        let lines = std::sync::Mutex::new(Vec::new());
        let walker = ignore::WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker.flatten() {
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(path) {
                for (i, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        let mut l = lines.lock().unwrap();
                        l.push(format!("{}:{}:{}", path.display(), i + 1, line));
                        if l.len() >= 50 {
                            break;
                        }
                    }
                }
            }
        }
        let results = lines.into_inner().unwrap();
        if results.is_empty() {
            return Ok("No matches found".into());
        }
        Ok(format!("Found {} matches:\n{}", results.len(), results.join("\n")))
    }
}

#[async_trait]
impl Tool for LsTool {
    fn name(&self) -> &str { "ls" }
    fn description(&self) -> &str { "List directory contents" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "ls",
                "description": "List directory contents",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Directory path (default: .)"}
                    },
                    "required": []
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        check_path(path)?;
        let entries = std::fs::read_dir(path)?;
        let mut items = Vec::new();
        for entry in entries {
            let entry = entry?;
            let ftype = if entry.file_type()?.is_dir() { "dir" } else { "file" };
            items.push(format!("{}  {}", ftype, entry.file_name().to_string_lossy()));
        }
        items.sort();
        Ok(format!("{}:\n{}", path, items.join("\n")))
    }
}
