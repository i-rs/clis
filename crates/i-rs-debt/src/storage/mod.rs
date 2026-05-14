use crate::models::{Debt, Store};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<Store> {
    let mut storage = Storage::<Store>::new("debt");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &Store) -> anyhow::Result<()> {
    let storage = Storage::<Store>::new("debt");
    storage.save_data(store)
}


pub fn add_debt(store: &mut Store, debt: Debt) {
    store.debts.insert(debt.name.clone(), debt);
}

pub fn get_debt<'a>(store: &'a Store, name: &str) -> Option<&'a Debt> {
    store.debts.get(name)
}

pub fn get_debt_mut<'a>(store: &'a mut Store, name: &str) -> Option<&'a mut Debt> {
    store.debts.get_mut(name)
}

pub fn delete_debt(store: &mut Store, name: &str) -> bool {
    store.debts.remove(name).is_some()
}

pub fn list_debts(store: &Store) -> Vec<&Debt> {
    store.debts.values().collect()
}

#[allow(dead_code)]
pub fn filter_by_tag<'a>(store: &'a Store, tag: &str) -> Vec<&'a Debt> {
    store
        .debts
        .values()
        .filter(|d| d.tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()))
        .collect()
}
