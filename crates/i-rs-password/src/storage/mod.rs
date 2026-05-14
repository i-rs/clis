use crate::models::{PasswordEntry, PasswordStore};
use anyhow::Result;
use keyring::use_native_store;
use keyring_core::Entry;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<PasswordStore> {
    let mut storage = Storage::<PasswordStore>::new("password");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &PasswordStore) -> anyhow::Result<()> {
    let storage = Storage::<PasswordStore>::new("password");
    storage.save_data(store)
}


const SERVICE_NAME: &str = "i-rs-password";

pub fn init_keyring() {
    let _ = use_native_store(false);
}


pub fn store_password(name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, name)
        .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;
    entry
        .set_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to store password: {}", e))?;
    Ok(())
}

pub fn get_password(name: &str) -> Result<Option<String>> {
    match Entry::new(SERVICE_NAME, name) {
        Ok(entry) => match entry.get_password() {
            Ok(pwd) => Ok(Some(pwd)),
            Err(e) => {
                let err_str = format!("{}", e);
                if err_str.contains("NoEntry") || err_str.contains("not found") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Failed to get password: {}", e))
                }
            }
        },
        Err(_) => Ok(None),
    }
}

pub fn delete_password(name: &str) -> Result<()> {
    match Entry::new(SERVICE_NAME, name) {
        Ok(entry) => {
            let _ = entry.delete_credential();
        }
        Err(_) => {}
    }
    Ok(())
}


pub fn add_entry(store: &mut PasswordStore, entry: PasswordEntry) {
    store.entries.insert(entry.name.clone(), entry);
}

pub fn remove_entry(store: &mut PasswordStore, name: &str) -> Option<PasswordEntry> {
    store.entries.remove(name)
}

pub fn get_entry<'a>(store: &'a PasswordStore, name: &str) -> Option<&'a PasswordEntry> {
    store.entries.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut PasswordStore, name: &str) -> Option<&'a mut PasswordEntry> {
    store.entries.get_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a PasswordStore, tag: Option<&str>) -> Vec<&'a PasswordEntry> {
    if let Some(tag) = tag {
        store
            .entries
            .values()
            .filter(|e| e.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.entries.values().collect()
    }
}
