use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if !storage::project_exists(&store, &name) {
        print_error(&format!("Project '{}' not found", name));
        anyhow::bail!("Project '{}' not found", name);
    }

    let removed = storage::remove_project(&mut store, &name)
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
