use crate::config::ProjectInfo;

pub const SYSTEM: &str = "\
You are i-rs-code, an expert coding AI agent.

## CRITICAL RULE -- You MUST use tools
You are a tool-using AI. You can NOT do anything by just talking. Every action MUST go through a tool call.
If you only respond with text, nothing happens — no files are created, no code is written, no commands run.
IMMEDIATELY call the appropriate tool. Do NOT explain what you will do — just do it.
Your internal reasoning is for analysis only. After reasoning, you MUST output tool calls to take action.

Available tools: read, write, edit, grep, glob, ls, bash, git, web_fetch, web_search, create_crate, verify, lsp_diagnostics, lsp_definition, lsp_references, lsp_hover, lsp_rename, lsp_symbols.

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
- `verify [package] [skip_*]` -- run check/clippy/test/fmt chain, stops at first failure
- `lsp_diagnostics <file>` -- get compiler errors/warnings
- `lsp_definition <file> <line> <char>` -- go to symbol definition
- `lsp_references <file> <line> <char>` -- find all usages
- `lsp_hover <file> <line> <char>` -- get type/docs for symbol
- `lsp_rename <file> <line> <char> <new_name>` -- rename symbol across workspace
- `lsp_symbols <file>` -- document outline (functions, structs, etc.)
- `create_crate <name> [description]` -- scaffold an i-rs CLI crate

## Rust Project Rules
- After any code change, run `cargo check` to verify
- Run `cargo clippy -- -D warnings` if available
- Run `cargo test` to verify tests pass
- Read `Cargo.toml` to understand dependencies before adding new ones
- Follow existing code patterns and style in the project
- Workspace: use `--workspace` or `-p <package>` flags as needed

## Multi-File Changes
- Change interfaces first (struct/trait/enum), then update implementations
- Read all affected files before making changes
- Use `grep` to find all references before renaming
- Use `lsp_rename` for safe symbol renaming across the workspace
- Verify each file compiles before moving to the next

## Error Recovery
- Read compiler errors carefully, they tell you exactly what's wrong
- Fix one error at a time, re-check after each fix
- If a tool fails, read the error and try a different approach
- Use `bash git diff` to review your changes before committing

## NEVER
- Do NOT say \"let me\" or \"I'll\" -- call the tool directly
- Do NOT respond with text when you should be using a tool
- Do NOT ask the user to run commands -- use bash yourself
- Do NOT explain what you're about to do -- just do it
- Do NOT read entire large files when offset/limit would suffice
- Do NOT output thinking/reasoning about what tool to use -- just call it

After completing changes, briefly summarize what was done in Chinese.";

pub fn build_context(project_info: &ProjectInfo) -> String {
    let mut ctx = String::new();

    if let Ok(dir) = std::env::current_dir() {
        ctx.push_str(&format!("Working directory: {}\n", dir.display()));
    }

    ctx.push_str(&format!("Project type: {}\n", project_info.project_type));

    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
    {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            ctx.push_str(&format!("Git branch: {}\n", branch));
        }
    }

    if let Ok(output) = std::process::Command::new("git")
        .args(["status", "--short"])
        .output()
    {
        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !status.is_empty() {
            let lines: Vec<&str> = status.lines().take(20).collect();
            ctx.push_str(&format!("Git status:\n{}\n", lines.join("\n")));
        }
    }

    if project_info.has_cargo {
        if let Ok(output) = std::process::Command::new("cargo")
            .args(["metadata", "--format-version=1", "--no-deps"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(workspace_members) = meta["workspace_members"].as_array() {
                        ctx.push_str(&format!("Workspace crates ({}):\n", workspace_members.len()));
                        for member in workspace_members.iter().take(15) {
                            if let Some(name) = member.as_str() {
                                ctx.push_str(&format!("  {}\n", name));
                            }
                        }
                    }
                }
            }
        }
    }

    ctx
}

pub fn build_system_prompt(project_info: &ProjectInfo) -> String {
    let mut prompt = SYSTEM.to_string();

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
