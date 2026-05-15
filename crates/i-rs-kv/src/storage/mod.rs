use crate::models::{KvEntry, KvStore};

i_rs_core::create_store!(KvStore, "kv");

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
