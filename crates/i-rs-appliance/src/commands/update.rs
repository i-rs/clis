use crate::models::MaintenanceRecord;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{NaiveDate, Utc};
use owo_colors::OwoColorize;

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
            print_error(&format!("Appliance '{}' not found", name));
            anyhow::bail!("Appliance '{}' not found", name);
        }
    };

    if let Some(b) = brand {
        appliance.brand = b;
    }
    if let Some(m) = model {
        appliance.model = m;
    }
    if let Some(l) = lifespan_years {
        appliance.lifespan_years = l;
    }
    if let Some(t) = tags {
        appliance.tags = t;
    }
    if let Some(r) = remark {
        appliance.remark = r;
    }

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
        print_success(&format!("✓ Maintenance record added for '{}'", name.green()));
    }

    appliance.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!("✓ Appliance '{}' updated", name.green()));

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
