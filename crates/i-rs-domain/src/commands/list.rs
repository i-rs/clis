use crate::presentation::{format_table, print_domain_count, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let domains: Vec<&crate::models::Domain> = storage::filter_by_tag(&store, tag.as_deref());

    i_rs_core::handle_empty!(domains, format, tag.as_deref(), "No domains found.");

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            expiry_date: String,
            days_until_expiry: i64,
            is_expired: bool,
            registrar: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<ListItem> = domains.iter().map(|d| ListItem {
            name: d.name.clone(),
            expiry_date: d.expiry_date.format("%Y-%m-%d").to_string(),
            days_until_expiry: d.days_until_expiry(),
            is_expired: d.is_expired(),
            registrar: d.registrar.clone(),
            tags: d.tags.clone(),
            remark: d.remark.clone(),
            created_at: d.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: d.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&domains);
    println!("\n{table}");

    print_domain_count(domains.len());

    Ok(())
}