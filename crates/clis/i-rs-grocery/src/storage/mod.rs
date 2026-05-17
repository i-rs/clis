use crate::models::{GroceryItem, GroceryStore};
use anyhow::Result;
use chrono::Utc;

i_rs_core::create_store!(GroceryStore, "grocery");

pub fn add_item(
    store: &mut GroceryStore,
    name: String,
    quantity: i32,
    unit: String,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<GroceryItem> {
    let now = Utc::now();
    let item = GroceryItem {
        name,
        quantity,
        unit,
        purchased: false,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(item.clone());
    Ok(item)
}

pub fn delete_item(store: &mut GroceryStore, name: &str) -> Result<GroceryItem> {
    store
        .remove_entry(name)
        .ok_or_else(|| anyhow::anyhow!("Item '{name}' not found"))
}

pub fn toggle_purchased(store: &mut GroceryStore, name: &str) -> Result<GroceryItem> {
    let item = store
        .get_entry_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Item '{name}' not found"))?;

    item.purchased = !item.purchased;
    item.updated_at = Utc::now();

    Ok(item.clone())
}

pub fn update_item(
    store: &mut GroceryStore,
    name: &str,
    quantity: Option<i32>,
    unit: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<GroceryItem> {
    let item = store
        .get_entry_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Item '{name}' not found"))?;

    if let Some(q) = quantity {
        item.quantity = q;
    }
    if let Some(u) = unit {
        item.unit = u;
    }
    if let Some(t) = tags {
        item.tags = t;
    }
    if let Some(r) = remark {
        item.remark = r;
    }
    item.updated_at = Utc::now();

    Ok(item.clone())
}

pub fn clear_purchased(store: &mut GroceryStore) -> Result<usize> {
    let purchased_names: Vec<String> = store
        .entries
        .iter()
        .filter(|(_, item)| item.purchased)
        .map(|(name, _)| name.clone())
        .collect();

    let count = purchased_names.len();
    for name in purchased_names {
        store.remove_entry(&name);
    }

    Ok(count)
}
