use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

#[allow(clippy::too_many_arguments)]
pub fn handle_update(
    name: String,
    price: Option<f64>,
    currency: Option<String>,
    priority: Option<String>,
    url: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    done: Option<bool>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match store.get_entry_mut(&name) {
        Some(e) => e,
        None => {
            anyhow::bail!("Item '{name}' not found");
        }
    };

    if let Some(p) = price {
        entry.price = Some(p);
    }
    if let Some(c) = currency {
        entry.currency = Some(c);
    }
    if let Some(p) = priority {
        entry.priority = p;
    }
    if let Some(u) = url {
        entry.url = Some(u);
    }
    if let Some(t) = tag {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }
    if let Some(d) = done {
        entry.is_done = d;
    }

    entry.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Item '{}' updated", name.green()));

    Ok(())
}
