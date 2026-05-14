use crate::models::{LedgerEntry, LedgerStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<LedgerStore> {
    let mut storage = Storage::<LedgerStore>::new("ledger");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &LedgerStore) -> anyhow::Result<()> {
    let storage = Storage::<LedgerStore>::new("ledger");
    storage.save_data(store)
}


pub fn add_entry(store: &mut LedgerStore, entry: LedgerEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut LedgerStore, id: &str) -> Option<LedgerEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a LedgerStore, id: &str) -> Option<&'a LedgerEntry> {
    store.get_entry(id)
}

pub fn filter_by_category<'a>(store: &'a LedgerStore, category: Option<&'a str>) -> Vec<&'a LedgerEntry> {
    match category {
        Some(c) => store.get_entries_by_category(c),
        None => store.entries.values().collect(),
    }
}
