use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Built-in tool for file operations (read/write/list) within allowed directories.
///
/// Strictly validates that all file paths are inside the configured `allowed_dirs`.
/// File operations are disabled if no allowed directories are configured.
pub struct FileOpsTool;

impl ClawTool for FileOpsTool {
    fn name(&self) -> &str {
        "file_ops"
    }

    fn description(&self) -> &str {
        "Read, write, or list files on the local filesystem. Only works within \
         directories configured as allowed in settings. Use this when the user \
         asks you to read a file, save content to a file, or explore a directory."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["read", "write", "list"],
                    "description": "Operation to perform: read (read file contents), write (write content to a file), list (list directory contents)"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory path (relative paths resolved from the first allowed directory)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write (only used when operation is 'write')"
                }
            },
            "required": ["operation", "path"],
            "additionalProperties": false
        })
    }

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if operation.is_empty() {
            return Err(ClawError::Validation("Please specify an operation: read, write, or list".to_string()));
        }
        if path_str.is_empty() {
            return Err(ClawError::Validation("Please specify a file path".to_string()));
        }

        // Load config for allowed directories
        let cfg = crate::config::Config::load().map_err(|e| format!("加载配置失败: {}", e))?;

        if cfg.allowed_dirs.is_empty() {
            return Err(ClawError::Validation("文件操作未启用：没有配置允许的目录。请运行 `i-rs-claw config` 设置 allowed_dirs。".to_string()));
        }

        // Resolve the path against the first allowed directory
        let base = PathBuf::from(&cfg.allowed_dirs[0]);
        let target = if Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            base.join(path_str)
        };

        // Check that the target is within one of the allowed directories.
        // Use canonicalize to resolve symlinks and prevent path traversal.
        // For write operations on new files (where canonicalize fails),
        // canonicalize the parent directory and verify the parent is allowed.
        let canonical_target = match target.canonicalize() {
            Ok(p) => p,
            Err(_) if operation == "write" => {
                // File doesn't exist yet — canonicalize its parent
                let parent = target.parent().unwrap_or(Path::new("/"));
                parent.canonicalize().map_err(|e| {
                    ClawError::Execution(format!("无法访问路径 '{}': {}", target.display(), e))
                })?
            }
            Err(e) => return Err(ClawError::Execution(format!("无法访问路径 '{}': {}", target.display(), e))),
        };

        let allowed = cfg
            .allowed_dirs
            .iter()
            .map(|d| PathBuf::from(d).canonicalize())
            .filter_map(|r| r.ok())
            .any(|allowed_dir| canonical_target.starts_with(&allowed_dir));

        if !allowed {
            return Err(ClawError::Validation(format!(
                "权限不足：路径 '{}' 不在允许的目录内。允许的目录: {}",
                canonical_target.display(),
                cfg.allowed_dirs.join(", ")
            )));
        }

        match operation {
            "read" => op_read(&canonical_target),
            "write" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                op_write(&canonical_target, content)
            }
            "list" => op_list(&canonical_target),
            other => Err(ClawError::Validation(format!("不支持的操作: '{}'。支持: read, write, list", other))),
        }
    }
}

fn op_read(path: &Path) -> Result<String, ClawError> {
    if !path.exists() {
        return Err(ClawError::NotFound(format!("文件不存在: {}", path.display())));
    }
    if !path.is_file() {
        return Err(ClawError::Validation(format!("不是文件: {}", path.display())));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // Truncate very large files to avoid excessive context
    let max_chars = 5000;
    let preview: String = content.chars().take(max_chars).collect();
    let mut result = format!("📄 {} ({} 字符):\n\n", path.display(), content.len());

    if content.len() > max_chars {
        result.push_str(&preview);
        result.push_str(&format!(
            "\n\n... (仅显示前 {} 字符，文件共 {} 字符)",
            max_chars, content.len()
        ));
    } else {
        result.push_str(&content);
    }

    Ok(result)
}

fn op_write(path: &Path, content: &str) -> Result<String, ClawError> {
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    std::fs::write(path, content)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    // Show a preview in the response
    let preview: String = content.chars().take(200).collect();
    let mut result = format!("✅ 已写入 {} ({} 字符)\n", path.display(), content.len());

    if !content.is_empty() {
        result.push_str("\n预览:\n");
        if content.len() > 200 {
            result.push_str(&format!("{}\n...", preview));
        } else {
            result.push_str(&preview);
        }
    }

    Ok(result)
}

fn op_list(path: &Path) -> Result<String, ClawError> {
    if !path.exists() {
        return Err(ClawError::NotFound(format!("目录不存在: {}", path.display())));
    }
    if !path.is_dir() {
        return Err(ClawError::Validation(format!("不是目录: {}", path.display())));
    }

    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("读取目录失败: {}", e))?;

    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }

        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            dirs.push(name);
        } else {
            files.push(name);
        }
    }

    dirs.sort();
    files.sort();

    if dirs.is_empty() && files.is_empty() {
        return Ok(format!("📁 {} (空目录)", path.display()));
    }

    let mut result = format!("📁 {}:\n\n", path.display());

    for d in &dirs {
        result.push_str(&format!("  📂 {}/\n", d));
    }
    for f in &files {
        result.push_str(&format!("  📄 {}\n", f));
    }

    result.push_str(&format!(
        "\n共 {} 个目录, {} 个文件",
        dirs.len(),
        files.len()
    ));

    Ok(result)
}
