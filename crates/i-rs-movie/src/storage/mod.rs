use crate::models::MovieStore;


i_rs_core::create_store!(MovieStore, "movie");


pub fn add_movie(store: &mut MovieStore, movie: crate::models::Movie) {
    store.add_movie(movie);
}

pub fn remove_movie(store: &mut MovieStore, name: &str) -> Option<crate::models::Movie> {
    store.remove_movie(name)
}