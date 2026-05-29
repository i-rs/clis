/// Shorten a path by replacing the home directory prefix with `~`.
pub fn short_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if let Some(rest) = path.strip_prefix(&*home_str) {
            return format!("~{}", rest);
        }
    }
    path.to_string()
}
