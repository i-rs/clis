use crate::models::{RecurEntry, RecurStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<RecurStore> {
    let mut storage = Storage::<RecurStore>::new("recur");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &RecurStore) -> anyhow::Result<()> {
    let storage = Storage::<RecurStore>::new("recur");
    storage.save_data(store)
}


pub fn add_entry(store: &mut RecurStore, entry: RecurEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut RecurStore, name: &str) -> Option<RecurEntry> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a RecurStore, name: &str) -> Option<&'a RecurEntry> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut RecurStore, name: &str) -> Option<&'a mut RecurEntry> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a RecurStore, tag: Option<&'a str>) -> Vec<&'a RecurEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
