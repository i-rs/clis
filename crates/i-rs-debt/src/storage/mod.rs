use crate::models::{Debt, DebtStore};

i_rs_core::create_store!(DebtStore, "debt");

pub fn list_entries(store: &DebtStore) -> Vec<&Debt> {
    store.debts.values().collect()
}

pub fn filter_by_tag<'a>(store: &'a DebtStore, tag: &str) -> Vec<&'a Debt> {
    store
        .debts
        .values()
        .filter(|d| d.tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()))
        .collect()
}
