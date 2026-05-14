use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entry = if let Some(e) = storage::get_entry(&store, &name) { e } else {
        let msg = format!("Subscription '{name}' not found");
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        } 
        anyhow::bail!("{msg}");
    };

    if matches!(format, OutputFormat::Json) {
        let output = crate::models::ListItem::from(entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Subscription: {}", entry.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    let days = entry.days_until_next();

    println!("{:16} {} {}", "Amount:".style(style), entry.amount, entry.currency);
    println!("{:16} {}", "Cycle:".style(style), entry.billing_cycle.cyan());

    let next_str = if days < 0 {
        format!("{} ({} days overdue)", "OVERDUE".red(), days.abs())
    } else if days <= 7 {
        format!("{} ({} days)", "DUE SOON".yellow().bold(), days)
    } else {
        format!("{days} days")
    };
    println!("{:16} {}", "Next in:".style(style), next_str);
    println!("{:16} {}", "Next date:".style(style), entry.next_billing_date.format("%Y-%m-%d").to_string().cyan());

    if let Some(url) = &entry.url {
        println!("{:16} {}", "URL:".style(style), url.yellow());
    }
    if !entry.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }
    if !entry.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), entry.remark.join("; ").dimmed());
    }

    Ok(())
}
