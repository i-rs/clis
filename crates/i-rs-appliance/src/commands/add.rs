use crate::models::Appliance;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_add(
    name: String,
    brand: String,
    model: String,
    purchase_date: String,
    lifespan_years: u32,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let purchase_date = parse_date(&purchase_date)?;

    let mut store = storage::load_store()?;

    if store.get_by_name(&name).is_some() {
        anyhow::bail!("Appliance '{name}' already exists");
    }

    let now = Utc::now();
    let appliance_name = name.clone();
    let appliance = Appliance {
        id: Uuid::new_v4().to_string(),
        name,
        brand,
        model,
        purchase_date: purchase_date
            .and_hms_opt(0, 0, 0)
            .expect("0:00:00 is always valid")
            .and_utc(),
        lifespan_years,
        tags,
        remark,
        maintenance_records: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    store.add_entry(appliance);
    storage::save_store(&store)?;

    print_success(&format!("✓ Appliance added: {}", appliance_name.green()));

    Ok(())
}
