use crate::models::{LedgerEntry, LedgerStore};

i_rs_core::create_store!(LedgerStore, "ledger");

pub fn filter_by_category<'a>(
    store: &'a LedgerStore,
    category: Option<&'a str>,
) -> Vec<&'a LedgerEntry> {
    match category {
        Some(c) => store.get_entries_by_category(c),
        None => store.entries.values().collect(),
    }
}
