use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entry = if let Some(e) = store.get_entry(&name) { e } else {
        let msg = format!("Item '{name}' not found");
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

    let status = if entry.is_done { "DONE" } else { "PENDING" };

    print_header(&format!("Wish: {} [{}]", entry.name.green(), status.cyan()));
    println!();

    let style = OwoStyle::new().bold();

    if let Some(p) = entry.price {
        let c = entry.currency.as_deref().unwrap_or("");
        println!("{:16} {:.2} {}", "Price:".style(style), p, c);
    }
    println!("{:16} {}", "Priority:".style(style), entry.priority.yellow());
    if let Some(url) = &entry.url {
        println!("{:16} {}", "URL:".style(style), url.cyan());
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
