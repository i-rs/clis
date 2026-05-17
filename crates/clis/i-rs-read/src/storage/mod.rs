use crate::models::{Book, ReadStore};

i_rs_core::create_store!(ReadStore, "read");

pub fn list_entries(store: &ReadStore) -> Vec<&Book> {
    store.books.values().collect()
}

pub fn filter_by_tag<'a>(tag: &str, store: &'a ReadStore) -> Vec<&'a Book> {
    store
        .books
        .values()
        .filter(|book| book.tags.iter().any(|t| t == tag))
        .collect()
}
