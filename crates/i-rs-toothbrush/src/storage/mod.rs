use crate::models::{ToothbrushEntry, ToothbrushStore};

i_rs_core::create_store!(ToothbrushStore, "toothbrush");

pub fn filter_by_tag<'a>(store: &'a ToothbrushStore, tag: Option<&'a str>) -> Vec<&'a ToothbrushEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}