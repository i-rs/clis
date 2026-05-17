use crate::models::{WalkdogEntry, WalkdogStore};

i_rs_core::create_store!(WalkdogStore, "walkdog");

pub fn filter_by_tag<'a>(store: &'a WalkdogStore, tag: Option<&'a str>) -> Vec<&'a WalkdogEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
