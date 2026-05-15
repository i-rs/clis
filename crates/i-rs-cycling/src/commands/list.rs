use crate::models::CyclingRecord;
use crate::presentation::{format_table, print_record_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    tag: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let records: Vec<CyclingRecord> = if let Some(ref t) = tag {
        store
            .records
            .values()
            .filter(|r| r.tags.iter().any(|tag| tag == t))
            .cloned()
            .collect()
    } else {
        store.records.values().cloned().collect()
    };

    if records.is_empty() {
        if format.is_json() {
            let filter = tag.as_deref();
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter, format));
        } else if tag.is_some() {
                if let Some(ref t) = tag {
                    print_warning(&format!("No cycling records found with tag '{}'", t));
                }
        } else {
            print_warning("No cycling records found.");
        }
        return Ok(());
    }

    let records_ref: Vec<&CyclingRecord> = records.iter().collect();

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            id: String,
            date: String,
            distance_km: f64,
            duration_minutes: u32,
            avg_speed: f64,
            elevation_gain: Option<f64>,
            route: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
        }

        let items: Vec<ListItem> = records.iter().map(|r| ListItem {
            id: r.id.to_string(),
            date: r.date.format("%Y-%m-%d").to_string(),
            distance_km: r.distance_km,
            duration_minutes: r.duration_minutes,
            avg_speed: r.avg_speed,
            elevation_gain: r.elevation_gain,
            route: r.route.clone(),
            tags: r.tags.clone(),
            remark: r.remark.clone(),
        }).collect();

        let filter = tag.as_deref();
        println!("{}", output_list(&items, items.len(), filter, format));
        return Ok(());
    }

    let table = format_table(&records_ref);
    println!("\n{table}");

    print_record_count(records_ref.len());

    Ok(())
}
