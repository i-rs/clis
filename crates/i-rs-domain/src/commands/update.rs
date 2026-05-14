use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

pub fn handle_update(
    name: String,
    expiry_date: Option<String>,
    registrar: Option<String>,
    password: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let domain = match storage::get_domain_mut(&mut store, &name) {
        Some(d) => d,
        None => {
            anyhow::bail!("Domain '{}' not found", name);
        }
    };

    if let Some(expiry) = expiry_date {
        domain.expiry_date = parse_datetime(&expiry)?;
    }
    if let Some(registrar) = registrar {
        domain.registrar = Some(registrar);
    }
    if let Some(password) = password {
        storage::store_password(&name, &password)?;
        println!("{}", "Password updated and stored securely in keychain".green());
    }
    if let Some(tag) = tag {
        domain.tags = tag;
    }
    if let Some(remark) = remark {
        domain.remark = remark;
    }

    domain.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Domain '{}' updated successfully", name.green()));

    Ok(())
}


