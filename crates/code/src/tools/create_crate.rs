use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

pub struct CreateCrateTool;

#[async_trait]
impl Tool for CreateCrateTool {
    fn name(&self) -> &str { "create_crate" }
    fn description(&self) -> &str { "Create a new i-rs CLI crate following project standards" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "create_crate",
                "description": "Generate a new i-rs CLI crate with all standard files",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Crate name (e.g. i-rs-mood)"},
                        "description": {"type": "string", "description": "Tool description"},
                        "output_dir": {"type": "string", "description": "Output directory (default: crates/clis/)"},
                        "special_commands": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Additional commands beyond standard CRUD"
                        }
                    },
                    "required": ["name"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("name required"))?;
        let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let output_dir = args.get("output_dir").and_then(|v| v.as_str()).unwrap_or("crates/clis");
        let special_cmds: Vec<&str> = args.get("special_commands")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
            .unwrap_or_default();

        let crate_dir = format!("{}/{}", output_dir, name);
        let src_dir = format!("{}/src", crate_dir);
        let cmds_dir = format!("{}/commands", src_dir);
        let models_dir = format!("{}/models", src_dir);
        let storage_dir = format!("{}/storage", src_dir);
        let pres_dir = format!("{}/presentation", src_dir);

        std::fs::create_dir_all(&cmds_dir)?;
        std::fs::create_dir_all(&models_dir)?;
        std::fs::create_dir_all(&storage_dir)?;
        std::fs::create_dir_all(&pres_dir)?;

        // Cargo.toml
        let cargo = format!(r#"[package]
name = "{}"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
i-rs-core = {{ path = "../../i-rs-core" }}
clap.workspace = true
anyhow.workspace = true
serde.workspace = true
tabled.workspace = true
owo-colors.workspace = true
chrono.workspace = true
uuid.workspace = true
"#, name);
        std::fs::write(format!("{}/Cargo.toml", crate_dir), &cargo)?;

        // main.rs
        let main_rs = format!(r#"use clap::{{Parser, Subcommand}};
use i_rs_core::presentation::OutputFormat;
use i_rs_core::exit_on_error;

#[derive(Parser, Debug)]
#[command(name = "{}")]
#[command(about = "{}", long_about = None)]
struct Cli {{
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, global = true)]
    json: bool,
}}

#[derive(Subcommand, Debug)]
enum Commands {{
    Add {{ name: String, #[arg(short, long)] tags: Vec<String> }},
    Delete {{ name: String }},
    Get {{ name: String }},
    List {{ #[arg(short, long)] tag: Option<String> }},
    Update {{ name: String, #[arg(short, long)] tags: Vec<String> }},
    Example {{}},
    Skill {{ sub: Option<String> }},
}}

fn main() {{
    let cli = Cli::parse();
    let format = if cli.json {{ OutputFormat::Json }} else {{ OutputFormat::Table }};
    exit_on_error!(run(cli.command, format), cli.json);
}}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {{
    match command {{
        Commands::Add {{ name, tags }} => {{ todo!() }}
        Commands::Delete {{ name }} => {{ todo!() }}
        Commands::Get {{ name }} => {{ todo!() }}
        Commands::List {{ tag }} => {{ todo!() }}
        Commands::Update {{ name, tags }} => {{ todo!() }}
        Commands::Example {{}} => {{ todo!() }}
        Commands::Skill {{ sub }} => {{ todo!() }}
    }}
}}
"#, name, description);
        std::fs::write(format!("{}/main.rs", src_dir), &main_rs)?;

        // models/mod.rs
        let models = r#"use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub type Store = BTreeMap<String, Entity>;
"#;
        std::fs::write(format!("{}/mod.rs", models_dir), models)?;

        // storage/mod.rs
        let storage = format!("i_rs_core::create_store!({}Store, \"{}\");", name, name);
        std::fs::write(format!("{}/mod.rs", storage_dir), &storage)?;

        // presentation/mod.rs
        let pres = r#"use crate::models::Entity;
use i_rs_core::presentation::{render_table, OutputFormat};

pub fn render_entries(entries: &[Entity], format: OutputFormat) -> anyhow::Result<()> {
    let rows: Vec<_> = entries.iter().map(|e| Row {
        name: &e.name,
        tags: &e.tags.join(", "),
        created: &e.created_at.format("%Y-%m-%d").to_string(),
    }).collect();
    render_table(&rows, format)?;
    Ok(())
}

#[derive(tabled::Tabled)]
struct Row<'a> {
    name: &'a str,
    tags: &'a str,
    created: &'a str,
}
"#;
        std::fs::write(format!("{}/mod.rs", pres_dir), pres)?;

        // commands/mod.rs
        let mut cmds_mod = String::from(r#"pub mod add;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;
pub mod example;
pub mod skill;
"#);
        for sc in &special_cmds {
            cmds_mod.push_str(&format!("pub mod {};\n", sc));
        }
        std::fs::write(format!("{}/mod.rs", cmds_dir), &cmds_mod)?;

        // Generate basic command files
        for cmd_name in &["add", "delete", "get", "list", "update", "example", "skill"] {
            let content = format!("// TODO: implement {} command\n", cmd_name);
            std::fs::write(format!("{}/{}.rs", cmds_dir, cmd_name), &content)?;
        }

        // Generate special command files
        for sc in &special_cmds {
            let content = format!("// TODO: implement {} command\n", sc);
            std::fs::write(format!("{}/{}.rs", cmds_dir, sc), &content)?;
        }

        Ok(format!("Created crate {} at {}/\nFiles:\n{:?}", name, crate_dir,
            std::fs::read_dir(&crate_dir).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect::<Vec<_>>()).unwrap_or_default()))
    }
}
