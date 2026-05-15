use crate::models::{PurifyEntry, PurifyStore};

i_rs_core::create_store!(PurifyStore, "purify");

pub fn filter_by_tag<'a>(store: &'a PurifyStore, tag: Option<&'a str>) -> Vec<&'a PurifyEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}