use crate::models::{SparkEntry, SparkStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<SparkStore> {
    let mut storage = Storage::<SparkStore>::new("spark");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &SparkStore) -> anyhow::Result<()> {
    let storage = Storage::<SparkStore>::new("spark");
    storage.save_data(store)
}


pub fn add_entry(store: &mut SparkStore, entry: SparkEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut SparkStore, id: &str) -> Option<SparkEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a SparkStore, id: &str) -> Option<&'a SparkEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a SparkStore, tag: Option<&'a str>) -> Vec<&'a SparkEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
