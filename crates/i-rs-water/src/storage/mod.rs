use crate::models::{WaterEntry, WaterStore};

i_rs_core::create_store!(WaterStore, "water");

pub fn filter_by_tag<'a>(store: &'a WaterStore, tag: Option<&'a str>) -> Vec<&'a WaterEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
