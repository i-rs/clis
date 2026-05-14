use crate::models::CyclingStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<CyclingStore> {
    let mut storage = Storage::<CyclingStore>::new("cycling");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &CyclingStore) -> anyhow::Result<()> {
    let storage = Storage::<CyclingStore>::new("cycling");
    storage.save_data(store)
}

