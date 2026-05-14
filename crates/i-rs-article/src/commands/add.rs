use crate::models::{Article, ReadStatus};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn handle_add(
    name: String,
    title: String,
    url: String,
    source: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.articles.contains_key(&name) {
        anyhow::bail!("Article '{}' already exists", name);
    }

    if let Err(e) = i_rs_core::validate_url(&url) {
        anyhow::bail!("{}", e.message);
    }

    if let Err(e) = i_rs_core::validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    let now = Utc::now();
    let article = Article {
        name,
        title,
        url,
        source,
        status: ReadStatus::Unread,
        notes: Vec::new(),
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
        read_at: None,
    };

    storage::add_article(&mut store, article);
    storage::save_store(&store)?;

    print_success(&format!("✓ Article added successfully"));

    Ok(())
}
