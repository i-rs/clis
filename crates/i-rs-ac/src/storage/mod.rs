use crate::models::{AcEntry, AcStore};

i_rs_core::create_store!(AcStore, "ac");

pub fn filter_by_tag<'a>(store: &'a AcStore, tag: Option<&'a str>) -> Vec<&'a AcEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}