use crate::models::BestByStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<BestByStore> {
    let mut storage = Storage::<BestByStore>::new("bestby");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &BestByStore) -> anyhow::Result<()> {
    let storage = Storage::<BestByStore>::new("bestby");
    storage.save_data(store)
}


pub fn add_entry(store: &mut BestByStore, entry: crate::models::Entity) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut BestByStore, name: &str) -> Option<crate::models::Entity> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a BestByStore, name: &str) -> Option<&'a crate::models::Entity> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut BestByStore, name: &str) -> Option<&'a mut crate::models::Entity> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a BestByStore, tag: Option<&'a str>) -> Vec<&'a crate::models::Entity> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
