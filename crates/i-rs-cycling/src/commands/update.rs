use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_update(
    id_or_date: String,
    distance: Option<f64>,
    duration: Option<u32>,
    elevation: Option<Option<f64>>,
    route: Option<String>,
    add_tag: Option<String>,
    remove_tag: Option<String>,
    add_remark: Option<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let record_id = find_record_id(&store, &id_or_date)?;

    let needs_recalc = distance.is_some() || duration.is_some();

    if let Some(d) = distance {
        if d <= 0.0 {
            print_error("Distance must be greater than 0");
            anyhow::bail!("Distance must be greater than 0");
        }
    }

    if let Some(d) = duration {
        if d == 0 {
            print_error("Duration must be greater than 0");
            anyhow::bail!("Duration must be greater than 0");
        }
    }

    let (distance_km, duration_minutes, avg_speed) = {
        let record = match store.get_record_mut(&record_id) {
            Some(r) => r,
            None => {
                print_error(&format!("Record '{}' not found", id_or_date));
                anyhow::bail!("Record '{}' not found", id_or_date);
            }
        };

        if let Some(d) = distance {
            record.distance_km = d;
        }

        if let Some(d) = duration {
            record.duration_minutes = d;
        }

        if needs_recalc {
            record.recalc_avg_speed();
        }

        if let Some(e) = elevation {
            record.elevation_gain = e;
        }

        if let Some(r) = route {
            if r.is_empty() {
                record.route = None;
            } else {
                record.route = Some(r);
            }
        }

        if let Some(tag) = add_tag {
            if !record.tags.contains(&tag) {
                record.tags.push(tag);
            }
        }

        if let Some(tag) = remove_tag {
            record.tags.retain(|t| t != &tag);
        }

        if let Some(remark) = add_remark {
            record.remark.push(remark);
        }

        record.updated_at = Utc::now();

        (record.distance_km, record.duration_minutes, record.avg_speed)
    };

    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Record updated: {} km in {} min ({} km/h)",
        distance_km.green(),
        duration_minutes.to_string().green(),
        format!("{:.1}", avg_speed).green()
    ));

    Ok(())
}

fn find_record_id(store: &crate::models::CyclingStore, id_or_date: &str) -> Result<Uuid, anyhow::Error> {
    if let Ok(uuid) = Uuid::parse_str(id_or_date) {
        if store.get_record(&uuid).is_some() {
            return Ok(uuid);
        }
    }

    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];
    for format in &formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(id_or_date, format) {
            for (id, record) in store.records.iter() {
                if record.date == date {
                    return Ok(*id);
                }
            }
        }
    }

    print_error(&format!("Record '{}' not found", id_or_date));
    anyhow::bail!("Record '{}' not found", id_or_date)
}
