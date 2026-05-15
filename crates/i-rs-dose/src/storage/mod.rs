use crate::models::{DoseEntry, DoseStore};

i_rs_core::create_store!(DoseStore, "dose");

pub fn filter_by_tag<'a>(store: &'a DoseStore, tag: Option<&'a str>) -> Vec<&'a DoseEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
