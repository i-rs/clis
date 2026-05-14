use crate::models::{FeedpetEntry, FeedpetStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<FeedpetStore> {
    let mut storage = Storage::<FeedpetStore>::new("feedpet");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &FeedpetStore) -> anyhow::Result<()> {
    let storage = Storage::<FeedpetStore>::new("feedpet");
    storage.save_data(store)
}


pub fn add_entry(store: &mut FeedpetStore, entry: FeedpetEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut FeedpetStore, id: &str) -> Option<FeedpetEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a FeedpetStore, id: &str) -> Option<&'a FeedpetEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a FeedpetStore, tag: Option<&'a str>) -> Vec<&'a FeedpetEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}