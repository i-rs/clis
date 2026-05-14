use crate::models::{PigEntry, PigStore};


i_rs_core::create_store!(PigStore, "pig");


pub fn add_entry(store: &mut PigStore, entry: PigEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut PigStore, id: &str) -> Option<PigEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a PigStore, id: &str) -> Option<&'a PigEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a PigStore, tag: Option<&'a str>) -> Vec<&'a PigEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
