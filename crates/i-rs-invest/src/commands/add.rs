use crate::models::{AssetType, Investment};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, Utc};
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    symbol: String,
    asset_type: AssetType,
    quantity: f64,
    buy_price: f64,
    buy_date: Option<String>,
    current_price: Option<f64>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.investments.contains_key(&name) {
        anyhow::bail!("Investment '{name}' already exists");
    }

    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    if quantity <= 0.0 {
        anyhow::bail!("Quantity must be greater than 0");
    }

    if buy_price <= 0.0 {
        anyhow::bail!("Buy price must be greater than 0");
    }

    if let Some(cp) = current_price
        && cp < 0.0 {
            anyhow::bail!("Current price cannot be negative");
        }

    let parsed_buy_date = if let Some(date_str) = buy_date {
        let date = i_rs_core::parse_date(&date_str)?;
        match date.and_hms_opt(0, 0, 0) {
            Some(dt) => DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc),
            None => Utc::now(),
        }
    } else {
        Utc::now()
    };

    let now = Utc::now();
    let investment = Investment {
        name: name.clone(),
        symbol: symbol.to_uppercase(),
        asset_type,
        quantity,
        buy_price,
        buy_date: parsed_buy_date,
        current_price,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, investment);
    storage::save_store(&store)?;

    println!("{}", format!("✓ Investment '{}' added successfully", name.green()));

    Ok(())
}
