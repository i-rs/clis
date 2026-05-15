use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let snippet = if let Some(s) = storage::get_entry(&store, &name) { s } else {
        let msg = format!("Snippet '{name}' not found");
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        } 
        anyhow::bail!("{msg}");
    };

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct GetOutput {
            id: String,
            name: String,
            language: String,
            code: Vec<String>,
            description: Vec<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            id: snippet.id.clone(),
            name: snippet.name.clone(),
            language: snippet.language.clone(),
            code: snippet.code.clone(),
            description: snippet.description.clone(),
            tags: snippet.tags.clone(),
            remark: snippet.remark.clone(),
            created_at: snippet.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: snippet.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Snippet: {}", snippet.name.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Language:".style(style), snippet.language.cyan());

    if !snippet.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), snippet.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !snippet.description.is_empty() {
        println!("\n{}:", "Description".bold());
        for line in &snippet.description {
            println!("  {line}");
        }
    }

    if !snippet.code.is_empty() {
        println!("\n{}:", "Code".bold());
        for line in &snippet.code {
            println!("  {line}");
        }
    }

    if !snippet.remark.is_empty() {
        println!("\n{}:", "Remark".bold());
        for line in &snippet.remark {
            println!("  {}", line.dimmed());
        }
    }

    println!("\n{:16} {}", "ID:".style(style), snippet.id.dimmed());
    println!("{:16} {}", "Created:".style(style), snippet.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), snippet.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
