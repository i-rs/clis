use crate::models::Book;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR_NAME: &str = "i-rs";
const CONFIG_FILE_NAME: &str = "read.json";

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join(CONFIG_DIR_NAME);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir.join(CONFIG_FILE_NAME))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Store {
    pub books: HashMap<String, Book>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            books: HashMap::new(),
        }
    }
}

pub fn load_store() -> Result<Store> {
    let path = get_config_path()?;

    if !path.exists() {
        return Ok(Store::new());
    }

    let content = fs::read_to_string(&path)?;
    let store: Store = serde_json::from_str(&content)?;
    Ok(store)
}

pub fn save_store(store: &Store) -> Result<()> {
    let path = get_config_path()?;
    let content = serde_json::to_string_pretty(store)?;
    fs::write(path, content)?;
    Ok(())
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
