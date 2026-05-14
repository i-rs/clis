use crate::models::{ToothbrushEntry, ToothbrushStore};


i_rs_core::create_store!(ToothbrushStore, "toothbrush");


pub fn add_entry(store: &mut ToothbrushStore, entry: ToothbrushEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut ToothbrushStore, id: &str) -> Option<ToothbrushEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a ToothbrushStore, id: &str) -> Option<&'a ToothbrushEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a ToothbrushStore, tag: Option<&'a str>) -> Vec<&'a ToothbrushEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}