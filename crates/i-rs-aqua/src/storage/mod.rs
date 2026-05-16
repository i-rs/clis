use crate::models::{AquaEntry, AquaStore};

i_rs_core::create_store!(AquaStore, "aqua");

pub fn filter_by_tag<'a>(store: &'a AquaStore, tag: Option<&'a str>) -> Vec<&'a AquaEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
