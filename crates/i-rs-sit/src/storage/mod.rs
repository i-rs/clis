use crate::models::{SitEntry, SitStore};

i_rs_core::create_store!(SitStore, "sit");

pub fn filter_by_tag<'a>(store: &'a SitStore, tag: Option<&'a str>) -> Vec<&'a SitEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}