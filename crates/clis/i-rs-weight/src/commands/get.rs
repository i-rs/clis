use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_error, output_item};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(id: String, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let record = match crate::service::get_weight(&store, &id) {
        Ok(r) => r,
        Err(e) => {
            if format.is_json() {
                println!("{}", output_error(&e.to_string(), "NOT_FOUND", format));
            }
            return Err(e);
        }
    };

    if format.is_json() {
        println!("{}", output_item(&ListItem::from(&record), format));
        return Ok(());
    }

    println!(
        "{}  {}",
        "Weight Record:".bold().cyan(),
        record.id[..8].to_string().dimmed()
    );
    println!("  {:16} {:.1} kg", "Weight:".bold(), record.weight);
    println!("  {:16} {}", "Date:".bold(), record.date.format("%Y-%m-%d"));
    if !record.tags.is_empty() {
        println!(
            "  {:16} {}",
            "Tags:".bold(),
            record.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", ")
        );
    }
    if !record.remark.is_empty() {
        println!("  {:16} {}", "Remark:".bold(), record.remark.join("; ").dimmed());
    }

    Ok(())
}
