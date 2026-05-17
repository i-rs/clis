use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

#[allow(clippy::too_many_arguments)]
pub fn handle_update(
    id: String,
    date: Option<String>,
    distance: Option<f64>,
    duration: Option<f64>,
    heart_rate: Option<Option<u32>>,
    weather: Option<Option<String>>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let record = match store.get_entry_mut(&id) {
        Some(r) => r,
        None => anyhow::bail!("Record '{id}' not found"),
    };

    if let Some(d) = date {
        record.date = parse_date(&d)?;
    }
    i_rs_core::update_field!(record.distance_km, distance);
    if let Some(d) = duration {
        record.duration_minutes = d;
        record.pace = crate::models::format_pace(record.distance_km, record.duration_minutes);
    }
    i_rs_core::update_field!(record.heart_rate, heart_rate);
    i_rs_core::update_field!(record.weather, weather);
    i_rs_core::update_field!(record.tags, tags);
    i_rs_core::update_field!(record.remark, remark);

    record.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Run record '{}' updated successfully",
        id.green()
    ));
    Ok(())
}
