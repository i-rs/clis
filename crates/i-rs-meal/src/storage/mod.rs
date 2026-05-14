use crate::models::{MealEntry, MealStore};
use chrono::NaiveDate;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<MealStore> {
    let mut storage = Storage::<MealStore>::new("meal");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &MealStore) -> anyhow::Result<()> {
    let storage = Storage::<MealStore>::new("meal");
    storage.save_data(store)
}


pub fn add_entry(store: &mut MealStore, entry: MealEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut MealStore, id: &str) -> Option<MealEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a MealStore, id: &str) -> Option<&'a MealEntry> {
    store.get_entry(id)
}

pub fn get_entries_by_date(store: &MealStore, date: NaiveDate) -> Vec<&MealEntry> {
    store.get_entries_by_date(date)
}
