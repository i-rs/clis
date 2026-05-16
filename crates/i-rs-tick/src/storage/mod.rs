use crate::models::{TickEntry, TickStore};

i_rs_core::create_store!(TickStore, "tick");

pub fn filter_by_tag<'a>(store: &'a TickStore, tag: Option<&'a str>) -> Vec<&'a TickEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
