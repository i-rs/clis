use crate::models::{SubEntry, SubStore};

i_rs_core::create_store!(SubStore, "sub");

pub fn filter_by_tag<'a>(store: &'a SubStore, tag: Option<&'a str>) -> Vec<&'a SubEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
