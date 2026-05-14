use crate::presentation::{print_warning, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let appliance = match store.get_by_name(&name) {
        Some(a) => a,
        None => {
            anyhow::bail!("Appliance '{}' not found", name);
        }
    };

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct ApplianceDetail {
            id: String,
            name: String,
            brand: String,
            model: String,
            purchase_date: String,
            lifespan_years: u32,
            expiry_date: String,
            days_until_expiry: i64,
            is_expired: bool,
            needs_replacement_soon: bool,
            tags: Vec<String>,
            remark: Vec<String>,
            maintenance_records: Vec<MaintenanceDetail>,
        }

        #[derive(serde::Serialize)]
        struct MaintenanceDetail {
            date: String,
            description: String,
            created_at: String,
        }

        let details = ApplianceDetail {
            id: appliance.id.clone(),
            name: appliance.name.clone(),
            brand: appliance.brand.clone(),
            model: appliance.model.clone(),
            purchase_date: appliance.purchase_date.format("%Y-%m-%d").to_string(),
            lifespan_years: appliance.lifespan_years,
            expiry_date: appliance.expiry_date().format("%Y-%m-%d").to_string(),
            days_until_expiry: appliance.days_until_expiry(),
            is_expired: appliance.is_expired(),
            needs_replacement_soon: appliance.needs_replacement_soon(),
            tags: appliance.tags.clone(),
            remark: appliance.remark.clone(),
            maintenance_records: appliance
                .maintenance_records
                .iter()
                .map(|r| MaintenanceDetail {
                    date: r.date.format("%Y-%m-%d").to_string(),
                    description: r.description.clone(),
                    created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect(),
        };

        println!("{}", output_item(&details, format));
        return Ok(());
    }

    println!();
    println!("{}", "Appliance Details".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    println!("  {:12} {}", "Name:".dimmed(), appliance.name.green());
    println!("  {:12} {}", "Brand:".dimmed(), appliance.brand);
    println!("  {:12} {}", "Model:".dimmed(), appliance.model);
    println!("  {:12} {}", "Purchase:".dimmed(), appliance.purchase_date.format("%Y-%m-%d"));
    println!("  {:12} {} years", "Lifespan:".dimmed(), appliance.lifespan_years);
    println!("  {:12} {}", "Expires:".dimmed(), appliance.expiry_date().format("%Y-%m-%d"));

    let days = appliance.days_until_expiry();
    let status = if appliance.is_expired() {
        format!("{} (expired {} days ago)", "EXPIRED".red(), days.abs())
    } else if appliance.needs_replacement_soon() {
        format!("{} ({} days left)", "REPLACE SOON".yellow(), days)
    } else {
        format!("{} ({} days left)", "OK".green(), days)
    };
    println!("  {:12} {}", "Status:".dimmed(), status);

    if !appliance.tags.is_empty() {
        println!("  {:12} {}", "Tags:".dimmed(), appliance.tags.join(", "));
    }

    if !appliance.remark.is_empty() {
        println!("  {:12} {}", "Remark:".dimmed(), appliance.remark.join(", "));
    }

    if !appliance.maintenance_records.is_empty() {
        println!();
        println!("{}", "Maintenance Records:".bold().cyan());
        for record in &appliance.maintenance_records {
            println!("  {} - {}", record.date.format("%Y-%m-%d").dimmed(), record.description);
        }
    } else {
        println!();
        print_warning("No maintenance records.");
    }

    Ok(())
}
