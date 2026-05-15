use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if !storage::project_exists(&store, &name) {
        anyhow::bail!("Project '{name}' not found");
    }

    let removed = store.remove_entry(&name)
        .ok_or_else(|| anyhow::anyhow!("Failed to remove project"))?;

    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Project '{}' deleted successfully (had {} milestones, {} tasks)",
        removed.name.green(),
        removed.milestones.len(),
        removed.tasks.len()
    ));

    Ok(())
}
