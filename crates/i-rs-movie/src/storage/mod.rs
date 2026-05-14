use crate::models::MovieStore;


i_rs_core::create_store!(MovieStore, "movie");


pub fn add_entry(store: &mut MovieStore, movie: crate::models::Movie) {
    store.add_entry(movie);
}

pub fn remove_entry(store: &mut MovieStore, name: &str) -> Option<crate::models::Movie> {
    store.remove_entry(name)
}