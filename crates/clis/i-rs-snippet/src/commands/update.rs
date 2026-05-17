use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    language: Option<String>,
    code: Option<Vec<String>>,
    description: Option<Vec<String>>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let snippet = match store.get_entry_mut(&name) {
        Some(s) => s,
        None => {
            anyhow::bail!("Snippet '{name}' not found");
        }
    };

    i_rs_core::update_field!(snippet.language, language);
    i_rs_core::update_field!(snippet.code, code);
    i_rs_core::update_field!(snippet.description, description);
    i_rs_core::update_field!(snippet.tags, tag);
    i_rs_core::update_field!(snippet.remark, remark);

    snippet.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Snippet '{}' updated successfully",
        name.green()
    ));

    Ok(())
}
