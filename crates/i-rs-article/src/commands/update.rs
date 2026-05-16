use crate::models::ReadStatus;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

#[allow(clippy::too_many_arguments)]
pub fn handle_update(
    name: String,
    title: Option<String>,
    url: Option<String>,
    source: Option<String>,
    status: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    notes: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let article = match store.get_entry_mut(&name) {
        Some(a) => a,
        None => {
            anyhow::bail!("Article '{name}' not found");
        }
    };

    i_rs_core::update_field!(article.title, title);

    if let Some(u) = url {
        if let Err(e) = i_rs_core::validate_url(&u) {
            anyhow::bail!("{}", e.message);
        }
        article.url = u;
    }

    i_rs_core::update_field!(article.source, source);

    if let Some(st) = status {
        let new_status = ReadStatus::from(st.as_str());
        article.status = new_status;
        if new_status == ReadStatus::Read && article.read_at.is_none() {
            article.read_at = Some(Utc::now());
        }
    }

    i_rs_core::update_field!(article.tags, tag);

    i_rs_core::update_field!(article.remark, remark);

    i_rs_core::update_field!(article.notes, notes);

    article.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Article '{}' updated successfully", name.green()));

    Ok(())
}
