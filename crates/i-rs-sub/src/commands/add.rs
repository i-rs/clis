use crate::models::SubEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, Utc};
use i_rs_core::validate_name;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

#[allow(clippy::too_many_arguments)]
pub fn handle_add(
    name: String,
    amount: f64,
    currency: String,
    billing_cycle: String,
    start_date: String,
    url: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&name) {
        anyhow::bail!("Subscription '{name}' already exists");
    }

    let start = parse_datetime(&start_date)?;
    let next = calculate_next_billing(&start, &billing_cycle);

    let now = Utc::now();
    let entry = SubEntry {
        name: name.clone(),
        amount,
        currency,
        billing_cycle,
        next_billing_date: next,
        url,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Subscription '{}' added", name.green()));

    Ok(())
}



fn calculate_next_billing(start: &DateTime<Utc>, cycle: &str) -> DateTime<Utc> {
    let now = Utc::now();
    let mut next = *start;
    let days = match cycle {
        "daily" => 1,
        "weekly" => 7,
        "monthly" => 30,
        "quarterly" => 90,
        "yearly" => 365,
        _ => 30,
    };
    while next < now {
        next += chrono::Duration::days(days);
    }
    next
}
