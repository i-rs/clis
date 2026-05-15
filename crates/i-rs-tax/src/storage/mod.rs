use crate::models::{TaxRecord, TaxStore};

i_rs_core::create_store!(TaxStore, "tax");

pub fn filter_by_tag<'a>(store: &'a TaxStore, tag: Option<&'a str>) -> Vec<&'a TaxRecord> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag.contains(t)))
            .collect(),
        None => store.entries.values().collect(),
    }
}
