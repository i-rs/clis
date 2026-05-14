use crate::models::GiftType;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

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
        None => anyhow::bail!("Gift '{}' not found", name),
    };

    if let Some(gt) = gift_type {
        gift.gift_type = match gt.to_lowercase().as_str() {
            "sent" => GiftType::Sent,
            "received" => GiftType::Received,
            _ => anyhow::bail!("Invalid gift type. Use 'sent' or 'received'"),
        };
    }
    if let Some(r) = recipient {
        gift.recipient = r;
    }
    if let Some(o) = occasion {
        gift.occasion = o;
    }
    if let Some(v) = value {
        gift.value = v;
    }
    if let Some(d) = date {
        let parsed = parse_date(&d)?;
        gift.date = parsed.and_hms_opt(0, 0, 0).expect("0:00:00 is always valid").and_utc();
    }
    if let Some(t) = tag {
        gift.tags = t;
    }
    if let Some(r) = remark {
        gift.remark = r;
    }

    gift.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!("✓ Gift '{}' updated successfully", name.green()));
    Ok(())
}
