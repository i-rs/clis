use crate::models::{Birthday, BirthdayStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("birthdays.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("birthdays.json")
    }
}

pub fn load_store() -> Result<BirthdayStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(BirthdayStore::default())
    }
}

pub fn save_store(store: &BirthdayStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
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
