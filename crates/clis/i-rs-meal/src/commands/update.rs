use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use crate::service;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(
    id: String,
    meal_type: Option<String>,
    food_items: Option<String>,
    calories: Option<Option<i32>>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    date: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = match date {
        Some(ref d) => Some(i_rs_core::parse_date(d)?),
        None => None,
    };

    let entry = service::update_meal(
        &mut store,
        &id,
        meal_type,
        food_items,
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

    print_success(&format!("✓ Meal '{}' updated", id.green()));

    Ok(())
}
