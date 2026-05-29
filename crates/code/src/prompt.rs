use crate::config::ProjectInfo;
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

pub const SYSTEM: &str = "\
You are i-rs-code, an expert coding AI agent.

## CRITICAL RULE -- You MUST use tools
You are a tool-using AI. You can NOT do anything by just talking. Every action MUST go through a tool call.
If you only respond with text, nothing happens — no files are created, no code is written, no commands run.
IMMEDIATELY call the appropriate tool. Do NOT explain what you will do — just do it.
Your internal reasoning is for analysis only. After reasoning, you MUST output tool calls to take action.

Available tools: read, write, edit, grep, glob, ls, bash, git, web_fetch, web_search, create_crate, verify, lsp_diagnostics, lsp_definition, lsp_references, lsp_hover, lsp_rename, lsp_symbols, lsp_completion.

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

pub fn build_system_prompt(project_info: &ProjectInfo) -> String {
    let mut prompt = SYSTEM.to_string();

    // Inject available skills for progressive disclosure
    let store = crate::skill_store::SkillStore::new();
    let skills = store.list();
    if !skills.is_empty() {
        prompt.push_str("\n\n## Available Skills\n");
        prompt.push_str("Skills provide specialized instructions. Use the `skill` tool to load them on demand.\n");
        prompt.push_str(&format!("Total: {} skill(s) installed.\n", skills.len()));
        prompt.push_str("To see all skills, call the `skill` tool with action='list'.\n");
        prompt.push_str("To load a skill, call `skill` with action='get' and name='<skill-name>'.\n");
    }

    if let Some(content) = &project_info.agents_md_content {
        let truncated = if content.len() > 8000 {
            let head: String = content.chars().take(4000).collect();
            let tail: String = content.chars().rev().take(2000).collect::<String>().chars().rev().collect();
            format!("{}\n\n[...truncated...]\n\n{}", head, tail)
        } else {
            content.clone()
        };
        prompt.push_str("\n\n## Project Guidelines (from AGENTS.md)\n");
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
    fn test_system_prompt_contains_tools() {
        assert!(SYSTEM.contains("read"));
        assert!(SYSTEM.contains("write"));
        assert!(SYSTEM.contains("edit"));
    }
}
