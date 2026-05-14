use crate::models::Book;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<Store> {
    let mut storage = Storage::<Store>::new("read");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &Store) -> anyhow::Result<()> {
    let storage = Storage::<Store>::new("read");
    storage.save_data(store)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Store {
    pub books: HashMap<String, Book>,
}

pub fn add_book(book: Book, store: &mut Store) {
    store.books.insert(book.name.clone(), book);
}

pub fn get_book<'a>(name: &str, store: &'a Store) -> Option<&'a Book> {
    store.books.get(name)
}

pub fn get_book_mut<'a>(name: &str, store: &'a mut Store) -> Option<&'a mut Book> {
    store.books.get_mut(name)
}

pub fn delete_book(name: &str, store: &mut Store) -> Option<Book> {
    store.books.remove(name)
}

pub fn list_books(store: &Store) -> Vec<&Book> {
    store.books.values().collect()
}

pub fn filter_by_tag<'a>(tag: &str, store: &'a Store) -> Vec<&'a Book> {
    store
        .books
        .values()
        .filter(|book| book.tags.iter().any(|t| t == tag))
        .collect()
}
