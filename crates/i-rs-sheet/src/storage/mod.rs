use crate::models::{SheetEntry, SheetStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<SheetStore> {
    let mut storage = Storage::<SheetStore>::new("sheet");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &SheetStore) -> anyhow::Result<()> {
    let storage = Storage::<SheetStore>::new("sheet");
    storage.save_data(store)
}


pub fn add_entry(store: &mut SheetStore, entry: SheetEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut SheetStore, id: &str) -> Option<SheetEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a SheetStore, id: &str) -> Option<&'a SheetEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a SheetStore, tag: Option<&'a str>) -> Vec<&'a SheetEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}