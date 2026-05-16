use crate::models::{WantEntry, WantStore};

i_rs_core::create_store!(WantStore, "want");

pub fn filter_by_tag<'a>(store: &'a WantStore, tag: Option<&'a str>) -> Vec<&'a WantEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
