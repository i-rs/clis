use crate::models::CyclingRecord;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    distance: f64,
    duration: u32,
    elevation: Option<f64>,
    route: Option<String>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let date = parse_date(&date)?;

    if distance <= 0.0 {
        anyhow::bail!("Distance must be greater than 0");
    }

    if duration == 0 {
        anyhow::bail!("Duration must be greater than 0");
    }

    let record = CyclingRecord::new(date, distance, duration, elevation, route, tags, remark);

    let mut store = storage::load_store()?;
    store.add_entry(record.clone());
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Cycling record added: {} km in {} min ({} km/h)",
        distance.green(),
        duration.to_string().green(),
        format!("{:.1}", record.avg_speed).green()
    ));

    Ok(())
}
