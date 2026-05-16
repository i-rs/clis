use crate::models::{PigEntry, PigStore};

i_rs_core::create_store!(PigStore, "pig");

pub fn filter_by_tag<'a>(store: &'a PigStore, tag: Option<&'a str>) -> Vec<&'a PigEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
