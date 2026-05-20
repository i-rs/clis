use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item};
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(id: String, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let record = crate::service::get_mood(&store, &id)?;

    if format.is_json() {
        let output = ListItem::from(&record);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    let style = OwoStyle::new().bold();
    println!(
        "{}",
        format!("Mood: {} ({})", record.date.format("%Y-%m-%d"), id.dimmed())
            .bold()
            .cyan()
    );
    println!();
    println!(
        "{:16} {} {}",
        "Mood:".style(style),
        record.mood.emoji(),
        record.mood.label().cyan()
    );

    if !record.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            record
                .tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !record.content.is_empty() {
        println!("\n{}:", "Content".bold());
        for line in &record.content {
            println!("  {line}");
        }
    }

    if !record.remark.is_empty() {
        println!(
            "{:16} {}",
            "Remark:".style(style),
            record.remark.join("; ").dimmed()
        );
    }

    println!(
        "\n{:16} {}",
        "Created:".style(style),
        record
            .created_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        record
            .updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    Ok(())
}
