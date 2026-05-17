use crate::models::KeyStore;
use anyhow::Result;
use keyring::use_native_store;
use keyring_core::Entry;

i_rs_core::create_store!(KeyStore, "keys");

const SERVICE_NAME: &str = "i-rs-keys";

pub fn init_keyring() {
    let _ = use_native_store(false);
}

pub fn store_key(name: &str, key: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, name)
        .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {e}"))?;
    entry
        .set_password(key)
        .map_err(|e| anyhow::anyhow!("Failed to store key: {e}"))?;
    Ok(())
}

pub fn get_key(name: &str) -> Result<Option<String>> {
    match Entry::new(SERVICE_NAME, name) {
        Ok(entry) => match entry.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(e) => {
                let err_str = format!("{e}");
                if err_str.contains("NoEntry") || err_str.contains("not found") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Failed to get key: {e}"))
                }
            }
        },
        Err(_) => Ok(None),
    }
}

pub fn delete_key(name: &str) -> Result<()> {
    if let Ok(entry) = Entry::new(SERVICE_NAME, name) {
        let _ = entry.delete_credential();
    }
    Ok(())
}
