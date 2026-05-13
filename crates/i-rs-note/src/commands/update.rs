use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    title: Option<String>,
    tag: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let note = match storage::get_note_mut(&mut store, &name) {
        Some(n) => n,
        None => {
            print_error(&format!("Note '{}' not found", name));
            anyhow::bail!("Note '{}' not found", name);
        }
    };

    if let Some(title) = title {
        note.title = Some(title);
    }
    if let Some(tag) = tag {
        note.tags = tag;
    }
    if let Some(content) = content {
        note.content = content;
    }

    note.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Note '{}' updated successfully", name.green()));

    Ok(())
}
