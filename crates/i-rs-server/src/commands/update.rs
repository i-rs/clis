use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let server = match store.get_entry_mut(&name) {
        Some(s) => s,
        None => {
            anyhow::bail!("Server '{name}' not found");
        }
    };

    i_rs_core::update_field!(server.host, host);
    i_rs_core::update_field!(server.port, port);
    if let Some(user) = user {
        server.user = Some(user);
    }
    if let Some(password) = password {
        storage::store_password(&name, &password)?;
        println!("{}", "Password updated and stored securely in keychain".green());
    }
    i_rs_core::update_field!(server.tags, tag);
    i_rs_core::update_field!(server.remark, remark);

    server.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Server '{}' updated successfully", name.green()));

    Ok(())
}
