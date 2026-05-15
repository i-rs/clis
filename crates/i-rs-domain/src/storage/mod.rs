use crate::models::{Domain, DomainStore};
use anyhow::Result;
use keyring::use_native_store;
use keyring_core::Entry;

i_rs_core::create_store!(DomainStore, "domain");

const SERVICE_NAME: &str = "i-rs-domain";

pub fn init_keyring() {
    let _ = use_native_store(false);
}

pub fn store_password(domain_name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, domain_name)
        .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {e}"))?;
    entry
        .set_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to store password: {e}"))?;
    Ok(())
}

pub fn get_password(domain_name: &str) -> Result<Option<String>> {
    match Entry::new(SERVICE_NAME, domain_name) {
        Ok(entry) => match entry.get_password() {
            Ok(pwd) => Ok(Some(pwd)),
            Err(e) => {
                let err_str = format!("{e}");
                if err_str.contains("NoEntry") || err_str.contains("not found") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Failed to get password: {e}"))
                }
            }
        },
        Err(_) => Ok(None),
    }
}

pub fn delete_password(domain_name: &str) -> Result<()> {
    if let Ok(entry) = Entry::new(SERVICE_NAME, domain_name) {
        let _ = entry.delete_credential();
    }
    Ok(())
}

pub fn filter_by_tag<'a>(store: &'a DomainStore, tag: Option<&str>) -> Vec<&'a Domain> {
    if let Some(tag) = tag {
        store
            .domains
            .values()
            .filter(|d| d.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.domains.values().collect()
    }
}
