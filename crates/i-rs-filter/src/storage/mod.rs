use crate::models::{FilterEntry, FilterStore};

i_rs_core::create_store!(FilterStore, "filter");

pub fn filter_by_tag<'a>(store: &'a FilterStore, tag: Option<&'a str>) -> Vec<&'a FilterEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}