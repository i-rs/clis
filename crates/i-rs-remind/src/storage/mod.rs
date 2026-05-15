use crate::models::{Remind, RemindStore};


i_rs_core::create_store!(RemindStore, "remind");


pub fn add_entry(store: &mut RemindStore, remind: Remind) {
    store.reminds.insert(remind.name.clone(), remind);
}

pub fn remove_entry(store: &mut RemindStore, name: &str) -> Option<Remind> {
    store.reminds.remove(name)
}

pub fn get_entry<'a>(store: &'a RemindStore, name: &str) -> Option<&'a Remind> {
    store.reminds.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut RemindStore, name: &str) -> Option<&'a mut Remind> {
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
