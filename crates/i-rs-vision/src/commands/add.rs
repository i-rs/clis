use crate::models::VisionRecord;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    left_sphere: Option<f64>,
    right_sphere: Option<f64>,
    left_cylinder: Option<f64>,
    right_cylinder: Option<f64>,
    left_axis: Option<i32>,
    right_axis: Option<i32>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let date = parse_date(&date)?;

    if left_sphere.is_none() && right_sphere.is_none() {
        anyhow::bail!("At least one sphere value is required");
    }

    let mut store = storage::load_store()?;

    if store.records.contains_key(&date) {
        anyhow::bail!("Record for {} already exists", date);
    }

    let record = VisionRecord {
        date,
        left_sphere,
        right_sphere,
        left_cylinder,
        right_cylinder,
        left_axis,
        right_axis,
        tags: tag,
        remark,
        created_at: Utc::now(),
    };

    store.add_entry(record);
    storage::save_store(&store)?;

    print_success(&format!("✓ Vision record added for {}", date.green()));

    Ok(())
}

