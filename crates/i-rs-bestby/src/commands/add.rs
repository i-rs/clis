use crate::models::Entity;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

pub fn handle_add(
    name: String,
    purchase_date: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&name) {
        anyhow::bail!("Item '{}' already exists", name);
    }

    let purchase = parse_datetime(&purchase_date)?;

    let now = Utc::now();
    let entity = Entity {
        name: name.clone(),
        purchase_date: purchase,
        cycle_days: None,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entity);
    storage::save_store(&store)?;

    print_success(&format!("✓ Item '{}' added successfully", name.green()));
    println!("  {}", "Note: Use 'update' command to set replacement cycle".dimmed());

    Ok(())
}


