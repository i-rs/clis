use crate::models::ApplianceStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<ApplianceStore> {
    let mut storage = Storage::<ApplianceStore>::new("appliance");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &ApplianceStore) -> anyhow::Result<()> {
    let storage = Storage::<ApplianceStore>::new("appliance");
    storage.save_data(store)
}

