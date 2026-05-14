use crate::models::WeightStore;
use anyhow::Result;

use i_rs_core::Storage;

pub fn load_store() -> Result<WeightStore> {
    let mut storage = Storage::<WeightStore>::new("weights");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &WeightStore) -> Result<()> {
    let storage = Storage::<WeightStore>::new("weights");
    storage.save_data(store)
}
