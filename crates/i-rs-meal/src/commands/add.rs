use crate::models::MealEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use i_rs_core::parse_date;

pub fn handle_add(
    meal_type: String,
    food_items: String,
    date: String,
    calories: Option<i32>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let entry = MealEntry::new(meal_type.clone(), food_items.clone(), calories, tag, remark, parsed_date);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Added {}: {} on {}", meal_type.green(), food_items.cyan(), parsed_date.format("%Y-%m-%d").to_string().yellow()));

    Ok(())
}


