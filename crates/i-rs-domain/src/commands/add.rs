use crate::models::Domain;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

pub fn handle_add(
    name: String,
    expiry_date: String,
    registrar: Option<String>,
    password: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.domains.contains_key(&name) {
        anyhow::bail!("Domain '{name}' already exists");
    }

    let expiry = parse_datetime(&expiry_date)?;

    if let Some(ref pwd) = password {
        storage::store_password(&name, pwd)?;
    }

    let now = Utc::now();
    let domain = Domain {
        name: name.clone(),
        expiry_date: expiry,
        registrar,
        password: None,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_domain(&mut store, domain);
    storage::save_store(&store)?;

    print_success(&format!("✓ Domain '{}' added successfully", name.green()));

    if password.is_some() {
        println!("  {}", "Password stored securely in keychain".dimmed());
    }

    Ok(())
}


