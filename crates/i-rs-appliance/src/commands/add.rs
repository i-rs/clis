use crate::models::Appliance;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{NaiveDate, Utc};
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
        print_error(&format!("Appliance '{}' already exists. Use update command instead.", name));
        anyhow::bail!("Appliance '{}' already exists", name);
    }

    let now = Utc::now();
    let appliance_name = name.clone();
    let appliance = Appliance {
        id: Uuid::new_v4().to_string(),
        name,
        brand,
        model,
        purchase_date: purchase_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        lifespan_years,
        tags,
        remark,
        maintenance_records: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    store.add_appliance(appliance);
    storage::save_store(&store)?;

    print_success(&format!("✓ Appliance added: {}", appliance_name.green()));

    Ok(())
}

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
