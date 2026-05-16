use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    url: Option<String>,
    account: Option<String>,
    password: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match store.get_entry_mut(&name) {
        Some(e) => e,
        None => {
            anyhow::bail!("Entry '{name}' not found");
        }
    };

    i_rs_core::update_field!(entry.url, url);
    if let Some(account) = account {
        entry.account = Some(account);
    }
    if let Some(password) = password {
        storage::store_password(&name, &password)?;
        println!("{}", "Password updated and stored securely in keychain".green());
    }
    i_rs_core::update_field!(entry.tags, tag);
    i_rs_core::update_field!(entry.remark, remark);

    entry.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' updated successfully", name.green()));

    Ok(())
}
