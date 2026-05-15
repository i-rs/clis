use crate::models::Snippet;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_add(
    name: String,
    language: String,
    code: Vec<String>,
    description: Vec<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.snippets.contains_key(&name) {
        anyhow::bail!("Snippet '{name}' already exists");
    }

    let now = Utc::now();
    let snippet = Snippet {
        id: Uuid::new_v4().to_string(),
        name: name.clone(),
        language,
        code,
        description,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, snippet);
    storage::save_store(&store)?;

    print_success(&format!("✓ Snippet '{}' added successfully", name.green()));

    Ok(())
}
