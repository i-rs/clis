use crate::models::{Debt, DebtStore};


i_rs_core::create_store!(DebtStore, "debt");


pub fn add_entry(store: &mut DebtStore, debt: Debt) {
    store.debts.insert(debt.name.clone(), debt);
}

pub fn get_entry<'a>(store: &'a DebtStore, name: &str) -> Option<&'a Debt> {
    store.debts.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut DebtStore, name: &str) -> Option<&'a mut Debt> {
    store.debts.get_mut(name)
}

pub fn remove_entry(store: &mut DebtStore, name: &str) -> bool {
    store.debts.remove(name).is_some()
}

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
