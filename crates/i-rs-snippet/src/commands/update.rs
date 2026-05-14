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

    let snippet = match storage::get_snippet_mut(&mut store, &name) {
        Some(s) => s,
        None => {
            anyhow::bail!("Snippet '{}' not found", name);
        }
    };

    if let Some(language) = language {
        snippet.language = language;
    }
    if let Some(code) = code {
        snippet.code = code;
    }
    if let Some(description) = description {
        snippet.description = description;
    }
    if let Some(tag) = tag {
        snippet.tags = tag;
    }
    if let Some(remark) = remark {
        snippet.remark = remark;
    }

    snippet.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Snippet '{}' updated successfully", name.green()));

    Ok(())
}
