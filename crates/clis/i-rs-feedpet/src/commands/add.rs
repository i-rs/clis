use crate::models::FeedpetEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    pet_name: String,
    food_type: String,
    amount: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = FeedpetEntry::new(
        pet_name.clone(),
        food_type.clone(),
        amount.clone(),
        tag,
        remark,
    );

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Fed {} with {} ({})",
        pet_name.green(),
        food_type.cyan(),
        amount.yellow()
    ));

    Ok(())
}
