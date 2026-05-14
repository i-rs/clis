use crate::models::{StepEntry, StepStore};
use chrono::NaiveDate;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<StepStore> {
    let mut storage = Storage::<StepStore>::new("step");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &StepStore) -> anyhow::Result<()> {
    let storage = Storage::<StepStore>::new("step");
    storage.save_data(store)
}


pub fn add_entry(store: &mut StepStore, entry: StepEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut StepStore, date: &NaiveDate) -> Option<StepEntry> {
    store.remove_entry(date)
}

pub fn get_entry<'a>(store: &'a StepStore, date: &NaiveDate) -> Option<&'a StepEntry> {
    store.get_entry(date)
}

pub fn get_entry_mut<'a>(store: &'a mut StepStore, date: &NaiveDate) -> Option<&'a mut StepEntry> {
    store.get_entry_mut(date)
}
