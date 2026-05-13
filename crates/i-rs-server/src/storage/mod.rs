use crate::models::{Server, ServerStore};
use anyhow::Result;
use keyring::use_native_store;
use keyring_core::Entry;
use std::path::PathBuf;

const SERVICE_NAME: &str = "i-rs-server";

pub fn init_keyring() {
    let _ = use_native_store(false);
}

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("servers.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("servers.json")
    }
}

pub fn store_password(server_name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, server_name)
        .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;
    entry
        .set_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to store password: {}", e))?;
    Ok(())
}

pub fn get_password(server_name: &str) -> Result<Option<String>> {
    match Entry::new(SERVICE_NAME, server_name) {
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

pub fn delete_password(server_name: &str) -> Result<()> {
    match Entry::new(SERVICE_NAME, server_name) {
        Ok(entry) => {
            let _ = entry.delete_credential();
        }
        Err(_) => {}
    }
    Ok(())
}

pub fn load_store() -> Result<ServerStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(ServerStore::default())
    }
}

pub fn save_store(store: &ServerStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_server(store: &mut ServerStore, server: Server) {
    store.servers.insert(server.name.clone(), server);
}

pub fn remove_server(store: &mut ServerStore, name: &str) -> Option<Server> {
    store.servers.remove(name)
}

pub fn get_server<'a>(store: &'a ServerStore, name: &str) -> Option<&'a Server> {
    store.servers.get(name)
}

pub fn get_server_mut<'a>(store: &'a mut ServerStore, name: &str) -> Option<&'a mut Server> {
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
