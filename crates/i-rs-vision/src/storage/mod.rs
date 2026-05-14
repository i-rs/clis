use crate::models::VisionStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<VisionStore> {
    let mut storage = Storage::<VisionStore>::new("vision");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &VisionStore) -> anyhow::Result<()> {
    let storage = Storage::<VisionStore>::new("vision");
    storage.save_data(store)
}

