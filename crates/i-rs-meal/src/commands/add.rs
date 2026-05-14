use crate::models::MealEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;

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

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Added {}: {} on {}", meal_type.green(), food_items.cyan(), parsed_date.format("%Y-%m-%d").to_string().yellow()));

    Ok(())
}

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }
    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
