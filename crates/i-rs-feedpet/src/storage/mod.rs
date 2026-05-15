use crate::models::{FeedpetEntry, FeedpetStore};

i_rs_core::create_store!(FeedpetStore, "feedpet");

pub fn filter_by_tag<'a>(store: &'a FeedpetStore, tag: Option<&'a str>) -> Vec<&'a FeedpetEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}