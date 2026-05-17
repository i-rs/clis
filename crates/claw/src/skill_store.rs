use std::path::PathBuf;

/// A single skill entry with name and content.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillEntry {
    pub name: String,
    pub content: String,
}

/// Loads and formats user-defined skills from `~/.i-rs-claw/skills/`.
///
/// Skills are `.md` files that inject custom behavior instructions into the
/// system prompt. Each file name (without `.md`) becomes the skill name,
/// and its content is the skill definition that guides the LLM.
pub struct SkillStore {
    skills_dir: PathBuf,
}

impl SkillStore {
    pub fn new(claw_dir: PathBuf) -> Self {
        Self {
            skills_dir: claw_dir.join("skills"),
        }
    }

    /// Format all skill files as a system prompt layer.
    /// Returns empty string when no skills exist or the directory is missing.
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

            if let Ok(content) = std::fs::read_to_string(&path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    result.push_str(&format!("\n### {}\n{}\n", skill_name, trimmed));
                }
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

    /// Return list of skills with their full content.
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
