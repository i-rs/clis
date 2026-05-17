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
        use clap::Subcommand;

        // --------------------------------------------------------------------
        // Internal structs for structured command data
        // --------------------------------------------------------------------

        /// Structured info about a single CLI command/subcommand.
        struct CmdInfo {
            name: ::std::string::String,
            desc: ::std::string::String,
            usage: ::std::string::String,
            args: ::std::vec::Vec<ArgInfo>,
            options: ::std::vec::Vec<ArgInfo>,
        }

        /// Structured info about a single argument or option.
        struct ArgInfo {
            name: ::std::string::String,
            desc: ::std::string::String,
            positional: bool,
            required: bool,
        }

        // --------------------------------------------------------------------
        // SkillCommand enum
        // --------------------------------------------------------------------

        #[derive(Subcommand, Debug, Clone)]
        #[command(
            name = "skill",
            about = "AI skill system: run 'skill teach' for a complete AI guide, or 'skill info' for tool metadata"
        )]
        pub enum SkillCommand {
            /// Show structured metadata (name, description, commands)
            Info,
            /// Search within the skill content
            Search {
                /// Search query (case-insensitive)
                query: String,
            },
            /// Generate a comprehensive teaching prompt for AI agents (text or --json)
            Teach {
                /// Output in machine-readable JSON format
                #[arg(long)]
                json: bool,
            },
            /// Install the skill file to a directory
            Install {
                /// Target directory path (prints to stdout if omitted)
                path: Option<String>,
                /// Target AI agent (for install path shortcuts)
                #[arg(short, long)]
                agent: Option<String>,
            },
            /// Show tool description from SKILL.md
            Summary,
            /// Show content after YAML frontmatter
            Content,
            /// Show raw SKILL.md (default action when no subcommand given)
            Raw,
        }

        const SKILL_RAW: &str =
            include_str!(concat!("../../../../skills/", $crate_name, "/SKILL.md"));

        pub fn handle_skill(cmd: &SkillCommand) -> ::anyhow::Result<()> {
            match cmd {
                SkillCommand::Info => {
                    let name = extract_frontmatter_field(SKILL_RAW, "name")
                        .unwrap_or($crate_name);
                    let desc = extract_frontmatter_field(SKILL_RAW, "description")
                        .unwrap_or("");
                    let commands = extract_commands(SKILL_RAW);

                    println!("Tool: {}", name);
                    println!("Description: {}", desc);
                    println!();
                    println!("Commands ({} total):", commands.len());
                    for cmd_info in &commands {
                        println!("  {} - {}", cmd_info.name, cmd_info.desc);
                    }
                }
                SkillCommand::Search { query } => {
                    let q = query.to_lowercase();
                    let mut found = false;
                    for (i, line) in SKILL_RAW.lines().enumerate() {
                        if line.to_lowercase().contains(&q) {
                            println!("{}: {}", i + 1, line);
                            found = true;
                        }
                    }
                    if !found {
                        println!("No matches found for '{}'.", query);
                    }
                }
                SkillCommand::Teach { json } => {
                    let name = extract_frontmatter_field(SKILL_RAW, "name")
                        .unwrap_or($crate_name);
                    let desc = extract_frontmatter_field(SKILL_RAW, "description")
                        .unwrap_or("");
                    let tool_name = $crate_name.trim_start_matches("i-rs-");
                    let commands = extract_commands(SKILL_RAW);
                    let examples = extract_examples(SKILL_RAW);

                    if *json {
                        print_teach_json(name, desc, $crate_name, tool_name, &commands, &examples);
                    } else {
                        print_teach_text(name, desc, $crate_name, tool_name, &commands, &examples);
                    }
                }
                SkillCommand::Install { path, agent } => {
                    let content = match agent.as_deref() {
                        Some(a) => format!(
                            "<!-- Installed from {} for agent: {} -->\n{}",
                            $crate_name, a, SKILL_RAW
                        ),
                        None => SKILL_RAW.to_string(),
                    };
                    if let Some(p) = path {
                        let dir = ::std::path::Path::new(&p);
                        ::std::fs::create_dir_all(dir)?;
                        let file_path = dir.join("SKILL.md");
                        ::std::fs::write(&file_path, &content)?;
                        println!(
                            "Skill installed to: {}",
                            file_path.display()
                        );
                    } else {
                        println!("{}", content);
                    }
                }
                SkillCommand::Summary => {
                    for line in SKILL_RAW.lines() {
                        if let Some(desc) = line
                            .strip_prefix("description:")
                            .or_else(|| line.strip_prefix("description :"))
                        {
                            println!("{}", desc.trim().trim_matches('"'));
                            return Ok(());
                        }
                    }
                    // Fallback: print first non-frontmatter line
                    for line in SKILL_RAW.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty()
                            && !trimmed.starts_with("---")
                            && !trimmed.starts_with("name:")
                            && !trimmed.starts_with("description:")
                        {
                            println!("{}", trimmed);
                            return Ok(());
                        }
                    }
                }
                SkillCommand::Content => {
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
                SkillCommand::Raw => {
                    println!("{}", SKILL_RAW);
                }
            }
            Ok(())
        }

        // ====================================================================
        // Helper: Extract YAML frontmatter fields
        // ====================================================================

        /// Extract a field value from YAML frontmatter.
        fn extract_frontmatter_field<'a>(content: &'a str, field: &str) -> Option<&'a str> {
            let mut in_frontmatter = false;
            for line in content.lines() {
                if line.trim() == "---" {
                    if in_frontmatter {
                        break; // End of frontmatter
                    }
                    in_frontmatter = true; // Start of frontmatter
                    continue;
                }
                if !in_frontmatter {
                    continue;
                }
                if let Some(val) = line
                    .strip_prefix(&format!("{}:", field))
                    .or_else(|| line.strip_prefix(&format!("{} :", field)))
                {
                    return Some(val.trim().trim_matches('"'));
                }
            }
            None
        }

        // ====================================================================
        // Helper: Extract structured command info from SKILL.md
        // ====================================================================

        /// Parse all `### <name>` command sections under `## Commands`.
        ///
        /// Returns structured info including description, usage, positional
        /// arguments, and options for each command.
        fn extract_commands(content: &str) -> ::std::vec::Vec<CmdInfo> {
            let lines: ::std::vec::Vec<&str> = content.lines().collect();
            let mut commands: ::std::vec::Vec<CmdInfo> = ::std::vec::Vec::new();
            let mut i = 0;
            let len = lines.len();

            // Phase: skip frontmatter
            let mut in_frontmatter = true;
            let mut in_commands_section = false;

            while i < len {
                let trimmed = lines[i].trim();

                if in_frontmatter {
                    if trimmed == "---" && i > 0 {
                        in_frontmatter = false;
                    }
                    i += 1;
                    continue;
                }

                // Find ## Commands
                if !in_commands_section {
                    if trimmed == "## Commands" {
                        in_commands_section = true;
                    }
                    i += 1;
                    continue;
                }

                // Another ## heading = left the commands section
                if trimmed.starts_with("## ") && trimmed != "## Commands" {
                    break;
                }

                // ### heading = new command
                if let Some(name) = trimmed.strip_prefix("### ") {
                    if let Some(cmd) = parse_one_command(name, &lines, &mut i, len) {
                        commands.push(cmd);
                    }
                    continue;
                }

                i += 1;
            }

            commands
        }

        /// Parse a single command block starting from `### <name>` at index `i`.
        /// Advances `i` past the block.
        fn parse_one_command(
            name: &str,
            lines: &[&str],
            i: &mut usize,
            len: usize,
        ) -> Option<CmdInfo> {
            *i += 1; // move past ### heading

            let mut cmd = CmdInfo {
                name: name.to_string(),
                desc: ::std::string::String::new(),
                usage: ::std::string::String::new(),
                args: ::std::vec::Vec::new(),
                options: ::std::vec::Vec::new(),
            };

            // States
            let mut in_code_block = false;
            let mut in_args_section = false;
            let mut in_options_section = false;
            let mut desc_lines: ::std::vec::Vec<::std::string::String> =
                ::std::vec::Vec::new();
            let mut usage_lines: ::std::vec::Vec<::std::string::String> =
                ::std::vec::Vec::new();
            let mut desc_done = false;

            while *i < len {
                let raw = lines[*i];
                let trimmed = raw.trim();

                // Stop at next ### or any ## (other sections)
                if trimmed.starts_with("### ") {
                    break;
                }
                if trimmed.starts_with("## ") && trimmed != "## " {
                    break;
                }

                // Code block
                if trimmed.starts_with("```") {
                    if !in_code_block {
                        in_code_block = true;
                        desc_done = true;
                        in_args_section = false;
                        in_options_section = false;
                    } else {
                        in_code_block = false;
                    }
                    *i += 1;
                    continue;
                }

                if in_code_block {
                    usage_lines.push(raw.to_string());
                    *i += 1;
                    continue;
                }

                // Section headers: Arguments / Options
                if trimmed.starts_with("Arguments:") || trimmed.starts_with("Arguments :") {
                    in_args_section = true;
                    in_options_section = false;
                    desc_done = true;
                    *i += 1;
                    continue;
                }
                if trimmed.starts_with("Options:") || trimmed.starts_with("Options :") {
                    in_options_section = true;
                    in_args_section = false;
                    desc_done = true;
                    *i += 1;
                    continue;
                }

                // Parse argument lines
                if in_args_section {
                    if trimmed.starts_with("- `") {
                        if let Some(arg) = parse_arg_line(trimmed) {
                            cmd.args.push(arg);
                        }
                    }
                    *i += 1;
                    continue;
                }

                // Parse option lines
                if in_options_section {
                    if trimmed.starts_with("- `") {
                        if let Some(opt) = parse_opt_line(trimmed) {
                            cmd.options.push(opt);
                        }
                    }
                    *i += 1;
                    continue;
                }

                // Description text (before code block or sections)
                if !desc_done && !trimmed.is_empty() {
                    desc_lines.push(trimmed.to_string());
                }

                *i += 1;
            }

            // Finalize description
            if !desc_lines.is_empty() {
                // Take first paragraph only
                cmd.desc = desc_lines[0].clone();
            }
            // Finalize usage: join any multi-line code block content
            if !usage_lines.is_empty() {
                cmd.usage = usage_lines.join("\n");
            }

            Some(cmd)
        }

        /// Parse a positional argument line like `- \`MEAL_TYPE\` - Description`.
        fn parse_arg_line(line: &str) -> Option<ArgInfo> {
            let inner = line.strip_prefix("- `")?;
            let (name_rest, desc) = inner.split_once("` - ")?;
            Some(ArgInfo {
                name: name_rest.to_string(),
                desc: desc.to_string(),
                positional: true,
                required: !name_rest.starts_with('[') && !name_rest.contains("OPTIONS"),
            })
        }

        /// Parse an option line like `- \`--food <ARG>\` - Description`
        /// or `- \`-f, --flag\` - Description`.
        fn parse_opt_line(line: &str) -> Option<ArgInfo> {
            let inner = line.strip_prefix("- `")?;
            let (opt_rest, desc) = inner.split_once("` - ")?;

            // Extract value placeholder `<VALUE>` from opt_rest
            // e.g. "--food <FOOD_ITEMS>" -> flag="--food", val="FOOD_ITEMS"
            let (flag_part, val_part) = if let Some(angle_start) = opt_rest.find(" <") {
                let flag = &opt_rest[..angle_start];
                let val = &opt_rest[angle_start + 2..opt_rest.len() - 1]; // strip < >
                (flag, Some(val.to_string()))
            } else {
                (opt_rest, None)
            };

            let full_name = if let Some(ref v) = val_part {
                format!("{} <{}>", flag_part, v)
            } else {
                flag_part.to_string()
            };

            Some(ArgInfo {
                name: full_name,
                desc: desc.to_string(),
                positional: false,
                required: false,
            })
        }

        // ====================================================================
        // Helper: Extract examples from SKILL.md
        // ====================================================================

        /// Parse example lines from the `## Examples` code block.
        fn extract_examples(content: &str) -> ::std::vec::Vec<::std::string::String> {
            let mut examples: ::std::vec::Vec<::std::string::String> =
                ::std::vec::Vec::new();
            let lines: ::std::vec::Vec<&str> = content.lines().collect();
            let mut i = 0;
            let len = lines.len();
            let mut in_frontmatter = true;
            let mut in_examples_section = false;
            let mut in_code_block = false;

            while i < len {
                let trimmed = lines[i].trim();

                if in_frontmatter {
                    if trimmed == "---" && i > 0 {
                        in_frontmatter = false;
                    }
                    i += 1;
                    continue;
                }

                if !in_examples_section {
                    if trimmed == "## Examples" {
                        in_examples_section = true;
                    }
                    i += 1;
                    continue;
                }

                // Another ## heading = leave examples
                if trimmed.starts_with("## ") && trimmed != "## Examples" {
                    break;
                }

                if trimmed.starts_with("```") {
                    if !in_code_block {
                        in_code_block = true;
                    } else {
                        break; // end of examples code block
                    }
                    i += 1;
                    continue;
                }

                if in_code_block {
                    examples.push(lines[i].to_string());
                }

                i += 1;
            }

            examples
        }

        // ====================================================================
        // Output functions for `teach`
        // ====================================================================

        /// Print a clean, LLM-friendly teaching document (text format).
        fn print_teach_text(
            name: &str,
            desc: &str,
            binary: &str,
            tool_name: &str,
            commands: &[CmdInfo],
            examples: &[::std::string::String],
        ) {
            // --- Header ---
            println!("# {} ({}) \u{2014} AI Tool Guide", tool_name, binary);
            println!();
            println!("## Description");
            println!("{}", desc);
            println!();
            println!("## Invocation");
            println!("  Claw: i_rs(tool=\"{}\", command=\"...\", args=[...])", tool_name);
            println!("  CLI:  i-rs {} <command> [args]", tool_name);
            println!("  Binary: {}", binary);
            println!();
            println!("## Commands ({} total)", commands.len());
            println!();

            for cmd in commands {
                println!("### {}", cmd.name);

                if !cmd.desc.is_empty() {
                    println!("{}", cmd.desc);
                    println!();
                }
                if !cmd.usage.is_empty() {
                    let normalized = cmd.usage.replace(
                        &format!("{} ", binary),
                        &format!("i-rs {} ", tool_name)
                    );
                    println!("  Usage: `{}`", normalized);
                    println!();
                }

                // Separate required / optional positional args
                let required_args: ::std::vec::Vec<&ArgInfo> = cmd
                    .args
                    .iter()
                    .filter(|a| a.required)
                    .collect();
                let optional_args: ::std::vec::Vec<&ArgInfo> = cmd
                    .args
                    .iter()
                    .filter(|a| !a.required)
                    .collect();

                if !required_args.is_empty() {
                    println!("  Arguments:");
                    for arg in &required_args {
                        println!("    {} (required) - {}", arg.name, arg.desc);
                    }
                    println!();
                }
                if !optional_args.is_empty() {
                    println!("  Optional Arguments:");
                    for arg in &optional_args {
                        println!("    {} - {}", arg.name, arg.desc);
                    }
                    println!();
                }

                // Separate required / optional options
                let required_opts: ::std::vec::Vec<&ArgInfo> = cmd
                    .options
                    .iter()
                    .filter(|o| o.required && !o.name.starts_with('['))
                    .collect();
                let optional_opts: ::std::vec::Vec<&ArgInfo> = cmd
                    .options
                    .iter()
                    .filter(|o| !o.required || o.name.starts_with('['))
                    .collect();

                if !required_opts.is_empty() {
                    println!("  Required Options:");
                    for opt in &required_opts {
                        println!("    {} - {}", opt.name, opt.desc);
                    }
                    println!();
                }
                if !optional_opts.is_empty() {
                    println!("  Options:");
                    for opt in &optional_opts {
                        println!("    {} - {}", opt.name, opt.desc);
                    }
                    println!();
                }
            }

            // --- Examples ---
            if !examples.is_empty() {
                println!("## Examples");
                println!();
                for ex in examples {
                    let normalized = ex.replace(
                        &format!("{} ", binary),
                        &format!("i-rs {} ", tool_name)
                    );
                    println!("  {}", normalized);
                }
                println!();
            }
        }

        /// Print a machine-readable JSON teaching document.
        fn print_teach_json(
            name: &str,
            desc: &str,
            binary: &str,
            tool_name: &str,
            commands: &[CmdInfo],
            examples: &[::std::string::String],
        ) {
            let cmds: ::std::vec::Vec<serde_json::Value> = commands
                .iter()
                .map(|cmd| {
                    let normalized_usage = cmd.usage.replace(
                        &format!("{} ", binary),
                        &format!("i-rs {} ", tool_name)
                    );

                    let args: ::std::vec::Vec<serde_json::Value> = cmd
                        .args
                        .iter()
                        .map(|a| {
                            serde_json::json!({
                                "name": a.name,
                                "description": a.desc,
                                "positional": a.positional,
                                "required": a.required,
                            })
                        })
                        .collect();

                    let opts: ::std::vec::Vec<serde_json::Value> = cmd
                        .options
                        .iter()
                        .map(|o| {
                            serde_json::json!({
                                "name": o.name,
                                "description": o.desc,
                                "positional": o.positional,
                                "required": o.required,
                            })
                        })
                        .collect();

                    serde_json::json!({
                        "name": cmd.name,
                        "description": cmd.desc,
                        "usage": normalized_usage,
                        "args": args,
                        "options": opts,
                    })
                })
                .collect();

            // Normalize example binary prefixes for JSON too
            let normalized_examples: ::std::vec::Vec<::std::string::String> = examples
                .iter()
                .map(|ex| {
                    ex.replace(
                        &format!("{} ", binary),
                        &format!("i-rs {} ", tool_name)
                    )
                })
                .collect();

            let output = serde_json::json!({
                "name": name,
                "description": desc,
                "tool": tool_name,
                "binary": binary,
                "commands": cmds,
                "examples": normalized_examples,
            });

            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }

        // ====================================================================
        // Legacy parser (deprecated)
        // ====================================================================

        /// Parse a skill subcommand argument. Returns `None` for empty input
        /// (show raw), `Some(cmd)` for valid subcommands, and exits with an
        /// error for invalid input.
        ///
        /// Note: This is deprecated. Use `#[clap(subcommand)] Skill(SkillCommand)`
        /// in the Commands enum instead.
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
                let tmp = ::std::env::temp_dir().join(::std::format!(
                    "{}-test-{}",
                    $prefix,
                    ::std::process::id()
                ));
                let _ = ::std::fs::create_dir_all(&tmp);
                // Safety: test-only, single-threaded access to CONFIG_DIR
                unsafe {
                    ::std::env::set_var("CONFIG_DIR", tmp.to_str().unwrap());
                }
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
                println!(
                    "{}",
                    $crate::presentation::output_list::<::serde_json::Value>(
                        &[],
                        0,
                        None::<&str>,
                        $format
                    )
                );
            } else {
                $crate::presentation::print_warning("No records found.");
            }
            return Ok(());
        }
    };
    ($entries:expr, $format:expr, $filter:expr) => {
        if $entries.is_empty() {
            if $format.is_json() {
                println!(
                    "{}",
                    $crate::presentation::output_list::<::serde_json::Value>(
                        &[],
                        0,
                        $filter,
                        $format
                    )
                );
            } else {
                $crate::presentation::print_warning("No records found.");
            }
            return Ok(());
        }
    };
    ($entries:expr, $format:expr, $filter:expr, $msg:expr) => {
        if $entries.is_empty() {
            if $format.is_json() {
                println!(
                    "{}",
                    $crate::presentation::output_list::<::serde_json::Value>(
                        &[],
                        0,
                        $filter,
                        $format
                    )
                );
            } else {
                $crate::presentation::print_warning($msg);
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

/// Generate a standard `presentation/mod.rs` for a crate.
///
/// Generates:
/// - Re-exports for `print_header`, `print_success`, `OutputFormat`
/// - Re-exports for `output_list`, `output_item`, `output_error`
/// - `format_table()` — wraps `render_table` with typed rows
/// - `print_entry_count()` — prints "Total: N {label}"
///
/// Extra re-exports can be specified as additional identifiers
/// (e.g., `print_warning`, `print_error`).
///
/// Usage in `crates/i-rs-xxx/src/presentation/mod.rs`:
/// ```ignore
/// i_rs_core::presentation!(XxxRow, "records");
/// i_rs_core::presentation!(XxxRow, "entries", print_warning);
/// ```
#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! presentation {
    ($row_type:ident, $label:expr $(,)?) => {
        use crate::models::$row_type;
        use owo_colors::OwoColorize;
        pub use i_rs_core::presentation::{print_header, print_success, OutputFormat};
        pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
        pub fn format_table(rows: &[$row_type]) -> String {
            i_rs_core::render_table(rows)
        }
        pub fn print_entry_count(count: usize) {
            println!("\n{} {} {}", "Total:".dimmed(), count.to_string().cyan(), $label);
        }
    };
    ($row_type:ident, $label:expr, $($extra:ident),+ $(,)?) => {
        use crate::models::$row_type;
        use owo_colors::OwoColorize;
        pub use i_rs_core::presentation::{print_header, print_success, OutputFormat $(, $extra)*};
        pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
        pub fn format_table(rows: &[$row_type]) -> String {
            i_rs_core::render_table(rows)
        }
        pub fn print_entry_count(count: usize) {
            println!("\n{} {} {}", "Total:".dimmed(), count.to_string().cyan(), $label);
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
