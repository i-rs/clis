use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_delete(id_or_date: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let id = find_and_remove(&mut store, &id_or_date)?;

    storage::save_store(&store)?;

    print_success(&format!("✓ Record '{}' deleted", id.green()));

    Ok(())
}

fn find_and_remove(store: &mut crate::models::CyclingStore, id_or_date: &str) -> Result<String, anyhow::Error> {
    if let Ok(uuid) = Uuid::parse_str(id_or_date) {
        if store.remove_entry(&uuid).is_some() {
            return Ok(id_or_date.to_string());
        }
    }

    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(id_or_date, format) {
            let mut found_id: Option<String> = None;
            for (id, record) in store.records.iter() {
                if record.date == date {
                    found_id = Some(id.to_string());
                    break;
                }
            }
            if let Some(fid) = found_id {
                store.remove_entry(&Uuid::parse_str(&fid).expect("stored ids are always valid UUIDs"));
                return Ok(fid[..8].to_string());
            }
        }
    }

    anyhow::bail!("Record '{}' not found", id_or_date)
}
