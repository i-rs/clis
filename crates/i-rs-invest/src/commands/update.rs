use crate::models::AssetType;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    symbol: Option<String>,
    asset_type: Option<AssetType>,
    quantity: Option<f64>,
    buy_price: Option<f64>,
    current_price: Option<f64>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let investment = match storage::get_investment_mut(&mut store, &name) {
        Some(inv) => inv,
        None => {
            anyhow::bail!("Investment '{name}' not found");
        }
    };

    if let Some(sym) = symbol {
        investment.symbol = sym.to_uppercase();
    }

    if let Some(typ) = asset_type {
        investment.asset_type = typ;
    }

    if let Some(qty) = quantity {
        if qty <= 0.0 {
            anyhow::bail!("Quantity must be greater than 0");
        }
        investment.quantity = qty;
    }

    if let Some(price) = buy_price {
        if price <= 0.0 {
            anyhow::bail!("Buy price must be greater than 0");
        }
        investment.buy_price = price;
    }

    if let Some(cp) = current_price {
        if cp < 0.0 {
            anyhow::bail!("Current price cannot be negative");
        }
        investment.current_price = Some(cp);
    }

    if let Some(tags) = tag {
        investment.tags = tags;
    }

    if let Some(remarks) = remark {
        investment.remark = remarks;
    }

    investment.updated_at = Utc::now();

    storage::save_store(&store)?;

    println!("{}", format!("✓ Investment '{}' updated successfully", name.green()));

    Ok(())
}
