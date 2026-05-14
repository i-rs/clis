use crate::models::{Birthday, BirthdayStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<BirthdayStore> {
    let mut storage = Storage::<BirthdayStore>::new("birthday");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &BirthdayStore) -> anyhow::Result<()> {
    let storage = Storage::<BirthdayStore>::new("birthday");
    storage.save_data(store)
}


pub fn add_birthday(store: &mut BirthdayStore, birthday: Birthday) {
    store.birthdays.insert(birthday.name.clone(), birthday);
}

pub fn remove_birthday(store: &mut BirthdayStore, name: &str) -> Option<Birthday> {
    store.birthdays.remove(name)
}

pub fn get_birthday<'a>(store: &'a BirthdayStore, name: &str) -> Option<&'a Birthday> {
    store.birthdays.get(name)
}

pub fn get_birthday_mut<'a>(store: &'a mut BirthdayStore, name: &str) -> Option<&'a mut Birthday> {
    store.birthdays.get_mut(name)
}

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

pub fn get_all_birthdays<'a>(store: &'a BirthdayStore) -> Vec<&'a Birthday> {
    store.birthdays.values().collect()
}

pub fn get_upcoming_birthdays<'a>(store: &'a BirthdayStore, days: i64) -> Vec<&'a Birthday> {
    let mut birthdays: Vec<&Birthday> = store.birthdays.values().collect();
    birthdays.retain(|b| b.is_upcoming(days));
    birthdays.sort_by_key(|b| b.days_until_birthday());
    birthdays
}
