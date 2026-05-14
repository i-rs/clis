use crate::models::HeightRecord;
use crate::presentation::{format_table, print_height_chart, print_record_count, print_stats, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn handle_list(days: Option<usize>, chart: bool, stats: bool, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let records: Vec<HeightRecord> = if let Some(d) = days {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(d as i64);
        store
            .records
            .values()
            .filter(|r| r.date >= cutoff)
            .cloned()
            .collect()
    } else {
        store.records.values().cloned().collect()
    };

    if records.is_empty() {
        if matches!(format, OutputFormat::Json) {
            let filter = days.map(|d| format!("last {d} days"));
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format));
        } else {
            print_warning("No height records found.");
        }
        return Ok(());
    }

    let records_ref: Vec<&HeightRecord> = records.iter().collect();

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            date: String,
            height_cm: f64,
            weight_kg: Option<f64>,
            tags: Vec<String>,
            remark: Vec<String>,
        }

        let items: Vec<ListItem> = records.iter().map(|r| ListItem {
            date: r.date.format("%Y-%m-%d").to_string(),
            height_cm: r.height_cm,
            weight_kg: r.weight_kg,
            tags: r.tags.clone(),
            remark: r.remark.clone(),
        }).collect();

        let filter = days.map(|d| format!("last {d} days"));
        println!("{}", output_list(&items, items.len(), filter.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&records_ref);
    println!("\n{table}");

    print_record_count(records_ref.len());

    if stats {
        print_stats(&records_ref, &store);
    }

    if chart {
        print_height_chart(&records_ref, days);
    }

    Ok(())
}
