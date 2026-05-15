use crate::models::{Server, ServerStore};
use anyhow::Result;
use keyring::use_native_store;
use keyring_core::Entry;


i_rs_core::create_store!(ServerStore, "server");


const SERVICE_NAME: &str = "i-rs-server";

pub fn init_keyring() {
    let _ = use_native_store(false);
}


pub fn store_password(server_name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, server_name)
        .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {e}"))?;
    entry
        .set_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to store password: {e}"))?;
    Ok(())
}

pub fn get_password(server_name: &str) -> Result<Option<String>> {
    match Entry::new(SERVICE_NAME, server_name) {
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

pub fn delete_password(server_name: &str) -> Result<()> {
    if let Ok(entry) = Entry::new(SERVICE_NAME, server_name) {
        let _ = entry.delete_credential();
    }
    Ok(())
}


pub fn add_entry(store: &mut ServerStore, server: Server) {
    store.servers.insert(server.name.clone(), server);
}

pub fn remove_entry(store: &mut ServerStore, name: &str) -> Option<Server> {
    store.servers.remove(name)
}

pub fn get_entry<'a>(store: &'a ServerStore, name: &str) -> Option<&'a Server> {
    store.servers.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut ServerStore, name: &str) -> Option<&'a mut Server> {
    store.servers.get_mut(name)
}

pub fn filter_servers_by_tag<'a>(store: &'a ServerStore, tag: Option<&str>) -> Vec<&'a Server> {
    if let Some(tag) = tag {
        store
            .servers
            .values()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.servers.values().collect()
    }
}
