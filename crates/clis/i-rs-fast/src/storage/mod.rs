use crate::models::{FastEntry, FastStore};

i_rs_core::create_store!(FastStore, "fast");

pub fn filter_by_tag<'a>(store: &'a FastStore, tag: Option<&'a str>) -> Vec<&'a FastEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
