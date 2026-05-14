/// Generate `load_store()` and `save_store()` functions for a crate's storage module.
///
/// Usage in `crates/i-rs-xxx/src/storage/mod.rs`:
/// ```ignore
/// use crate::models::XxxStore;
/// i_rs_core::create_store!(XxxStore, "xxx");
/// ```
#[macro_export]
macro_rules! create_store {
    ($store_type:ty, $filename:expr) => {
        /// Load data from disk.
        pub fn load_store() -> ::anyhow::Result<$store_type> {
            let mut storage = $crate::Storage::<$store_type>::new($filename);
            storage.load()?;
            Ok(storage.data)
        }

        /// Save data to disk.
        pub fn save_store(store: &$store_type) -> ::anyhow::Result<()> {
            let storage = $crate::Storage::<$store_type>::new($filename);
            storage.save_data(store)
        }
    };
}

/// Generate the `SkillCommand` enum and `handle_skill` function for a crate.
///
/// The SKILL.md file is loaded at compile time via `include_str!`.
/// The path is resolved relative to the crate's `commands/skill.rs` source file.
///
/// Usage in `crates/i-rs-xxx/src/commands/skill.rs`:
/// ```ignore
/// i_rs_core::skill_command!("i-rs-xxx");
/// ```
#[macro_export]
macro_rules! skill_command {
    ($crate_name:literal) => {
        use clap::Parser;

        #[derive(Parser, Debug)]
        pub enum SkillCommand {
            Summary,
            Content,
            Raw,
        }

        const SKILL_RAW: &str = include_str!(concat!(
            "../../../../skills/",
            $crate_name,
            "/SKILL.md"
        ));

        pub fn handle_skill(which: Option<SkillCommand>) {
            match which {
                Some(SkillCommand::Summary) => {
                    for line in SKILL_RAW.lines() {
                        if let Some(desc) = line
                            .strip_prefix("description:")
                            .or_else(|| line.strip_prefix("description :"))
                        {
                            println!("{}", desc.trim().trim_matches('"'));
                            return;
                        }
                    }
                    // Fallback: print first non-frontmatter line
                    for line in SKILL_RAW.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !trimmed.starts_with("---") && !trimmed.starts_with("name:") && !trimmed.starts_with("description:") {
                            println!("{}", trimmed);
                            return;
                        }
                    }
                }
                Some(SkillCommand::Content) => {
                    let mut in_frontmatter = true;
                    for line in SKILL_RAW.lines() {
                        if in_frontmatter {
                            if line.trim() == "---" {
                                in_frontmatter = false;
                            }
                            continue;
                        }
                        println!("{}", line);
                    }
                }
                Some(SkillCommand::Raw) | None => {
                    println!("{}", SKILL_RAW);
                }
            }
        }
    };
}

/// Handle a CLI `Result` by printing the error and exiting.
///
/// If `json` is true, the error is printed as JSON. Otherwise, it's printed as
/// a colored error message via `print_error`.
///
/// Usage in `main.rs`:
/// ```ignore
/// fn main() {
///     let cli = Cli::parse();
///     let json = cli.json;
///     i_rs_core::exit_on_error!(run(cli.command), json);
/// }
/// ```
#[macro_export]
macro_rules! exit_on_error {
    ($result:expr, $json:expr) => {
        if let Err(e) = $result {
            if $json {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": { "code": "UNKNOWN", "message": e.to_string() }
                    })
                );
            } else {
                $crate::print_error(&format!("{}", e));
            }
            ::std::process::exit(1);
        }
    };
}

/// Generate the `handle_example()` function.
///
/// Shows CLI usage examples from a static string.
///
/// Usage in `crates/i-rs-xxx/src/commands/example.rs`:
/// ```ignore
/// i_rs_core::example_command!("i-rs-todo");
/// ```
/// The second argument is the content to display.
#[macro_export]
macro_rules! example_command {
    ($examples:expr) => {
        use owo_colors::OwoColorize;

        pub fn handle_example() {
            println!();
            println!("{}", $examples);
            println!();
        }
    };
}
