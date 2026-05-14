use crate::presentation::{output_error, output_item, print_error, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(id: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let short_id = if id.len() >= 8 { &id[..8] } else { &id };

    let entry = match storage::get_entry(&store, short_id) {
        Some(e) => e,
        None => {
            let msg = format!("Spark '{}' not found", id);
            if matches!(format, OutputFormat::Json) {
                println!("{}", output_error(&msg, "NOT_FOUND", format));
            } else {
                print_error(&msg);
            }
            anyhow::bail!("{}", msg);
        }
    };

    if matches!(format, OutputFormat::Json) {
        let output = crate::models::ListItem::from(entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    let display_id = entry.id[..8].to_string();
    print_header(&format!("Spark: {}", display_id.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Content:".style(style), entry.content.cyan());
    if let Some(source) = &entry.source {
        println!("{:16} {}", "Source:".style(style), source.yellow());
    }
    if !entry.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }
    if !entry.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), entry.remark.join("; ").dimmed());
    }
    println!("{:16} {}", "Created:".style(style), entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
