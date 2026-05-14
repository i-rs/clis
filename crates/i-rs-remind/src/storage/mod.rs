use crate::models::{Remind, RemindStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<RemindStore> {
    let mut storage = Storage::<RemindStore>::new("remind");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &RemindStore) -> anyhow::Result<()> {
    let storage = Storage::<RemindStore>::new("remind");
    storage.save_data(store)
}


pub fn add_remind(store: &mut RemindStore, remind: Remind) {
    store.reminds.insert(remind.name.clone(), remind);
}

pub fn remove_remind(store: &mut RemindStore, name: &str) -> Option<Remind> {
    store.reminds.remove(name)
}

pub fn get_remind<'a>(store: &'a RemindStore, name: &str) -> Option<&'a Remind> {
    store.reminds.get(name)
}

pub fn get_remind_mut<'a>(store: &'a mut RemindStore, name: &str) -> Option<&'a mut Remind> {
    store.reminds.get_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a RemindStore, tag: Option<&str>) -> Vec<&'a Remind> {
    if let Some(tag) = tag {
        store
            .reminds
            .values()
            .filter(|r| r.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.reminds.values().collect()
    }
}
