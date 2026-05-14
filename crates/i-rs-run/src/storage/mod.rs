use crate::models::RunStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<RunStore> {
    let mut storage = Storage::<RunStore>::new("run");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &RunStore) -> anyhow::Result<()> {
    let storage = Storage::<RunStore>::new("run");
    storage.save_data(store)
}

