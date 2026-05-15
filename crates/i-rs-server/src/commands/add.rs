use crate::models::Server;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    host: String,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.servers.contains_key(&name) {
        anyhow::bail!("Server '{name}' already exists");
    }

    let port = port.unwrap_or(22);

    if let Some(ref pwd) = password {
        storage::store_password(&name, pwd)?;
    }

    let now = Utc::now();
    let server = Server {
        name: name.clone(),
        host,
        port,
        user,
        password: None,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, server);
    storage::save_store(&store)?;

    print_success(&format!("✓ Server '{}' added successfully", name.green()));

    if password.is_some() {
        println!("  {}", "Password stored securely in keychain".dimmed());
    }

    Ok(())
}
