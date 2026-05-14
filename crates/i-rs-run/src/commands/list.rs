use crate::models::RunRecord;
use crate::presentation::{format_run_table, output_list, print_run_count, print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let records: Vec<RunRecord> = store.records.values().cloned().collect();

    if records.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, None, format));
        } else {
            print_warning("No run records found.");
        }
        return Ok(());
    }

    let records_ref: Vec<&RunRecord> = records.iter().collect();

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            id: String,
            date: String,
            distance_km: f64,
            duration_minutes: f64,
            pace: String,
            heart_rate: Option<u32>,
            weather: Option<String>,
            tags: Vec<String>,
        }

        let items: Vec<ListItem> = records
            .iter()
            .map(|r| ListItem {
                id: r.id.clone(),
                date: r.date.format("%Y-%m-%d").to_string(),
                distance_km: r.distance_km,
                duration_minutes: r.duration_minutes,
                pace: r.pace.clone(),
                heart_rate: r.heart_rate,
                weather: r.weather.clone(),
                tags: r.tags.clone(),
            })
            .collect();

        println!("{}", output_list(&items, items.len(), None, format));
        return Ok(());
    }

    let table = format_run_table(&records_ref);
    println!("\n{}", table);

    print_run_count(records_ref.len());

    Ok(())
}
