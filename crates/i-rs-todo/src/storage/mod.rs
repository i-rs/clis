use crate::models::TodoStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<TodoStore> {
    let mut storage = Storage::<TodoStore>::new("todo");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &TodoStore) -> anyhow::Result<()> {
    let storage = Storage::<TodoStore>::new("todo");
    storage.save_data(store)
}

