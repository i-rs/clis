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

    let article = match storage::get_entry_mut(&mut store, &name) {
        Some(a) => a,
        None => {
            anyhow::bail!("Article '{name}' not found");
        }
    };

    if let Some(t) = title {
        article.title = t;
    }

    if let Some(u) = url {
        if let Err(e) = i_rs_core::validate_url(&u) {
            anyhow::bail!("{}", e.message);
        }
        article.url = u;
    }

    if let Some(s) = source {
        article.source = s;
    }

    if let Some(st) = status {
        let new_status = ReadStatus::from(st.as_str());
        article.status = new_status;
        if new_status == ReadStatus::Read && article.read_at.is_none() {
            article.read_at = Some(Utc::now());
        }
    }

    if let Some(t) = tag {
        article.tags = t;
    }

    if let Some(r) = remark {
        article.remark = r;
    }

    if let Some(n) = notes {
        article.notes = n;
    }

    article.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Article '{}' updated successfully", name.green()));

    Ok(())
}
