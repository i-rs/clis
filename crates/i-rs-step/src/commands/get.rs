use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;
use i_rs_core::parse_date;

pub fn handle_get(date: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let entry = if let Some(e) = storage::get_entry(&store, &parsed_date) { e } else {
        let msg = format!("No record for {date}");
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

    print_header(&format!("Steps on {}", entry.date.format("%Y-%m-%d").green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Steps:".style(style), entry.steps.to_string().cyan());
    if let Some(dist) = entry.distance {
        println!("{:16} {:.1} km", "Distance:".style(style), dist);
    }
    if !entry.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }
    if !entry.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), entry.remark.join("; ").dimmed());
    }

    Ok(())
}


