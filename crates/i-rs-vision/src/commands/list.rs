use crate::models::VisionRecord;
use crate::presentation::{format_table, print_record_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn handle_list(days: Option<usize>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let records: Vec<VisionRecord> = if let Some(d) = days {
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
            let filter = days.map(|d| format!("last {} days", d));
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format));
        } else {
            print_warning("No vision records found.");
        }
        return Ok(());
    }

    let records_ref: Vec<&VisionRecord> = records.iter().collect();

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            date: String,
            left_sphere: Option<f64>,
            right_sphere: Option<f64>,
            left_cylinder: Option<f64>,
            right_cylinder: Option<f64>,
            left_axis: Option<i32>,
            right_axis: Option<i32>,
            tags: Vec<String>,
            remark: Vec<String>,
        }

        let items: Vec<ListItem> = records.iter().map(|r| ListItem {
            date: r.date.format("%Y-%m-%d").to_string(),
            left_sphere: r.left_sphere,
            right_sphere: r.right_sphere,
            left_cylinder: r.left_cylinder,
            right_cylinder: r.right_cylinder,
            left_axis: r.left_axis,
            right_axis: r.right_axis,
            tags: r.tags.clone(),
            remark: r.remark.clone(),
        }).collect();

        let filter = days.map(|d| format!("last {} days", d));
        println!("{}", output_list(&items, items.len(), filter.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&records_ref);
    println!("\n{}", table);

    print_record_count(records_ref.len());

    Ok(())
}
