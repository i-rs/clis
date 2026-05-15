use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

#[allow(clippy::too_many_arguments)]
pub fn handle_update(
    name: String,
    amount: Option<f64>,
    billing_cycle: Option<String>,
    next_date: Option<String>,
    url: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match store.get_entry_mut(&name) {
        Some(e) => e,
        None => {
            anyhow::bail!("Subscription '{name}' not found");
        }
    };

    if let Some(a) = amount {
        entry.amount = a;
    }
    if let Some(c) = billing_cycle {
        entry.billing_cycle = c;
    }
    if let Some(d) = next_date {
        entry.next_billing_date = parse_datetime(&d)?;
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

    entry.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Subscription '{}' updated", name.green()));

    Ok(())
}


