use crate::models::{TaxRecord, TaxStore};


i_rs_core::create_store!(TaxStore, "tax");

pub fn add_entry(store: &mut TaxStore, entry: TaxRecord) {
    store.entries.insert(entry.name.clone(), entry);
}

pub fn remove_entry(store: &mut TaxStore, name: &str) -> Option<TaxRecord> {
    store.entries.remove(name)
}

pub fn get_entry<'a>(store: &'a TaxStore, name: &str) -> Option<&'a TaxRecord> {
    store.entries.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut TaxStore, name: &str) -> Option<&'a mut TaxRecord> {
    store.entries.get_mut(name)
}

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
