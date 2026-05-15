use crate::models::{CycleEntry, CycleStore};

i_rs_core::create_store!(CycleStore, "cycle");

pub fn filter_by_tag<'a>(store: &'a CycleStore, tag: Option<&'a str>) -> Vec<&'a CycleEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}