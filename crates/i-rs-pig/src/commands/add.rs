use crate::models::PigEntry;
use crate::presentation::{print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    food_name: String,
    description: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = PigEntry::new(food_name.clone(), description, tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Recorded '{}' at {}", food_name.green(), chrono::Utc::now().format("%H:%M").to_string().cyan()));

    Ok(())
}
