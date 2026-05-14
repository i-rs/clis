use crate::models::{KvEntry, KvStore};


i_rs_core::create_store!(KvStore, "kv");


pub fn add_entry(store: &mut KvStore, entry: KvEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut KvStore, name: &str) -> Option<KvEntry> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a KvStore, name: &str) -> Option<&'a KvEntry> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut KvStore, name: &str) -> Option<&'a mut KvEntry> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a KvStore, tag: Option<&'a str>) -> Vec<&'a KvEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
