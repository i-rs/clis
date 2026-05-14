use crate::models::{WalkdogEntry, WalkdogStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<WalkdogStore> {
    let mut storage = Storage::<WalkdogStore>::new("walkdog");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &WalkdogStore) -> anyhow::Result<()> {
    let storage = Storage::<WalkdogStore>::new("walkdog");
    storage.save_data(store)
}


pub fn add_entry(store: &mut WalkdogStore, entry: WalkdogEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut WalkdogStore, id: &str) -> Option<WalkdogEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a WalkdogStore, id: &str) -> Option<&'a WalkdogEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a WalkdogStore, tag: Option<&'a str>) -> Vec<&'a WalkdogEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}