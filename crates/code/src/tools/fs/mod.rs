pub mod read;
pub mod write;
pub mod edit;
pub mod glob;
pub mod grep;
pub mod ls;

use std::path::{Path, PathBuf};

pub use read::ReadTool;
pub use write::WriteTool;
pub use edit::EditTool;
pub use glob::GlobTool;
pub use grep::GrepTool;
pub use ls::LsTool;

/// Validate path is within workspace and return safe path.
pub fn resolve_safe_path(path: &str) -> anyhow::Result<PathBuf> {
    let p = Path::new(path);
    if p.components().any(|c| c.as_os_str() == "..") {
        anyhow::bail!("Path traversal detected: {}", path);
    }
    let cwd = std::env::current_dir()?;
    // Resolve relative paths against cwd
    let absolute = if p.is_relative() {
        cwd.join(p)
    } else {
        p.to_path_buf()
    };
    // For existing paths, canonicalize and verify
    if absolute.exists() {
        let canonical = absolute.canonicalize()?;
        if !canonical.starts_with(&cwd) {
            anyhow::bail!("Access denied: path outside workspace: {}", path);
        }
        return Ok(canonical);
    }
    // For non-existent paths, verify parent directory
    if let Some(parent) = absolute.parent()
        && parent.exists()
    {
        let canonical_parent = parent.canonicalize()?;
        if !canonical_parent.starts_with(&cwd) {
            anyhow::bail!("Access denied: path outside workspace: {}", path);
        }
    }
    Ok(absolute)
}

#[allow(dead_code)]
pub(crate) fn check_path(path: &str) -> anyhow::Result<()> {
    check_path_impl(path, false)
}

pub(crate) async fn check_path_async(path: &str) -> anyhow::Result<()> {
    let p = Path::new(path);
    let cwd = std::env::current_dir()?;
    let absolute = if p.is_relative() { cwd.join(p) } else { p.to_path_buf() };

    // Sync checks first (no I/O needed for traversal / relative resolution)
    if p.components().any(|c| c.as_os_str() == "..") {
        anyhow::bail!("Path traversal detected: {}", path);
    }

    if absolute.try_exists().unwrap_or(false) {
        let canonical = tokio::fs::canonicalize(&absolute).await?;
        if !canonical.starts_with(&cwd) {
            anyhow::bail!("Access denied: path outside workspace: {}", path);
        }
    } else if let Some(parent) = absolute.parent()
        && parent.try_exists().unwrap_or(false)
    {
        let canonical_parent = tokio::fs::canonicalize(parent).await?;
        if !canonical_parent.starts_with(&cwd) {
            anyhow::bail!("Access denied: path outside workspace: {}", path);
        }
    }
    Ok(())
}

fn check_path_impl(path: &str, _async_version: bool) -> anyhow::Result<()> {
    let p = Path::new(path);
    if p.components().any(|c| c.as_os_str() == "..") {
        anyhow::bail!("Path traversal detected: {}", path);
    }
    let cwd = std::env::current_dir()?;
    let absolute = if p.is_relative() { cwd.join(p) } else { p.to_path_buf() };
    if absolute.exists() {
        let canonical = absolute.canonicalize()?;
        if !canonical.starts_with(&cwd) {
            anyhow::bail!("Access denied: path outside workspace: {}", path);
        }
        return Ok(());
    }
    if let Some(parent) = absolute.parent()
        && parent.exists()
    {
        let canonical_parent = parent.canonicalize()?;
        if !canonical_parent.starts_with(&cwd) {
            anyhow::bail!("Access denied: path outside workspace: {}", path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Tool;

    #[test]
    fn test_resolve_safe_path_rejects_traversal() {
        let err = resolve_safe_path("../etc/passwd").unwrap_err().to_string();
        assert!(err.contains("Path traversal") || err.contains("outside workspace"));
    }

    #[test]
    fn test_resolve_safe_path_rejects_outside_workspace() {
        let result = resolve_safe_path("/tmp");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_safe_path_accepts_relative() {
        let result = resolve_safe_path(".");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_path_rejects_absolute_outside() {
        let result = check_path("/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_path_rejects_traversal() {
        let result = check_path("foo/../../etc");
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_names() {
        assert_eq!(ReadTool.name(), "read");
        assert_eq!(WriteTool.name(), "write");
        assert_eq!(EditTool.name(), "edit");
        assert_eq!(GlobTool.name(), "glob");
        assert_eq!(GrepTool.name(), "grep");
        assert_eq!(LsTool.name(), "ls");
    }
}
