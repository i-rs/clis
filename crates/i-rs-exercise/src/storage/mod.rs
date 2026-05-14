use crate::models::ExerciseStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<ExerciseStore> {
    let mut storage = Storage::<ExerciseStore>::new("exercise");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &ExerciseStore) -> anyhow::Result<()> {
    let storage = Storage::<ExerciseStore>::new("exercise");
    storage.save_data(store)
}

