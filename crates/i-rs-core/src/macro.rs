/// Generate storage functions for a crate.
///
/// Generates:
/// - `load_store()` / `save_store()` — basic persistence
/// - `export_data()` — serialize store to JSON string
/// - `import_data(input)` — deserialize and save from JSON string
/// - `clear_data()` — reset store to default (empty)
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

        /// Export all data as pretty JSON string.
        pub fn export_data() -> ::anyhow::Result<String> {
            let store = load_store()?;
            Ok(::serde_json::to_string_pretty(&store)?)
        }

        /// Import data from a JSON string, replacing all existing data.
        pub fn import_data(input: &str) -> ::anyhow::Result<()> {
            let store: $store_type = ::serde_json::from_str(input)?;
            save_store(&store)
        }

        /// Clear all data (reset store to default/empty).
        pub fn clear_data() -> ::anyhow::Result<()> {
            let store = <$store_type>::default();
            save_store(&store)
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

        /// Parse a skill subcommand argument. Returns `None` for empty input
        /// (show raw), `Some(cmd)` for valid subcommands, and exits with an
        /// error for invalid input.
        pub fn parse_skill_arg(sub: Option<&str>) -> Option<SkillCommand> {
            match sub {
                Some("summary") => Some(SkillCommand::Summary),
                Some("content") => Some(SkillCommand::Content),
                Some("raw") => Some(SkillCommand::Raw),
                None => None,
                Some(_) => {
                    $crate::print_error("Invalid subcommand. Use: summary, content, or raw");
                    ::std::process::exit(1);
                }
            }
        }
    };
}

/// Generate a `setup()` function for test setup.
///
/// Creates a temp directory and sets `CONFIG_DIR` for test isolation.
///
/// Usage in `crates/i-rs-xxx/src/tests.rs`:
/// ```ignore
/// #[cfg(test)]
/// mod tests {
///     use crate::{Cli, Commands, commands, run};
///     use clap::Parser;
///
///     i_rs_core::test_setup!("i-rs-xxx");
///
///     #[test]
///     fn test_example() {
///         setup();
///         let cmd = Commands::Example {};
///         assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
///     }
/// }
/// ```
#[macro_export]
macro_rules! test_setup {
    ($prefix:literal) => {
        fn setup() {
            use ::std::sync::OnceLock;
            static INIT: OnceLock<()> = OnceLock::new();
            INIT.get_or_init(|| {
                let tmp = ::std::env::temp_dir()
                    .join(::std::format!("{}-test-{}", $prefix, ::std::process::id()));
                let _ = ::std::fs::create_dir_all(&tmp);
                // Safety: test-only, single-threaded access to CONFIG_DIR
                unsafe { ::std::env::set_var("CONFIG_DIR", tmp.to_str().unwrap()); }
            });
        }
    };
}

/// Handle empty list results with JSON/table output.
///
/// Replaces the common pattern in `list.rs`:
/// ```ignore
/// if entries.is_empty() {
///     if format.is_json() {
///         println!("{}", output_list::<serde_json::Value>(&[], 0, ...));
///     } else {
///         print_warning("No records found.");
///     }
///     return Ok(());
/// }
/// ```
#[macro_export]
macro_rules! handle_empty {
    ($entries:expr, $format:expr) => {
        if $entries.is_empty() {
            if $format.is_json() {
                ::println!("{}", $crate::presentation::output_list::<::serde_json::Value>(&[], 0, None::<&str>, $format));
            } else {
                $crate::presentation::print_warning("No records found.");
            }
            return Ok(());
        }
    };
    ($entries:expr, $format:expr, $filter:expr) => {
        if $entries.is_empty() {
            if $format.is_json() {
                ::println!("{}", $crate::presentation::output_list::<::serde_json::Value>(&[], 0, $filter, $format));
            } else {
                $crate::presentation::print_warning("No records found.");
            }
            return Ok(());
        }
    };
}

/// Update a field if the new value is `Some`.
///
/// Replaces the common pattern:
/// ```ignore
/// if let Some(v) = new_value {
///     entity.field = v;
/// }
/// ```
#[macro_export]
macro_rules! update_field {
    ($target:expr, $value:expr) => {
        if let Some(v) = $value {
            $target = v;
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

/// Generate the `DataCommand` enum and `handle` function for data export/import/clear.
///
/// Usage in `crates/i-rs-xxx/src/commands/data.rs`:
/// ```ignore
/// i_rs_core::data_command!();
/// ```
#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! data_command {
    () => {
        #[derive(::clap::Subcommand, Debug, Clone)]
        pub enum DataCommand {
            #[command(about = "Export all data as JSON")]
            Export,
            #[command(about = "Import data from JSON file or stdin")]
            Import {
                #[arg(value_name = "FILE")]
                file: Option<String>,
            },
            #[command(about = "Clear all data")]
            Clear,
        }

        pub fn handle(command: &DataCommand) -> ::anyhow::Result<()> {
            use ::std::io::Read;
            match command {
                DataCommand::Export => {
                    let exported = crate::storage::export_data()?;
                    println!("{exported}");
                    Ok(())
                }
                DataCommand::Import { file } => {
                    let input = if let Some(path) = &file {
                        ::std::fs::read_to_string(path)?
                    } else {
                        let mut buf = String::new();
                        ::std::io::stdin().read_to_string(&mut buf)?;
                        buf
                    };
                    crate::storage::import_data(&input)?;
                    println!("Data imported successfully");
                    Ok(())
                }
                DataCommand::Clear => {
                    crate::storage::clear_data()?;
                    println!("Data cleared successfully");
                    Ok(())
                }
            }
        }
    };
}
