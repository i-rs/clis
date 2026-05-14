use crate::presentation::{output_list, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(id: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(record) = store.get_entry(&id) {
        if matches!(format, OutputFormat::Json) {
            #[derive(serde::Serialize, Clone)]
            struct RecordItem {
                id: String,
                date: String,
                distance_km: f64,
                duration_minutes: f64,
                pace: String,
                heart_rate: Option<u32>,
                weather: Option<String>,
                tags: Vec<String>,
                remark: Vec<String>,
                created_at: String,
            }

            let item = RecordItem {
                id: record.id.clone(),
                date: record.date.format("%Y-%m-%d").to_string(),
                distance_km: record.distance_km,
                duration_minutes: record.duration_minutes,
                pace: record.pace.clone(),
                heart_rate: record.heart_rate,
                weather: record.weather.clone(),
                tags: record.tags.clone(),
                remark: record.remark.clone(),
                created_at: record.created_at.to_rfc3339(),
            };

            println!("{}", output_list(&[item], 1, None, format));
        } else {
            println!("\n{}", "Run Record Details:".bold().cyan());
            println!("  {:12} {}", "ID:".dimmed(), record.id);
            println!("  {:12} {}", "Date:".dimmed(), record.date);
            println!("  {:12} {} km", "Distance:".dimmed(), format!("{:.2}", record.distance_km));
            println!("  {:12} {} min", "Duration:".dimmed(), format!("{:.2}", record.duration_minutes));
            println!("  {:12} {}/km", "Pace:".dimmed(), record.pace);
            if let Some(hr) = record.heart_rate {
                println!("  {:12} {} bpm", "Heart Rate:".dimmed(), hr);
            }
            if let Some(ref weather) = record.weather {
                println!("  {:12} {}", "Weather:".dimmed(), weather);
            }
            if !record.tags.is_empty() {
                println!("  {:12} {}", "Tags:".dimmed(), record.tags.join(", "));
            }
            if !record.remark.is_empty() {
                println!("  {:12} {}", "Remark:".dimmed(), record.remark.join(", "));
            }
        }
    } else {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, Some(&format!("ID: {}", id)), format));
        } else {
        }
        anyhow::bail!("No record found with ID: {}", id);
    }

    Ok(())
}
