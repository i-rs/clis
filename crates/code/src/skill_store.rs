use crate::config;
use std::path::PathBuf;

/// A parsed skill from a SKILL.md file.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
    #[allow(dead_code)]
    pub path: PathBuf,
}

/// Scans and loads skills from `~/.i-rs/code/skills/`.
pub struct SkillStore {
    skills_dir: PathBuf,
}

impl Default for SkillStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillStore {
    pub fn new() -> Self {
        Self {
            skills_dir: config::i_rs_code_dir().join("skills"),
        }
    }

    /// Directory where skills are stored.
    pub fn dir(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// List all installed skills (name + description only).
    pub fn list(&self) -> Vec<Skill> {
        self.load_all()
    }

    /// Find a skill by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<Skill> {
        let name_lower = name.to_lowercase();
        self.list().into_iter().find(|s| s.name.to_lowercase() == name_lower)
    }

    /// Load all skills by scanning the skills directory.
    fn load_all(&self) -> Vec<Skill> {
        let dir = &self.skills_dir;
        if !dir.exists() {
            return Vec::new();
        }

        let mut skills = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = match path.file_stem().and_then(|n| n.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if let Some(skill) = parse_skill_md(&name, &content, &path) {
                    skills.push(skill);
                }
            }
        }
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        skills
    }
}

/// Parse a SKILL.md file: extract YAML frontmatter (`---` delimited), then the body.
fn parse_skill_md(fallback_name: &str, content: &str, path: &PathBuf) -> Option<Skill> {
    let content = content.trim();

    // Check for YAML frontmatter between --- markers
    let (frontmatter, body) = if content.starts_with("---") {
        let after_first = content.trim_start_matches("---").trim_start();
        if let Some(end) = after_first.find("\n---") {
            let yaml_block = &after_first[..end];
            let body = after_first[end + 4..].trim();
            (Some(yaml_block), body)
        } else {
            (None, content)
        }
    } else {
        (None, content)
    };

    // Parse frontmatter — only name and description are required
    let name = frontmatter
        .and_then(|y| parse_yaml_scalar(y, "name"))
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| fallback_name.to_lowercase());

    let description = frontmatter
        .and_then(|y| parse_yaml_scalar(y, "description"))
        .unwrap_or_default();

    Some(Skill {
        name: name.to_string(),
        description: description.to_string(),
        content: body.to_string(),
        path: path.clone(),
    })
}

/// Minimal YAML key-value parser (no dependency).
/// Handles both inline (`key: value`) and block literal (`key: |` with indented next lines).
fn parse_yaml_scalar<'a>(yaml: &'a str, key: &str) -> Option<String> {
    let prefix = format!("{}:", key);
    let lines: Vec<&str> = yaml.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let after_colon = rest.trim();
            // Block literal: key: |
            if after_colon == "|" {
                let mut value = String::new();
                for next_line in lines.iter().skip(i + 1) {
                    let trimmed = next_line.trim();
                    if trimmed.is_empty() || !next_line.starts_with(' ') {
                        break;
                    }
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(trimmed);
                }
                return Some(value);
            }
            // Inline value
            let val = after_colon.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Create a new SKILL.md file with the given name and description.
pub fn create_skill(name: &str, description: &str) -> anyhow::Result<PathBuf> {
    let store = SkillStore::new();
    let dir = store.dir();
    std::fs::create_dir_all(dir)?;

    let path = dir.join(format!("{}.md", name.to_lowercase()));
    if path.exists() {
        anyhow::bail!("Skill '{}' already exists at {:?}", name, path);
    }

    let content = format!(
        "---\nname: {name}\ndescription: |\n  {description}\n---\n\n# {name}\n\nAdd your skill instructions here.\n"
    );
    std::fs::write(&path, content)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_frontmatter() {
        let content = "\
---
name: my-skill
description: A test skill
---
This is the body content.";
        let skill = parse_skill_md("fallback", content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.content, "This is the body content.");
    }

    #[test]
    fn test_parse_without_frontmatter() {
        let content = "Just body content\nwithout frontmatter";
        let skill = parse_skill_md("fallback-name", content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(skill.name, "fallback-name");
        assert_eq!(skill.description, "");
        assert_eq!(skill.content, "Just body content\nwithout frontmatter");
    }

    #[test]
    fn test_parse_yaml_scalar_inline() {
        let yaml = "name: hello\ndescription: world\nfoo: bar";
        assert_eq!(parse_yaml_scalar(yaml, "name").as_deref(), Some("hello"));
        assert_eq!(parse_yaml_scalar(yaml, "description").as_deref(), Some("world"));
        assert_eq!(parse_yaml_scalar(yaml, "missing"), None);
    }

    #[test]
    fn test_parse_yaml_scalar_quoted() {
        let yaml = "name: \"hello world\"";
        assert_eq!(parse_yaml_scalar(yaml, "name").as_deref(), Some("hello world"));
    }

    #[test]
    fn test_parse_yaml_scalar_block_literal() {
        let yaml = "name: test\ndescription: |\n  First line\n  Second line";
        assert_eq!(parse_yaml_scalar(yaml, "description").as_deref(), Some("First line Second line"));
    }

    #[test]
    fn test_create_skill_roundtrip() {
        use std::fs;
        let tmp = std::env::temp_dir().join("i-rs-code-test-skills");
        let _ = fs::remove_dir_all(&tmp);

        // Override config dir via env
        unsafe { std::env::set_var("I_RS_CODE_DIR", tmp.to_str().unwrap()); }

        let path = create_skill("test-roundtrip", "Roundtrip test").unwrap();
        assert!(path.exists());

        let store = SkillStore::new();
        let skills = store.list();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-roundtrip");

        let fetched = store.get("test-roundtrip").unwrap();
        assert_eq!(fetched.name, "test-roundtrip");
        assert_eq!(fetched.description, "Roundtrip test");

        let _ = fs::remove_dir_all(&tmp);
        unsafe { std::env::remove_var("I_RS_CODE_DIR"); }
    }
}
