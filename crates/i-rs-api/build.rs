use std::fs;
use std::path::Path;

fn main() {
    let out_path = Path::new("src/routes.rs");

    let mut mods: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir("src/routes") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                && name != "mod"
            {
                mods.push(name.to_string());
            }
        }
    }

    mods.sort();

    let content: String = mods
        .iter()
        .map(|name| format!("pub mod {};\n", name))
        .collect();

    // Only write when content changes, to avoid unnecessary mtime updates
    // that would trigger crate recompilation
    let should_write = match fs::read_to_string(out_path) {
        Ok(existing) => existing != content,
        Err(_) => true,
    };

    if should_write {
        fs::write(out_path, content).expect("Failed to write routes.rs");
    }

    // Re-run build script only when route files change
    println!("cargo:rerun-if-changed=src/routes");
}
