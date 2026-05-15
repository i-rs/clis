use crate::presentation::{output_error, output_item, OutputFormat};
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(date: String, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let record = match crate::service::get_weight(&store, &date) {
        Ok(r) => r,
        Err(e) => {
            if format.is_json() {
                println!("{}", output_error(&e.to_string(), "NOT_FOUND", format));
            }
            return Err(e);
        }
    };

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct GetOutput {
            date: String,
            weight: f64,
            tags: Vec<String>,
            remark: Vec<String>,
        }

        let output = GetOutput {
            date: record.date.format("%Y-%m-%d").to_string(),
            weight: record.weight,
            tags: record.tags.clone(),
            remark: record.remark.clone(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    let style = OwoStyle::new().bold();
    println!(
        "{}",
        format!("Weight Record: {}", record.date.format("%Y-%m-%d"))
            .bold()
            .cyan()
    );
    println!();
    println!("{:16} {:.1} kg", "Weight:".style(style), record.weight);
    if !record.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            record.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", ")
        );
    }
    if !record.remark.is_empty() {
        println!(
            "{:16} {}",
            "Remark:".style(style),
            record.remark.join("; ").dimmed()
        );
    }

    Ok(())
}
