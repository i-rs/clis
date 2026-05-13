use crate::models::Server;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    host: String,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    tag: Vec<String>,
    note: Vec<String>,
) -> Result<()> {
    let store = storage::load_store()?;

    if store.servers.contains_key(&name) {
        print_error(&format!("Server '{}' already exists", name));
        anyhow::bail!("Server '{}' already exists", name);
    }

    let port = port.unwrap_or(22);

    if let Some(ref pwd) = password {
        storage::store_password(&name, pwd)?;
    }

    let server = Server {
        name: name.clone(),
        host,
        port,
        user,
        password: None,
        tags: tag,
        notes: note,
    };

    let mut store = storage::load_store()?;
    storage::add_server(&mut store, server);
    storage::save_store(&store)?;

    print_success(&format!("✓ Server '{}' added successfully", name.green()));

    if password.is_some() {
        println!("  {}", "Password stored securely in keychain".dimmed());
    }

    Ok(())
}
