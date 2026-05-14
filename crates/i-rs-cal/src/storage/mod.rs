use crate::models::{CalEntry, CalStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<CalStore> {
    let mut storage = Storage::<CalStore>::new("cal");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &CalStore) -> anyhow::Result<()> {
    let storage = Storage::<CalStore>::new("cal");
    storage.save_data(store)
}


pub fn add_entry(store: &mut CalStore, entry: CalEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut CalStore, id: &str) -> Option<CalEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a CalStore, id: &str) -> Option<&'a CalEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a CalStore, tag: Option<&'a str>) -> Vec<&'a CalEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}