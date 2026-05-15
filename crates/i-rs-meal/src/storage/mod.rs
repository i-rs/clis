use crate::models::{MealEntry, MealStore};
use chrono::NaiveDate;

i_rs_core::create_store!(MealStore, "meal");

pub fn get_entries_by_date(store: &MealStore, date: NaiveDate) -> Vec<&MealEntry> {
    store.get_entries_by_date(date)
}
