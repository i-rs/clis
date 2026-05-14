use crate::models::MoodStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<MoodStore> {
    let mut storage = Storage::<MoodStore>::new("mood");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &MoodStore) -> anyhow::Result<()> {
    let storage = Storage::<MoodStore>::new("mood");
    storage.save_data(store)
}

