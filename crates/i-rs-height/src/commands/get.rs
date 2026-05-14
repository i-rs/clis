use crate::presentation::{output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_get(date: String, format: OutputFormat) -> Result<()> {
    let date = parse_date(&date)?;

    let store = storage::load_store()?;

    let record = match store.get_entry(&date) {
        Some(r) => r,
        None => {
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

