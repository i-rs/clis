use crate::presentation::{print_error, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;

pub fn handle_get(date: String, format: OutputFormat) -> Result<()> {
    let date = parse_date(&date)?;

    let store = storage::load_store()?;

    let record = match store.get_record(&date) {
        Some(r) => r,
        None => {
            print_error(&format!("No record found for {}", date));
            anyhow::bail!("No record found for {}", date);
        }
    };

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct Item {
            date: String,
            height_cm: f64,
            weight_kg: Option<f64>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
        }

        let item = Item {
            date: record.date.format("%Y-%m-%d").to_string(),
            height_cm: record.height_cm,
            weight_kg: record.weight_kg,
            tags: record.tags.clone(),
            remark: record.remark.clone(),
            created_at: record.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        };

        println!("{}", output_item(&item, format));
        return Ok(());
    }

    println!();
    println!("{}", "Date:".dimmed(),);
    println!("  {}", record.date.format("%Y-%m-%d").cyan());
    println!();
    println!("{}", "Height:".dimmed());
    println!("  {} cm", record.height_cm.cyan());
    if let Some(weight) = record.weight_kg {
        println!();
        println!("{}", "Weight:".dimmed());
        println!("  {} kg", weight.cyan());
    }
    if !record.tags.is_empty() {
        println!();
        println!("{}", "Tags:".dimmed());
        for tag in &record.tags {
            println!("  {}", format!("#{}", tag).green());
        }
    }
    if !record.remark.is_empty() {
        println!();
        println!("{}", "Remark:".dimmed());
        for r in &record.remark {
            println!("  {}", r);
        }
    }
    println!();
    println!("{}", "Created:".dimmed());
    println!("  {}", record.created_at.format("%Y-%m-%d %H:%M:%S UTC").dimmed());

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
