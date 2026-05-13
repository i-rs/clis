use crate::presentation::{format_table, print_server_count, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let servers: Vec<&crate::models::Server> = storage::filter_servers_by_tag(&store, tag.as_deref());

    if servers.is_empty() {
        print_warning("No servers found.");
        return Ok(());
    }

    let table = format_table(&servers);
    println!("\n{}", table);

    print_server_count(servers.len());

    Ok(())
}
