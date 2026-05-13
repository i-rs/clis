use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    tag: Option<Vec<String>>,
    note: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let server = match storage::get_server_mut(&mut store, &name) {
        Some(s) => s,
        None => {
            print_error(&format!("Server '{}' not found", name));
            anyhow::bail!("Server '{}' not found", name);
        }
    };

    if let Some(host) = host {
        server.host = host;
    }
    if let Some(port) = port {
        server.port = port;
    }
    if let Some(user) = user {
        server.user = Some(user);
    }
    if let Some(password) = password {
        storage::store_password(&name, &password)?;
        println!("{}", "Password updated and stored securely in keychain".green());
    }
    if let Some(tag) = tag {
        server.tags = tag;
    }
    if let Some(note) = note {
        server.notes = note;
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Server '{}' updated successfully", name.green()));

    Ok(())
}
