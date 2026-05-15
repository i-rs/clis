use crate::models::ListItem;
use crate::presentation::{print_header, format_minutes, OutputFormat, output_item};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_get(id: String, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let entry = match store.get_entry(&id) {
        Some(e) => e,
        None => anyhow::bail!("Entry '{id}' not found"),
    };

    if format.is_json() {
        let item: ListItem = entry.into();
        println!("{}", output_item(&item, format));
    } else {
        print_header(&format!("Time Entry: {}", entry.name));
        println!("  {} {}", "ID:".cyan(), entry.id.green());
        println!("  {} {}", "Name:".cyan(), entry.name.green());
        println!("  {} {}", "Started:".cyan(), entry.start_time.format("%Y-%m-%d %H:%M").green());
        if let Some(end) = entry.end_time {
            println!("  {} {}", "Ended:".cyan(), end.format("%Y-%m-%d %H:%M").green());
        }
        println!("  {} {}", "Duration:".cyan(), format_minutes(entry.duration_minutes).green());
        if !entry.tags.is_empty() {
            println!("  {} {}", "Tags:".cyan(), entry.tags.join(", ").green());
        }
        if !entry.remark.is_empty() {
            println!("  {} {}", "Remark:".cyan(), entry.remark.join(", ").green());
        }
        println!("  {} {}", "Created:".cyan(), entry.created_at.format("%Y-%m-%d %H:%M:%S").dimmed());
        println!("  {} {}", "Updated:".cyan(), entry.updated_at.format("%Y-%m-%d %H:%M:%S").dimmed());
    }

    Ok(())
}
