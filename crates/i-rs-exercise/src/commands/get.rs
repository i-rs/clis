use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entry = match store.get_entry(&name) {
        Some(e) => e,
        None => {
            let msg = format!("Exercise '{}' not found", name);
            if matches!(format, OutputFormat::Json) {
                println!("{}", output_error(&msg, "NOT_FOUND", format));
            } else {
            }
            anyhow::bail!("{}", msg);
        }
    };

    if matches!(format, OutputFormat::Json) {
        let output = crate::models::ListItem::from(entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Exercise: {}", entry.name.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Name:".style(style), entry.name.cyan());
    println!("{:16} {}", "Type:".style(style), entry.exercise_type.cyan());
    println!("{:16} {} min", "Duration:".style(style), entry.duration_minutes);
    if let Some(cal) = entry.calories {
        println!("{:16} {} kcal", "Calories:".style(style), cal.to_string().yellow());
    }
    if !entry.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }
    if !entry.notes.is_empty() {
        println!("{:16} {}", "Notes:".style(style), entry.notes.iter().map(|n| n.dimmed().to_string()).collect::<Vec<_>>().join("; "));
    }
    if !entry.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), entry.remark.iter().map(|r| r.dimmed().to_string()).collect::<Vec<_>>().join("; "));
    }
    println!("{:16} {}", "Created:".style(style), entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
