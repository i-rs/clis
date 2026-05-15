use crate::models::WantEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    url: Option<String>,
    price: Option<f64>,
    currency: Option<String>,
    priority: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&name) {
        anyhow::bail!("Item '{name}' already exists");
    }

    let entry = WantEntry::new(name.clone(), url, price, currency, priority, tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Item '{}' added to wishlist", name.green()));

    Ok(())
}
