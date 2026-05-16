use crate::models::{BedEntry, BedStore};

i_rs_core::create_store!(BedStore, "bed");

pub fn filter_by_tag<'a>(store: &'a BedStore, tag: Option<&'a str>) -> Vec<&'a BedEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
