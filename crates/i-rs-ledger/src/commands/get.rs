use crate::presentation::{OutputFormat, output_error, output_item, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(id: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let short_id = if id.len() >= 8 { &id[..8] } else { &id };

    let entry = if let Some(e) = store.get_entry(short_id) {
        e
    } else {
        let msg = format!("Entry '{id}' not found");
        if format.is_json() {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        }
        anyhow::bail!("{msg}");
    };

    if format.is_json() {
        let output = crate::models::ListItem::from(entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    let display_id = entry.id[..8].to_string();
    print_header(&format!("Entry: {}", display_id.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "ID:".style(style), entry.id.cyan());
    println!(
        "{:16} {}",
        "Date:".style(style),
        entry.date.format("%Y-%m-%d").to_string().cyan()
    );
    println!("{:16} {}", "Type:".style(style), entry.entry_type.yellow());
    println!(
        "{:16} {}",
        "Category:".style(style),
        entry.category.yellow()
    );
    println!(
        "{:16} {} {}",
        "Amount:".style(style),
        entry.amount,
        entry.currency
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
