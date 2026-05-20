use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_header};
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(key: String, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let entry = crate::service::get_kv(&store, &key)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Key: {}", entry.key.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Key:".style(style), entry.key.cyan());
    println!("{:16} {}", "Value:".style(style), entry.value.yellow());

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
    println!(
        "{:16} {}",
        "Updated:".style(style),
        entry
            .updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    Ok(())
}
