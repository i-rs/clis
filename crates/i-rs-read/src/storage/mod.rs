use crate::models::{Book, ReadStore};


i_rs_core::create_store!(ReadStore, "read");

pub fn add_entry(book: Book, store: &mut ReadStore) {
    store.books.insert(book.name.clone(), book);
}

pub fn get_entry<'a>(name: &str, store: &'a ReadStore) -> Option<&'a Book> {
    store.books.get(name)
}

pub fn get_entry_mut<'a>(name: &str, store: &'a mut ReadStore) -> Option<&'a mut Book> {
    store.books.get_mut(name)
}

pub fn remove_entry(name: &str, store: &mut ReadStore) -> Option<Book> {
    store.books.remove(name)
}

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
