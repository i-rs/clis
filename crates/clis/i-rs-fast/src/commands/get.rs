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
        let msg = format!("Record '{id}' not found");
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
    print_header(&format!("Fast: {}", display_id.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!(
        "{:16} {}",
        "Target:".style(style),
        format!("{} hours", entry.target_hours).cyan()
    );
    if let Some(actual) = entry.actual_hours {
        println!(
            "{:16} {}",
            "Actual:".style(style),
            format!("{actual} hours").yellow()
        );
    }
    println!(
        "{:16} {}",
        "Start:".style(style),
        entry
            .start_time
            .format("%Y-%m-%d %H:%M")
            .to_string()
            .green()
    );
    if let Some(end) = entry.end_time {
        println!(
            "{:16} {}",
            "End:".style(style),
            end.format("%Y-%m-%d %H:%M").to_string().red()
        );
    }
    let status = if entry.end_time.is_some() {
        "COMPLETED"
    } else {
        "ACTIVE"
    };
    let status_color = if entry.end_time.is_some() {
        status.green().to_string()
    } else {
        status.yellow().to_string()
    };
    println!("{:16} {}", "Status:".style(style), status_color);
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
