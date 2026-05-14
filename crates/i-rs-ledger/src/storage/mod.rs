use crate::models::{LedgerEntry, LedgerStore};


i_rs_core::create_store!(LedgerStore, "ledger");


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
