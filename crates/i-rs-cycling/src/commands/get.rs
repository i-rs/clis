use crate::models::CyclingRecord;
use crate::presentation::{format_detail_table, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use uuid::Uuid;

pub fn handle_get(id_or_date: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let record = find_record(&store, &id_or_date)?;

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct RecordItem {
            id: String,
            date: String,
            distance_km: f64,
            duration_minutes: u32,
            avg_speed: f64,
            elevation_gain: Option<f64>,
            route: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let item = RecordItem {
            id: record.id.to_string(),
            date: record.date.format("%Y-%m-%d").to_string(),
            distance_km: record.distance_km,
            duration_minutes: record.duration_minutes,
            avg_speed: record.avg_speed,
            elevation_gain: record.elevation_gain,
            route: record.route.clone(),
            tags: record.tags.clone(),
            remark: record.remark.clone(),
            created_at: record.created_at.format("%Y-%m-%d %H:%M").to_string(),
            updated_at: record.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        };

        println!("{}", output_item(&item, format));
        return Ok(());
    }

    let table = format_detail_table(record);
    println!("\n{}", table);

    Ok(())
}

fn find_record<'a>(store: &'a crate::models::CyclingStore, id_or_date: &str) -> Result<&'a CyclingRecord> {
    if let Ok(uuid) = Uuid::parse_str(id_or_date) {
        if let Some(record) = store.get_entry(&uuid) {
            return Ok(record);
        }
    }

    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];
    for format in &formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(id_or_date, format) {
            for record in store.records.values() {
                if record.date == date {
                    return Ok(record);
                }
            }
        }
    }

    anyhow::bail!("Record '{}' not found", id_or_date)
}
