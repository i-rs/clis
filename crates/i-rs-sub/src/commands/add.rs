use crate::models::SubEntry;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

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
        print_error(&e.message);
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&name) {
        print_error(&format!("Subscription '{}' already exists", name));
        anyhow::bail!("Subscription '{}' already exists", name);
    }

    let start = parse_date(&start_date)?;
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

fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];
    for format in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }
    if let Ok(naive) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap()));
    }
    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
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
        next = next + chrono::Duration::days(days);
    }
    next
}
