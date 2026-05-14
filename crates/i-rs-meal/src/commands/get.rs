use crate::presentation::{output_error, output_item, print_error, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::Style as OwoStyle;
use owo_colors::OwoColorize;

pub fn handle_get(id: String, date: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(date_str) = date {
        let parsed_date = parse_date(&date_str)?;
        let entries = storage::get_entries_by_date(&store, parsed_date);

        if entries.is_empty() {
            if matches!(format, OutputFormat::Json) {
                println!("{}", output_error(&format!("No meals on {}", date_str), "NOT_FOUND", format));
            } else {
                print_error(&format!("No meals on {}", date_str));
            }
            anyhow::bail!("No meals on {}", date_str);
        }

        if matches!(format, OutputFormat::Json) {
            let items: Vec<crate::models::ListItem> = entries.iter().map(|e| crate::models::ListItem::from(*e)).collect();
            println!("{}", output_item(&items, format));
            return Ok(());
        }

        print_header(&format!("Meals on {}", parsed_date.format("%Y-%m-%d").green()));
        for entry in entries {
            println!();
            let style = OwoStyle::new().bold();
            println!("{:16} {}", "Type:".style(style), entry.meal_type.cyan());
            println!("{:16} {}", "Food:".style(style), entry.food_items.yellow());
            if let Some(cal) = entry.calories {
                println!("{:16} {}", "Calories:".style(style), cal);
            }
            if !entry.tags.is_empty() {
                println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
            }
        }
        return Ok(());
    }

    let short_id = if id.len() >= 8 { &id[..8] } else { &id };

    let entry = match storage::get_entry(&store, short_id) {
        Some(e) => e,
        None => {
            let msg = format!("Meal '{}' not found", id);
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

    let display_id = entry.id[..8].to_string();
    print_header(&format!("Meal: {}", display_id.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Type:".style(style), entry.meal_type.cyan());
    println!("{:16} {}", "Food:".style(style), entry.food_items.yellow());
    if let Some(cal) = entry.calories {
        println!("{:16} {}", "Calories:".style(style), cal);
    }
    if !entry.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }
    if !entry.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), entry.remark.join("; ").dimmed());
    }
    println!("{:16} {}", "Date:".style(style), entry.date.format("%Y-%m-%d").to_string().dimmed());

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
