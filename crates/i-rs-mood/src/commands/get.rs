use crate::presentation::{output_error, output_item, OutputFormat};
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(date: String, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let record = match crate::service::get_mood(&store, &date) {
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
            mood: String,
            tags: Vec<String>,
            content: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            date: record.date.format("%Y-%m-%d").to_string(),
            mood: record.mood.label().to_string(),
            tags: record.tags.clone(),
            content: record.content.clone(),
            remark: record.remark.clone(),
            created_at: record.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    let style = OwoStyle::new().bold();
    println!(
        "{}",
        format!("Mood: {}", record.date.format("%Y-%m-%d"))
            .bold()
            .cyan()
    );
    println!();
    println!(
        "{:16} {} {}",
        "Mood:".style(style),
        record.mood.emoji(),
        record.mood.label().cyan()
    );

    if !record.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            record.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", ")
        );
    }

    if !record.content.is_empty() {
        println!("\n{}:", "Content".bold());
        for line in &record.content {
            println!("  {line}");
        }
    }

    if !record.remark.is_empty() {
        println!(
            "{:16} {}",
            "Remark:".style(style),
            record.remark.join("; ").dimmed()
        );
    }

    println!(
        "\n{:16} {}",
        "Created:".style(style),
        record.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
    );

    Ok(())
}
