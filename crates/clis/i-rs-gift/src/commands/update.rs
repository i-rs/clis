use crate::models::GiftType;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

#[allow(clippy::too_many_arguments)]
pub fn handle_update(
    name: String,
    gift_type: Option<String>,
    recipient: Option<String>,
    occasion: Option<String>,
    value: Option<f64>,
    date: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let gift = match store.gifts.get_mut(&name) {
        Some(g) => g,
        None => anyhow::bail!("Gift '{name}' not found"),
    };

    if let Some(gt) = gift_type {
        gift.gift_type = match gt.to_lowercase().as_str() {
            "sent" => GiftType::Sent,
            "received" => GiftType::Received,
            _ => anyhow::bail!("Invalid gift type. Use 'sent' or 'received'"),
        };
    }
    i_rs_core::update_field!(gift.recipient, recipient);
    i_rs_core::update_field!(gift.occasion, occasion);
    i_rs_core::update_field!(gift.value, value);
    if let Some(d) = date {
        let parsed = parse_date(&d)?;
        gift.date = parsed
            .and_hms_opt(0, 0, 0)
            .expect("0:00:00 is always valid")
            .and_utc();
    }
    i_rs_core::update_field!(gift.tags, tag);
    i_rs_core::update_field!(gift.remark, remark);

    gift.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!("✓ Gift '{}' updated successfully", name.green()));
    Ok(())
}
