use crate::presentation::{output_error, output_item, print_error, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(date: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let entry = match storage::get_entry(&store, &parsed_date) {
        Some(e) => e,
        None => {
            let msg = format!("No record for {}", date);
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

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }
    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
