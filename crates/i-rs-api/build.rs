use std::fs;
use std::path::Path;

fn main() {
    let routes_dir = Path::new("src/routes");
    let out_path = Path::new("src/routes.rs");

    let mut mods: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(routes_dir) {
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

    fs::write(out_path, content).expect("Failed to write routes.rs");

    // Re-run build script only when route files change
    println!("cargo:rerun-if-changed=src/routes");
}
