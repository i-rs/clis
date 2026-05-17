use crate::models::{RecurEntry, RecurStore};

i_rs_core::create_store!(RecurStore, "recur");

pub fn filter_by_tag<'a>(store: &'a RecurStore, tag: Option<&'a str>) -> Vec<&'a RecurEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
