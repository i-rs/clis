use crate::models::{TaxRecord, TaxStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<TaxStore> {
    let mut storage = Storage::<TaxStore>::new("tax");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &TaxStore) -> anyhow::Result<()> {
    let storage = Storage::<TaxStore>::new("tax");
    storage.save_data(store)
}

pub fn add_entry(store: &mut TaxStore, entry: TaxRecord) {
    store.entries.insert(entry.name.clone(), entry);
}

pub fn remove_entry(store: &mut TaxStore, name: &str) -> Option<TaxRecord> {
    store.entries.remove(name)
}

#[allow(dead_code)]
pub fn get_entry<'a>(store: &'a TaxStore, name: &str) -> Option<&'a TaxRecord> {
    store.entries.get(name)
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn get_entry_mut<'a>(store: &'a mut TaxStore, name: &str) -> Option<&'a mut TaxRecord> {
    store.entries.get_mut(name)
}

#[allow(dead_code)]
pub fn filter_by_tag<'a>(store: &'a TaxStore, tag: Option<&'a str>) -> Vec<&'a TaxRecord> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag.contains(t)))
            .collect(),
        None => store.entries.values().collect(),
    }
}
