use crate::models::{CalEntry, CalStore};


i_rs_core::create_store!(CalStore, "cal");


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