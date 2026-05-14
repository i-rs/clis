use crate::models::CalEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    food_name: String,
    calories: i32,
    date: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = date.map_or_else(|| chrono::Utc::now().date_naive(), |d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").unwrap_or_else(|_| chrono::Utc::now().date_naive()));

    let entry = CalEntry::new(food_name.clone(), calories, tag, remark, parsed_date);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Recorded {} ({} kcal)", food_name.green(), calories.to_string().cyan()));

    Ok(())
}