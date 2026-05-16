use crate::presentation::{OutputFormat, output_error, output_item, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entry = if let Some(e) = store.get_entry(&name) {
        e
    } else {
        let msg = format!("Entry '{name}' not found");
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

    print_header(&format!("Entry: {}", entry.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    let days = entry.days_until_next();

    println!("{:16} {}", "Name:".style(style), entry.name.cyan());
    println!(
        "{:16} {} {}",
        "Amount:".style(style),
        entry.amount,
        entry.currency
    );
    println!(
        "{:16} {}",
        "Frequency:".style(style),
        entry.frequency.cyan()
    );
    println!(
        "{:16} {}",
        "Start Date:".style(style),
        entry.start_date.format("%Y-%m-%d").to_string().cyan()
    );

    let next_str = if days < 0 {
        format!("{} ({} days overdue)", "OVERDUE".red(), days.abs())
    } else if days <= 7 {
        format!("{} ({} days)", "DUE SOON".yellow().bold(), days)
    } else {
        format!("{days} days")
    };
    println!("{:16} {}", "Next In:".style(style), next_str);

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
