use crate::models::{Gift, GiftType};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

#[allow(clippy::too_many_arguments)]
pub fn handle_add(
    name: String,
    gift_type: String,
    recipient: String,
    occasion: String,
    value: f64,
    date: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.gifts.contains_key(&name) {
        anyhow::bail!("Gift '{name}' already exists");
    }

    let parsed_type = match gift_type.to_lowercase().as_str() {
        "sent" | "s" => GiftType::Sent,
        "received" | "r" => GiftType::Received,
        _ => {
            anyhow::bail!("Invalid gift type: {gift_type}");
        }
    };

    let parsed_date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Invalid date format: {e}. Use YYYY-MM-DD"))?;
    let parsed_date = parsed_date
        .and_hms_opt(0, 0, 0)
        .expect("0:00:00 is always valid");
    let parsed_date = chrono::DateTime::<Utc>::from_naive_utc_and_offset(parsed_date, Utc);

    let now = Utc::now();
    let gift = Gift {
        name: name.clone(),
        gift_type: parsed_type,
        recipient,
        occasion,
        value,
        date: parsed_date,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(gift);
    storage::save_store(&store)?;

    print_success(&format!("✓ Gift '{}' added successfully", name.green()));

    Ok(())
}
