use crate::models::BudgetStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<BudgetStore> {
    let mut storage = Storage::<BudgetStore>::new("budget");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &BudgetStore) -> anyhow::Result<()> {
    let storage = Storage::<BudgetStore>::new("budget");
    storage.save_data(store)
}

