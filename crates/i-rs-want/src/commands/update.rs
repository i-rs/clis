use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

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

    let entry = match storage::get_entry_mut(&mut store, &name) {
        Some(e) => e,
        None => {
            print_error(&format!("Item '{}' not found", name));
            anyhow::bail!("Item '{}' not found", name);
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
