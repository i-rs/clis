use crate::models::BestByStore;


i_rs_core::create_store!(BestByStore, "bestby");


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
