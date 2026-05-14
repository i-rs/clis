use crate::models::{Event, EventStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<EventStore> {
    let mut storage = Storage::<EventStore>::new("event");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &EventStore) -> anyhow::Result<()> {
    let storage = Storage::<EventStore>::new("event");
    storage.save_data(store)
}

pub fn add_event(store: &mut EventStore, event: Event) {
    store.events.insert(event.name.clone(), event);
}

#[allow(dead_code)]
pub fn get_event<'a>(store: &'a EventStore, name: &str) -> Option<&'a Event> {
    store.events.get(name)
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn get_event_mut<'a>(store: &'a mut EventStore, name: &str) -> Option<&'a mut Event> {
    store.events.get_mut(name)
}

pub fn remove_event(store: &mut EventStore, name: &str) -> bool {
    store.events.remove(name).is_some()
}

#[allow(dead_code)]
pub fn filter_by_tag<'a>(store: &'a EventStore, tag: &str) -> Vec<&'a Event> {
    store
        .events
        .values()
        .filter(|e| e.tags.iter().any(|t| t == tag))
        .collect()
}

#[allow(dead_code)]
pub fn filter_by_type<'a>(store: &'a EventStore, event_type: &str) -> Vec<&'a Event> {
    store
        .events
        .values()
        .filter(|e| e.event_type.to_string() == event_type)
        .collect()
}
