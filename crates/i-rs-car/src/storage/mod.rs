use crate::models::Store;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<Store> {
    let mut storage = Storage::<Store>::new("car");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &Store) -> anyhow::Result<()> {
    let storage = Storage::<Store>::new("car");
    storage.save_data(store)
}

