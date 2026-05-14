use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.investments.contains_key(&name) {
        anyhow::bail!("Investment '{}' not found", name);
    }

    storage::remove_investment(&mut store, &name);
    storage::save_store(&store)?;

    println!("{}", format!("✓ Investment '{}' deleted successfully", name.green()));

    Ok(())
}
