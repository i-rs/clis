use crate::models::MaintenanceRecord;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

#[allow(clippy::too_many_arguments)]
pub fn handle_update(
    name: String,
    brand: Option<String>,
    model: Option<String>,
    lifespan_years: Option<u32>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    add_maintenance: Option<String>,
    maintenance_date: Option<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let appliance = match store.get_by_name_mut(&name) {
        Some(a) => a,
        None => {
            anyhow::bail!("Appliance '{name}' not found");
        }
    };

    i_rs_core::update_field!(appliance.brand, brand);
    i_rs_core::update_field!(appliance.model, model);
    i_rs_core::update_field!(appliance.lifespan_years, lifespan_years);
    i_rs_core::update_field!(appliance.tags, tags);
    i_rs_core::update_field!(appliance.remark, remark);

    if let Some(desc) = add_maintenance {
        let date = if let Some(date_str) = maintenance_date {
            parse_date(&date_str)?
        } else {
            Utc::now().date_naive()
        };

        let record = MaintenanceRecord {
            date,
            description: desc,
            created_at: Utc::now(),
        };
        appliance.maintenance_records.push(record);
        print_success(&format!(
            "✓ Maintenance record added for '{}'",
            name.green()
        ));
    }

    appliance.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!("✓ Appliance '{}' updated", name.green()));

    Ok(())
}
