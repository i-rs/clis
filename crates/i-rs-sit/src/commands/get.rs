use crate::presentation::{OutputFormat, output_item, print_header};
use crate::service;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(id: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entry = service::get_sit(&store, &id)?;

    if format.is_json() {
        let output = crate::models::ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    let display_id = entry.id[..8].to_string();
    print_header(&format!("Sit: {}", display_id.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!(
        "{:16} {}",
        "Duration:".style(style),
        format!("{} minutes", entry.duration_minutes).cyan()
    );
    println!(
        "{:16} {}",
        "Started:".style(style),
        entry
            .started_at
            .format("%Y-%m-%d %H:%M")
            .to_string()
            .yellow()
    );
    println!(
        "{:16} {}",
        "Ended:".style(style),
        entry.ended_at.format("%Y-%m-%d %H:%M").to_string().yellow()
    );
    if !entry.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            entry
                .tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !entry.remark.is_empty() {
        println!(
            "{:16} {}",
            "Remark:".style(style),
            entry.remark.join("; ").dimmed()
        );
    }
    println!(
        "{:16} {}",
        "Created:".style(style),
        entry
            .created_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    Ok(())
}
