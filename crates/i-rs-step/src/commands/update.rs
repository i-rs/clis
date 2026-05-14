use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use i_rs_core::parse_date;

pub fn handle_update(
    date: String,
    steps: Option<i32>,
    distance: Option<f64>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let entry = match storage::get_entry_mut(&mut store, &parsed_date) {
        Some(e) => e,
        None => {
            anyhow::bail!("No record for {}", date);
        }
    };

    if let Some(s) = steps {
        entry.steps = s;
    }
    if let Some(d) = distance {
        entry.distance = Some(d);
    }
    if let Some(t) = tag {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Record for {} updated", date.green()));

    Ok(())
}


