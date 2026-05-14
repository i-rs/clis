use crate::presentation::{output_error, output_item, print_error, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entry = match storage::get_entry(&store, &name) {
        Some(e) => e,
        None => {
            let msg = format!("Entry '{}' not found", name);
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

    print_header(&format!("Entry: {}", entry.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    let days = entry.days_until_next();

    println!("{:16} {}", "Name:".style(style), entry.name.cyan());
    println!("{:16} {} {}", "Amount:".style(style), entry.amount, entry.currency);
    println!("{:16} {}", "Frequency:".style(style), entry.frequency.cyan());
    println!("{:16} {}", "Start Date:".style(style), entry.start_date.format("%Y-%m-%d").to_string().cyan());

    let next_str = if days < 0 {
        format!("{} ({} days overdue)", "OVERDUE".red(), days.abs())
    } else if days <= 7 {
        format!("{} ({} days)", "DUE SOON".yellow().bold(), days)
    } else {
        format!("{} days", days)
    };
    println!("{:16} {}", "Next In:".style(style), next_str);

    if !entry.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !entry.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), entry.remark.join("; ").dimmed());
    }

    println!("{:16} {}", "Created:".style(style), entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
