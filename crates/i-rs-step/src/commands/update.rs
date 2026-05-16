use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_update(
    date: String,
    steps: Option<i32>,
    distance: Option<f64>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let entry = match store.get_entry_mut(&parsed_date) {
        Some(e) => e,
        None => {
            anyhow::bail!("No record for {date}");
        }
    };

    i_rs_core::update_field!(entry.steps, steps);
    if let Some(d) = distance {
        entry.distance = Some(d);
    }
    i_rs_core::update_field!(entry.tags, tag);
    i_rs_core::update_field!(entry.remark, remark);

    storage::save_store(&store)?;

    print_success(&format!("✓ Record for {} updated", date.green()));

    Ok(())
}
