use crate::presentation::{format_table, print_server_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let servers: Vec<&crate::models::Server> = storage::filter_servers_by_tag(&store, tag.as_deref());

    if servers.is_empty() {
        if format.is_json() {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No servers found.");
        }
        return Ok(());
    }

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            host: String,
            port: u16,
            user: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<ListItem> = servers.iter().map(|s| ListItem {
            name: s.name.clone(),
            host: s.host.clone(),
            port: s.port,
            user: s.user.clone(),
            tags: s.tags.clone(),
            remark: s.remark.clone(),
            created_at: s.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: s.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&servers);
    println!("\n{table}");

    print_server_count(servers.len());

    Ok(())
}