use crate::presentation::{format_table, print_domain_count, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let domains: Vec<&crate::models::Domain> = storage::filter_by_tag(&store, tag.as_deref());

    if domains.is_empty() {
        print_warning("No domains found.");
        return Ok(());
    }

    let table = format_table(&domains);
    println!("\n{}", table);

    print_domain_count(domains.len());

    Ok(())
}
