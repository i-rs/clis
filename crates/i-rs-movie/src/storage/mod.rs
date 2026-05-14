use crate::models::MovieStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<MovieStore> {
    let mut storage = Storage::<MovieStore>::new("movie");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &MovieStore) -> anyhow::Result<()> {
    let storage = Storage::<MovieStore>::new("movie");
    storage.save_data(store)
}


pub fn add_movie(store: &mut MovieStore, movie: crate::models::Movie) {
    store.add_movie(movie);
}

pub fn remove_movie(store: &mut MovieStore, name: &str) -> Option<crate::models::Movie> {
    store.remove_movie(name)
}