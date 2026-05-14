use crate::models::ReadStatus;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_read(name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let article = match storage::get_article_mut(&mut store, &name) {
        Some(a) => a,
        None => {
            anyhow::bail!("Article '{}' not found", name);
        }
    };

    let old_status = article.status;
    article.status = ReadStatus::Read;
    article.read_at = Some(Utc::now());
    article.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Article '{}' marked as read (was: {})",
        name.green(),
        old_status.to_string().dimmed()
    ));

    Ok(())
}
