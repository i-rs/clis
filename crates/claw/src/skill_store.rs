use std::path::PathBuf;

/// A single skill entry with name and content (simplified version).
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct SkillEntry {
    pub name: String,
    pub content: String,
}

/// Full skill definition including metadata from TOML frontmatter.
///
/// Skills with `parameters` defined are registered as callable tools
/// (prefixed `skill_{name}`) in addition to being injected into the
/// system prompt via `format_skills()`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    /// JSON Schema for tool parameters (None = pure instruction skill).
    pub parameters: Option<serde_json::Value>,
    /// Skill content after frontmatter (empty if no content).
    pub content: String,
}

// ── Frontmatter parsing ──

/// Parse TOML frontmatter from a skill file content.
///
/// Expected format:
/// ```text
/// ---
/// description = "..."
/// [parameters]
/// type = "object"
/// ---
/// content...
/// ```
pub(crate) fn parse_frontmatter(content: &str) -> (Option<toml::Value>, &str) {
    let content = content.trim_start();
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        return (None, content);
    }

    // Find the closing ---
    let after_opener = if content.starts_with("---\r\n") {
        &content[5..]
    } else {
        &content[4..]
    };

    if let Some(end_pos) = after_opener.find("\n---")
        .or_else(|| after_opener.find("\r\n---"))
    {
        let toml_str = &after_opener[..end_pos];
        let rest = if after_opener[end_pos..].starts_with("\r\n---") {
            &after_opener[end_pos + 5..]
        } else {
            &after_opener[end_pos + 4..]
        };
        // Skip the trailing newline after closing ---
        let rest = rest.trim_start();

        match toml::from_str::<toml::Value>(toml_str) {
            Ok(toml_val) => (Some(toml_val), rest),
            Err(_) => (None, content),
        }
    } else {
        (None, content) // No closing ---, treat as plain content
    }
}

/// Build a `SkillDefinition` from frontmatter + file content.
fn build_definition(name: &str, raw_content: &str) -> SkillDefinition {
    let (frontmatter, body) = parse_frontmatter(raw_content);
    let description = frontmatter
        .as_ref()
        .and_then(|t| t.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or(name)
        .to_string();
    let parameters = frontmatter
        .as_ref()
        .and_then(|t| t.get("parameters"))
        .and_then(|v| {
            // Convert toml::Value → serde_json::Value
            serde_json::to_value(v).ok()
        });
    let content = body.trim().to_string();
    SkillDefinition {
        name: name.to_string(),
        description,
        parameters,
        content,
    }
}

/// Loads and formats user-defined skills from `~/.i-rs-claw/skills/`.
///
/// Skills are `.md` files that inject custom behavior instructions into the
/// system prompt. Each file name (without `.md`) becomes the skill name,
/// and its content is the skill definition that guides the LLM.
///
/// Files may contain optional TOML frontmatter (between `---` markers)
/// to declare metadata. Skills with `parameters` in frontmatter are
/// registered as callable tools (see `executable_skills()`).
#[derive(Debug, Clone)]
pub struct SkillStore {
    skills_dir: PathBuf,
}

impl SkillStore {
    /// Create skill store for a specific agent.
    /// "default" reads from legacy `claw_dir/skills`; others from
    /// `claw_dir/agents/{agent_id}/skills`.
    pub fn for_agent(claw_dir: &PathBuf, agent_id: &str) -> Self {
        let skills_dir = if agent_id == "default" {
            claw_dir.join("skills")
        } else {
            claw_dir.join("agents").join(agent_id).join("skills")
        };
        Self { skills_dir }
    }

    /// Get the skills directory path.
    pub fn path(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// Read a specific skill by name.
    pub fn get_skill(&self, name: &str) -> Option<SkillDefinition> {
        let path = self.skills_dir.join(format!("{}.md", name));
        if !path.exists() {
            return None;
        }
        let raw = std::fs::read_to_string(path).ok()?;
        Some(build_definition(name, &raw))
    }

    /// Install a new skill (create .md file with optional content).
    /// Creates the skills directory if it doesn't exist.
    pub fn install(&self, name: &str, content: &str) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.skills_dir)?;
        let path = self.skills_dir.join(format!("{}.md", name));
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Remove a skill by name.
    pub fn remove(&self, name: &str) -> anyhow::Result<()> {
        let path = self.skills_dir.join(format!("{}.md", name));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Return all skills that have parameters defined (callable as tools).
    pub fn executable_skills(&self) -> Vec<SkillDefinition> {
        let dir = match std::fs::read_dir(&self.skills_dir) {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let mut skills: Vec<SkillDefinition> = dir
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false) && e.path().is_file())
            .filter_map(|e| {
                let path = e.path();
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())?;
                let raw = std::fs::read_to_string(&path).ok()?;
                let def = build_definition(name, &raw);
                // Only include skills with parameters
                if def.parameters.is_some() {
                    Some(def)
                } else {
                    None
                }
            })
            .collect();
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        skills
    }

    /// Generate a skill template with TOML frontmatter.
    pub fn skill_template(name: &str) -> String {
        format!(
            r#"---
description = "{}"

[parameters]
type = "object"

[parameters.properties]
---

编写技能指令，指导 AI 在合适场景下遵循此技能。
使用 Markdown 格式书写。
"#,
            name
        )
    }

    /// Format all skill files as a system prompt layer.
    /// Returns empty string when no skills exist or the directory is missing.
    ///
    /// Skills with frontmatter use `description` as the heading (if present)
    /// and exclude the frontmatter block from the displayed content.
    pub fn format_skills(&self) -> String {
        let dir = match std::fs::read_dir(&self.skills_dir) {
            Ok(d) => d,
            Err(_) => return String::new(),
        };

        let mut entries: Vec<_> = dir
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false) && e.path().is_file())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        if entries.is_empty() {
            return String::new();
        }

        let mut result = String::from("## 用户技能\n\n");
        result.push_str("以下是用户定义的自定义技能指令，请在对话中遵循这些指导：\n");

        for entry in &entries {
            let path = entry.path();
            let skill_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            if let Ok(raw) = std::fs::read_to_string(&path) {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let (frontmatter, body) = parse_frontmatter(trimmed);
                let heading = frontmatter
                    .as_ref()
                    .and_then(|t| t.get("description"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(skill_name);
                let display_content = if body.is_empty() { trimmed } else { body };
                result.push_str(&format!("\n### {}\n{}\n", heading, display_content));
            }
        }

        result
    }

    /// Return list of available skill names.
    #[allow(dead_code)]
    pub fn skill_names(&self) -> Vec<String> {
        let dir = match std::fs::read_dir(&self.skills_dir) {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let mut names: Vec<String> = dir
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false) && e.path().is_file())
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect();
        names.sort();
        names
    }

    /// Return list of skills with their full content (simplified entry).
    /// Includes raw content (with frontmatter if present).
    #[allow(dead_code)]
    pub fn list_skills(&self) -> Vec<SkillEntry> {
        let dir = match std::fs::read_dir(&self.skills_dir) {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let mut entries: Vec<SkillEntry> = dir
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false) && e.path().is_file())
            .filter_map(|e| {
                let name = e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())?;
                let content = std::fs::read_to_string(e.path()).ok()?;
                Some(SkillEntry { name, content })
            })
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a temporary directory for testing, returning its path.
    fn temp_skills_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("i-rs-claw-test")
            .join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("failed to create temp skills dir");
        dir
    }

    /// Write a skill .md file into the given directory.
    fn install_skill(dir: &PathBuf, name: &str, content: &str) {
        let path = dir.join(format!("{}.md", name));
        fs::write(&path, content).expect("failed to write skill file");
    }

    // ── Frontmatter parsing tests ──

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "这是一个纯技能内容。";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_frontmatter_with_metadata() {
        let content = concat!(
            "---\n",
            "description = \"格式化偏好\"\n",
            "[parameters]\n",
            "type = \"object\"\n",
            "[parameters.properties.format]\n",
            "type = \"string\"\n",
            "---\n",
            "\n",
            "当用户请求输出时使用 Markdown 格式。\n",
        );
        eprintln!("DEBUG content repr: {:?}", content);
        eprintln!("DEBUG starts_with: {}", content.starts_with("---\n"));
        let (fm, body) = parse_frontmatter(content);
        eprintln!("DEBUG fm: {:?}", fm);
        eprintln!("DEBUG body starts: {:?}", body.chars().take(30).collect::<String>());
        assert!(fm.is_some(), "should parse frontmatter");
        let t = fm.unwrap();
        assert_eq!(t.get("description").and_then(|v| v.as_str()), Some("格式化偏好"));
        assert!(body.contains("Markdown"), "body should contain content after frontmatter");
    }

    #[test]
    fn test_parse_frontmatter_no_closing_delim() {
        let content = "---\ndescription = \"broken\"\n\n实际内容";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_none(), "no closing --- should fall back to plain");
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_frontmatter_invalid_toml() {
        let content = "---\ninvalid = [[[\n---\n\n内容";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_none(), "invalid TOML should fall back to plain");
        assert_eq!(body, content);
    }

    // ── SkillDefinition building tests ──

    #[test]
    fn test_build_definition_with_frontmatter() {
        let raw = concat!(
            "---\n",
            "description = \"格式化偏好\"\n",
            "[parameters]\n",
            "type = \"object\"\n",
            "---\n",
            "\n",
            "当用户请求时使用 Markdown。\n",
        );
        let def = build_definition("format-pref", raw);
        assert_eq!(def.name, "format-pref");
        assert_eq!(def.description, "格式化偏好");
        assert!(def.parameters.is_some());
        assert!(def.content.contains("Markdown"));
    }

    #[test]
    fn test_build_definition_without_frontmatter() {
        let raw = "当用户请求时，默认使用 Markdown。";
        let def = build_definition("format-pref", raw);
        assert_eq!(def.name, "format-pref");
        assert_eq!(def.description, "format-pref"); // falls back to name
        assert!(def.parameters.is_none());
        assert_eq!(def.content, raw);
    }

    #[test]
    fn test_build_definition_empty_content() {
        let raw = "";
        let def = build_definition("empty-skill", raw);
        assert_eq!(def.name, "empty-skill");
        assert_eq!(def.description, "empty-skill");
        assert!(def.parameters.is_none());
        assert!(def.content.is_empty());
    }

    // ── get_skill tests ──

    #[test]
    fn test_get_skill_found() {
        let dir = temp_skills_dir("get_skill_found");
        install_skill(&dir, "test-skill", "这是一个测试技能。");
        let store = SkillStore { skills_dir: dir.clone() };
        let skill = store.get_skill("test-skill");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().name, "test-skill");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_skill_not_found() {
        let dir = temp_skills_dir("get_skill_not_found");
        let store = SkillStore { skills_dir: dir.clone() };
        assert!(store.get_skill("nonexistent").is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    // ── install / remove tests ──

    #[test]
    fn test_install_and_remove_skill() {
        let dir = temp_skills_dir("install_remove");
        let store = SkillStore { skills_dir: dir.clone() };

        // Install
        assert!(store.install("my-skill", "测试内容").is_ok());
        let path = dir.join("my-skill.md");
        assert!(path.exists());

        // Verify installed
        let skill = store.get_skill("my-skill");
        assert!(skill.is_some());

        // Remove
        assert!(store.remove("my-skill").is_ok());
        assert!(!path.exists());

        // Remove non-existent (should not error)
        assert!(store.remove("nonexistent").is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_install_creates_directory() {
        let dir = temp_skills_dir("install_creates_dir");
        let sub = dir.join("nested");
        let store = SkillStore { skills_dir: sub.clone() };
        assert!(!sub.exists());
        assert!(store.install("test", "content").is_ok());
        assert!(sub.exists());
        assert!(sub.join("test.md").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    // ── executable_skills tests ──

    #[test]
    fn test_executable_skills_only_with_parameters() {
        let dir = temp_skills_dir("executable_skills");
        // Skill with parameters → executable
        install_skill(&dir, "param-skill", r#"---
description = "有参数"
[parameters]
type = "object"
---

内容。
"#);
        // Skill without parameters → not executable
        install_skill(&dir, "plain-skill", "纯指令。");
        // Another with parameters
        install_skill(&dir, "another", r#"---
description = "另一个"
[parameters.properties.x]
type = "string"
---

更多内容。
"#);

        let store = SkillStore { skills_dir: dir.clone() };
        let executables = store.executable_skills();
        assert_eq!(executables.len(), 2);
        assert_eq!(executables[0].name, "another");
        assert_eq!(executables[1].name, "param-skill");

        let _ = fs::remove_dir_all(&dir);
    }

    // ── Existing tests (preserved) ──

    #[test]
    fn test_nonexistent_directory() {
        let store = SkillStore {
            skills_dir: PathBuf::from("/tmp/__i_rs_claw_test_nonexistent__"),
        };
        assert_eq!(store.format_skills(), "");
        assert_eq!(store.skill_names(), Vec::<String>::new());
        assert!(store.list_skills().is_empty());
    }

    #[test]
    fn test_empty_directory() {
        let dir = temp_skills_dir("empty_dir");
        let store = SkillStore { skills_dir: dir.clone() };
        assert_eq!(store.format_skills(), "");
        assert_eq!(store.skill_names(), Vec::<String>::new());
        assert!(store.list_skills().is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_install_and_discover_single_skill() {
        let dir = temp_skills_dir("single_skill");

        // Install a skill
        install_skill(&dir, "my-skill", "当用户提到 '帮我总结' 时，自动调用总结流程。");

        let store = SkillStore { skills_dir: dir.clone() };

        // Discover by name
        let names = store.skill_names();
        assert_eq!(names, vec!["my-skill"]);

        // Discover with content
        let entries = store.list_skills();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "my-skill");
        assert!(
            entries[0].content.contains("帮我总结"),
            "expected content to contain the skill text"
        );

        // Format for system prompt
        let formatted = store.format_skills();
        assert!(formatted.contains("## 用户技能"), "should have header");
        assert!(formatted.contains("my-skill"), "should have skill name");
        assert!(formatted.contains("帮我总结"), "should have skill content");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_install_and_discover_multiple_skills() {
        let dir = temp_skills_dir("multi_skill");

        install_skill(&dir, "alphabetize", "把所有列表按字母序排列。");
        install_skill(&dir, "translate-en", "当用户说英文时自动翻译成中文。");
        install_skill(&dir, "z-skills", "z结尾的排序校验。");

        let store = SkillStore { skills_dir: dir.clone() };

        // Names should be sorted alphabetically
        let names = store.skill_names();
        assert_eq!(names, vec!["alphabetize", "translate-en", "z-skills"]);

        // Full entries should also be sorted
        let entries = store.list_skills();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name, "alphabetize");
        assert_eq!(entries[1].name, "translate-en");
        assert_eq!(entries[2].name, "z-skills");

        // Format should include all skills in order
        let formatted = store.format_skills();
        assert!(formatted.contains("alphabetize"));
        assert!(formatted.contains("translate-en"));
        assert!(formatted.contains("z-skills"));
        // Skills should appear in order
        let alphabetize_pos = formatted.find("alphabetize").unwrap();
        let z_pos = formatted.find("z-skills").unwrap();
        assert!(
            alphabetize_pos < z_pos,
            "skills should be in alphabetical order in format_skills()"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ignores_non_md_files() {
        let dir = temp_skills_dir("non_md");

        install_skill(&dir, "valid-skill", "这是一个有效的技能。");
        fs::write(dir.join("notes.txt"), "这不是技能文件").unwrap();
        fs::write(dir.join("data.json"), r#"{"key": "value"}"#).unwrap();
        fs::write(dir.join("README"), "没有扩展名的文件").unwrap();

        let store = SkillStore { skills_dir: dir.clone() };

        let names = store.skill_names();
        assert_eq!(names, vec!["valid-skill"], "only .md files should be counted");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_skips_empty_skill_files() {
        let dir = temp_skills_dir("empty_skills");

        install_skill(&dir, "empty", "");
        install_skill(&dir, "only-whitespace", "   \n  \n  ");
        install_skill(&dir, "real-skill", "这是一个真实的技能。");

        let store = SkillStore { skills_dir: dir.clone() };

        let formatted = store.format_skills();
        assert!(!formatted.contains("empty"), "empty skills should be skipped");
        assert!(!formatted.contains("only-whitespace"), "whitespace-only skills should be skipped");
        assert!(formatted.contains("real-skill"), "non-empty skills should be included");

        // list_skills should still include empty/whitespace files
        let entries = store.list_skills();
        assert_eq!(entries.len(), 3, "list_skills returns all .md files regardless of content");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_for_agent_default_path() {
        let claw_dir = PathBuf::from("/tmp/__i_rs_claw_test_path__");
        let store = SkillStore::for_agent(&claw_dir, "default");
        assert_eq!(store.skills_dir, claw_dir.join("skills"));
    }

    #[test]
    fn test_for_agent_custom_path() {
        let claw_dir = PathBuf::from("/tmp/__i_rs_claw_test_path__");
        let store = SkillStore::for_agent(&claw_dir, "my-agent");
        assert_eq!(store.skills_dir, claw_dir.join("agents").join("my-agent").join("skills"));
    }
}
