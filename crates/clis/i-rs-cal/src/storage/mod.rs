use crate::models::{CalEntry, CalStore};

i_rs_core::create_store!(CalStore, "cal");

pub fn filter_by_tag<'a>(store: &'a CalStore, tag: Option<&'a str>) -> Vec<&'a CalEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
