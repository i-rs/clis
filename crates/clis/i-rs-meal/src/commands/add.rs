use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use crate::service;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_add(
    meal_type: String,
    food_items: String,
    date: Option<String>,
    calories: Option<i32>,
    tag: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let date_str = date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });
    let parsed_date = parse_date(&date_str)?;

    let entry = service::add_meal(
        &mut store,
        meal_type.clone(),
        food_items.clone(),
        calories,
        tag,
        remark,
        parsed_date,
    )?;
    storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success(&format!(
        "✓ Added {}: {} on {}",
        meal_type.green(),
        food_items.cyan(),
        parsed_date.format("%Y-%m-%d").to_string().yellow()
    ));

    Ok(())
}
