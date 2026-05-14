use crate::models::{WantEntry, WantStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<WantStore> {
    let mut storage = Storage::<WantStore>::new("want");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &WantStore) -> anyhow::Result<()> {
    let storage = Storage::<WantStore>::new("want");
    storage.save_data(store)
}


pub fn add_entry(store: &mut WantStore, entry: WantEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut WantStore, name: &str) -> Option<WantEntry> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a WantStore, name: &str) -> Option<&'a WantEntry> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut WantStore, name: &str) -> Option<&'a mut WantEntry> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a WantStore, tag: Option<&'a str>) -> Vec<&'a WantEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
