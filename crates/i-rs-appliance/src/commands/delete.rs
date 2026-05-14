use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.get_by_name(&name).is_none() {
        print_error(&format!("Appliance '{}' not found", name));
        anyhow::bail!("Appliance '{}' not found", name);
    }

    store.remove_appliance_by_name(&name);
    storage::save_store(&store)?;

    print_success(&format!("✓ Appliance '{}' deleted", name.green()));

    Ok(())
}
