use crate::presentation::{format_table, print_appliance_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let appliances: Vec<_> = if let Some(ref t) = tag {
        store.filter_by_tag(t)
    } else {
        store.get_all_appliances()
    };

    if appliances.is_empty() {
        if format.is_json() {
            let filter = tag.map(|t| format!("tag: {t}"));
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format));
        } else {
            print_warning("No appliances found.");
        }
        return Ok(());
    }

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            id: String,
            name: String,
            brand: String,
            model: String,
            purchase_date: String,
            lifespan_years: u32,
            expiry_date: String,
            days_until_expiry: i64,
            tags: Vec<String>,
            maintenance_count: usize,
        }

        let items: Vec<ListItem> = appliances
            .iter()
            .map(|a| ListItem {
                id: a.id.clone(),
                name: a.name.clone(),
                brand: a.brand.clone(),
                model: a.model.clone(),
                purchase_date: a.purchase_date.format("%Y-%m-%d").to_string(),
                lifespan_years: a.lifespan_years,
                expiry_date: a.expiry_date().format("%Y-%m-%d").to_string(),
                days_until_expiry: a.days_until_expiry(),
                tags: a.tags.clone(),
                maintenance_count: a.maintenance_records.len(),
            })
            .collect();

        let filter = tag.map(|t| format!("tag: {t}"));
        println!("{}", output_list(&items, items.len(), filter.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&appliances);
    println!("\n{table}");

    print_appliance_count(appliances.len());

    Ok(())
}
