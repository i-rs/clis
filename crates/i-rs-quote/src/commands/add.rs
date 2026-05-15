use crate::models::Quote;
use crate::presentation::{print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_add(
    content: String,
    author: Option<String>,
    source: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let quote = Quote {
        id: id.clone(),
        content,
        author,
        source,
        tags: tag,
        remark,
        created_at: now,
    };

    store.add_entry(quote);
    storage::save_store(&store)?;

    print_success(&format!("✓ Quote '{}' added successfully", id.green().bold()));

    Ok(())
}
