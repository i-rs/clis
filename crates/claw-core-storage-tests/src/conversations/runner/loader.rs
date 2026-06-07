use super::Script;
use std::path::Path;

/// Load a conversation script from a directory containing script.json.
pub fn load_script(dir: &Path) -> anyhow::Result<Script> {
    let path = dir.join("script.json");
    if !path.exists() {
        anyhow::bail!("script.json not found in {:?}", dir);
    }
    let content = std::fs::read_to_string(&path)?;
    let script: Script = serde_json::from_str(&content)?;
    Ok(script)
}

/// Find all script directories under a given root.
pub fn discover_scripts(root: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut scripts = Vec::new();
    if !root.exists() {
        return Ok(scripts);
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("script.json").exists() {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent() {
        let result = load_script(Path::new("/tmp/nonexistent_script_dir"));
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let scripts = discover_scripts(dir.path()).unwrap();
        assert!(scripts.is_empty());
    }
}
