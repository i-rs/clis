use crate::models::HeightStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<HeightStore> {
    let mut storage = Storage::<HeightStore>::new("height");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &HeightStore) -> anyhow::Result<()> {
    let storage = Storage::<HeightStore>::new("height");
    storage.save_data(store)
}

