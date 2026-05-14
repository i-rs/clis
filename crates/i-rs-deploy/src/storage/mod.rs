use crate::models::{DeployRecord, DeployStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<DeployStore> {
    let mut storage = Storage::<DeployStore>::new("deploy");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &DeployStore) -> anyhow::Result<()> {
    let storage = Storage::<DeployStore>::new("deploy");
    storage.save_data(store)
}


pub fn add_entry(store: &mut DeployStore, entry: DeployRecord) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut DeployStore, id: &str) -> Option<DeployRecord> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a DeployStore, id: &str) -> Option<&'a DeployRecord> {
    store.get_entry(id)
}

#[allow(dead_code)]
pub fn get_entry_mut<'a>(store: &'a mut DeployStore, id: &str) -> Option<&'a mut DeployRecord> {
    store.get_entry_mut(id)
}

pub fn filter_by_project<'a>(store: &'a DeployStore, project: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match project {
        Some(p) => store
            .entries
            .values()
            .filter(|e| e.project == p)
            .collect(),
        None => store.entries.values().collect(),
    }
}

pub fn filter_by_environment<'a>(store: &'a DeployStore, env: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match env {
        Some(e) => store
            .entries
            .values()
            .filter(|r| r.environment == e)
            .collect(),
        None => store.entries.values().collect(),
    }
}

pub fn filter_by_tag<'a>(store: &'a DeployStore, tag: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
