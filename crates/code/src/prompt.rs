use crate::config::ProjectInfo;
use std::path::PathBuf;
use std::sync::Mutex;

struct BuildContextCache {
    key: String,
    dir: String,
    branch: String,
    git_status: String,
    workspace_crates: String,
}

static CONTEXT_CACHE: Mutex<Option<BuildContextCache>> = Mutex::new(None);

fn cache_key() -> String {
    let dir = std::env::current_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_default();
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|h| !h.is_empty())
        .unwrap_or_default();
    format!("{}::{}", dir, head)
}

fn build_cached_context(project_info: &ProjectInfo) -> BuildContextCache {
    let dir = std::env::current_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_default();

    let branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|b| !b.is_empty())
        .unwrap_or_default();

    let git_status = std::process::Command::new("git")
        .args(["status", "--short"])
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_default();

    let workspace_crates = if project_info.has_cargo {
        std::process::Command::new("cargo")
            .args(["metadata", "--format-version=1", "--no-deps"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                serde_json::from_str::<serde_json::Value>(&stdout).ok()
            })
            .and_then(|meta| meta["workspace_members"].as_array().cloned())
            .map(|members| {
                let mut s = String::new();
                s.push_str(&format!("Workspace crates ({}):\n", members.len()));
                for member in members.iter().take(15) {
                    if let Some(name) = member.as_str() {
                        s.push_str(&format!("  {}\n", name));
                    }
                }
                s
            })
            .unwrap_or_default()
    } else {
        String::new()
    };

    BuildContextCache { key: cache_key(), dir, branch, git_status, workspace_crates }
}

pub fn build_context(project_info: &ProjectInfo) -> String {
    let key = cache_key();
    let mut cache_guard = CONTEXT_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let should_rebuild = match cache_guard.as_ref() {
        Some(cached) => cached.key != key,
        None => true,
    };
    if should_rebuild {
        *cache_guard = Some(build_cached_context(project_info));
    }
    let cache = cache_guard.as_ref().unwrap();

    let mut ctx = String::new();

    if !cache.dir.is_empty() {
        ctx.push_str(&format!("Working directory: {}\n", cache.dir));
    }
    ctx.push_str(&format!("Project type: {}\n", project_info.project_type));
    if !cache.branch.is_empty() {
        ctx.push_str(&format!("Git branch: {}\n", cache.branch));
    }
    if !cache.git_status.is_empty() {
        let lines: Vec<&str> = cache.git_status.lines().take(20).collect();
        ctx.push_str(&format!("Git status:\n{}\n", lines.join("\n")));
    }
    if !cache.workspace_crates.is_empty() {
        ctx.push_str(&cache.workspace_crates);
    }

    ctx
}

/// Directory containing prompt files.
pub fn prompt_dir() -> PathBuf {
    crate::config::i_rs_code_dir().join("prompts")
}

/// Path to the system prompt file.
pub fn system_prompt_path() -> PathBuf {
    prompt_dir().join("system.md")
}

/// Load the system prompt — from file if it exists, otherwise from the built-in default.
/// Auto-creates the file on first access so the user can edit it.
pub fn load_system_prompt() -> String {
    let path = system_prompt_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    // Auto-create the file with the built-in default
    let _ = ensure_prompt_file();
    // Re-read from the file we just wrote; fallback if that fails
    std::fs::read_to_string(&path).unwrap_or_else(|_| DEFAULT_SYSTEM.to_string())
}

/// Write the built-in default system prompt to the file, overwriting any existing content.
pub fn write_default_prompt_file() -> anyhow::Result<PathBuf> {
    let path = system_prompt_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, DEFAULT_SYSTEM)?;
    Ok(path)
}

/// Ensure the prompt file exists (create from default if not).
pub fn ensure_prompt_file() -> anyhow::Result<PathBuf> {
    let path = system_prompt_path();
    if !path.exists() {
        write_default_prompt_file()
    } else {
        Ok(path)
    }
}

pub const DEFAULT_SYSTEM: &str = "\
You are i-rs-code, an expert coding AI agent.

## CRITICAL RULE -- You MUST use tools
You are a tool-using AI. You can NOT do anything by just talking. Every action MUST go through a tool call.
If you only respond with text, nothing happens — no files are created, no code is written, no commands run.
IMMEDIATELY call the appropriate tool. Do NOT explain what you will do — just do it.
Your internal reasoning is for analysis only. After reasoning, you MUST output tool calls to take action.

Available tools: read, write, edit, grep, glob, ls, bash, git, web_fetch, web_search, create_crate, verify, lsp_diagnostics, lsp_definition, lsp_references, lsp_hover, lsp_rename, lsp_symbols, lsp_completion, skill.

## Workflow
1. UNDERSTAND -- use read/glob/grep/ls to understand current code
2. PLAN -- think about what changes are needed, which files are affected
3. EXECUTE -- use write/edit/bash to make changes
4. VERIFY -- use `verify` tool or `bash cargo check` to verify compilation
5. FIX -- if errors, read the error output and fix iteratively

## Tool Usage Patterns
- `read <path> [offset] [limit]` -- read file contents (use offset/limit for large files)
- `write <path> <content>` -- create or overwrite a file
- `edit <path> <old_string> <new_string>` -- surgical edit (preferred over write)
- `grep <pattern> [path] [include]` -- search file contents with regex
- `glob <pattern> [path]` -- find files by pattern
- `ls [path]` -- list directory
- `bash <command> <description>` -- run shell command
- `git <args>` -- run git command
- `web_fetch <url>` -- fetch web page content
- `web_search <query>` -- search the web
- `create_crate <name> <description>` -- bootstrap a new Rust crate (i-rs project)
- `verify [package]` -- run check, clippy, test, fmt
- `lsp_diagnostics` -- get code diagnostics
- `lsp_definition` -- go to definition
- `lsp_references` -- find references
- `lsp_hover` -- hover info
- `lsp_rename` -- rename symbol
- `lsp_symbols` -- list workspace symbols
- `lsp_completion` -- get completions
- `skill` -- list or load skill documents

## Communication
- When you encounter errors, read the error output and think about what might be wrong
- Use `read` to check files before editing them
- Use `grep` to find relevant code patterns
- Output in Chinese only for the final summary
- Do NOT make up file paths — always verify with read/glob
- Do NOT make up command output — always run commands to verify
- Do NOT explain what you're about to do -- just do it
- Do NOT read entire large files when offset/limit would suffice
- Do NOT output thinking/reasoning about what tool to use -- just call it

After completing changes, briefly summarize what was done in Chinese.";

fn system_prompt_cache_key(project_info: &ProjectInfo) -> String {
    let ctx_key = cache_key();
    let mtime = system_prompt_path()
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);
    let agent_len = project_info.agents_md_content.as_ref().map(|s| s.len()).unwrap_or(0);
    format!("{}:{}:{}", ctx_key, mtime, agent_len)
}

static SYSTEM_PROMPT_CACHE: Mutex<Option<(String, String)>> = Mutex::new(None);

pub fn build_system_prompt(project_info: &ProjectInfo) -> String {
    let key = system_prompt_cache_key(project_info);
    let mut guard = SYSTEM_PROMPT_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((ref cached_key, ref cached_prompt)) = *guard {
        if cached_key == &key {
            return cached_prompt.clone();
        }
    }

    let prompt = build_system_prompt_inner(project_info);
    *guard = Some((key, prompt.clone()));
    prompt
}

fn build_system_prompt_inner(project_info: &ProjectInfo) -> String {
    let base = load_system_prompt();
    let mut prompt = base;

    prompt.push_str("\n\n## Project Context\n");
    prompt.push_str(&build_context(project_info));

    let store = crate::skill_store::SkillStore::new();
    let skills = store.list();
    if !skills.is_empty() {
        prompt.push_str("\n## Available Skills\n");
        prompt.push_str(&format!("{} skill(s) installed. Use `skill` tool with action='list' to see them, or action='get' name='<name>' to load one.\n", skills.len()));
    }

    if let Some(content) = &project_info.agents_md_content {
        let truncated = if content.len() > 8000 {
            let head: String = content.chars().take(4000).collect();
            let tail: String = content.chars().rev().take(2000).collect::<String>().chars().rev().collect();
            format!("{}\n\n[...truncated...]\n\n{}", head, tail)
        } else {
            content.clone()
        };
        prompt.push_str("\n## Project Guidelines (from AGENTS.md)\n");
        prompt.push_str(&truncated);
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProjectInfo;

    #[test]
    fn test_build_context_non_empty() {
        let info = ProjectInfo {
            project_type: "rust".into(),
            has_cargo: false,
            has_package_json: false,
            has_pyproject: false,
            has_makefile: false,
            has_agents_md: false,
            has_cursor_rules: false,
            agents_md_content: None,
            cursor_rules_content: None,
        };
        let ctx = build_context(&info);
        assert!(!ctx.is_empty(), "build_context should return a non-empty string");
        assert!(ctx.contains("rust"), "should contain project type");
    }

    #[test]
    fn test_default_system_contains_tools() {
        assert!(DEFAULT_SYSTEM.contains("read"));
        assert!(DEFAULT_SYSTEM.contains("write"));
        assert!(DEFAULT_SYSTEM.contains("skill"));
    }

    #[test]
    fn test_load_system_prompt_fallback() {
        let content = load_system_prompt();
        assert!(content.contains("i-rs-code"));
        assert!(content.contains("read"));
    }

    #[test]
    fn test_prompt_paths() {
        let dir = prompt_dir();
        assert!(dir.to_string_lossy().contains("prompts"));
        let path = system_prompt_path();
        assert!(path.to_string_lossy().ends_with("system.md"));
    }
}
