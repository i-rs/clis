use crate::models::StepEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use i_rs_core::parse_date;

pub fn handle_add(
    steps: i32,
    date: String,
    distance: Option<f64>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let entry = StepEntry::new(steps, distance, parsed_date, tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    let dist_str = distance.map(|d| format!(" ({:.1} km)", d)).unwrap_or_default();
    print_success(&format!("✓ Recorded {} steps on {}{}", steps.to_string().green(), parsed_date.format("%Y-%m-%d").to_string().cyan(), dist_str));

    Ok(())
}


