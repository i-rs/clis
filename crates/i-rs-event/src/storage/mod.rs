use crate::models::{Event, EventStore};


i_rs_core::create_store!(EventStore, "event");

pub fn add_entry(store: &mut EventStore, event: Event) {
    store.events.insert(event.name.clone(), event);
}

pub fn get_entry<'a>(store: &'a EventStore, name: &str) -> Option<&'a Event> {
    store.events.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut EventStore, name: &str) -> Option<&'a mut Event> {
    store.events.get_mut(name)
}

pub fn remove_entry(store: &mut EventStore, name: &str) -> bool {
    store.events.remove(name).is_some()
}

pub fn filter_by_tag<'a>(store: &'a EventStore, tag: &str) -> Vec<&'a Event> {
    store
        .events
        .values()
        .filter(|e| e.tags.iter().any(|t| t == tag))
        .collect()
}

pub fn filter_by_type<'a>(store: &'a EventStore, event_type: &str) -> Vec<&'a Event> {
    store
        .events
        .values()
        .filter(|e| e.event_type.to_string() == event_type)
        .collect()
}
