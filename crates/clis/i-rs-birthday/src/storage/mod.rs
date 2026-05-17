use crate::models::{Birthday, BirthdayStore};

i_rs_core::create_store!(BirthdayStore, "birthday");

pub fn filter_by_tag<'a>(store: &'a BirthdayStore, tag: Option<&str>) -> Vec<&'a Birthday> {
    if let Some(tag) = tag {
        store
            .birthdays
            .values()
            .filter(|b| b.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.birthdays.values().collect()
    }
}

pub fn get_all_birthdays(store: &BirthdayStore) -> Vec<&Birthday> {
    store.birthdays.values().collect()
}

pub fn get_upcoming_birthdays(store: &BirthdayStore, days: i64) -> Vec<&Birthday> {
    let mut birthdays: Vec<&Birthday> = store.birthdays.values().collect();
    birthdays.retain(|b| b.is_upcoming(days));
    birthdays.sort_by_key(|b| b.days_until_birthday());
    birthdays
}
