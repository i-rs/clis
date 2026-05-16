use crate::models::{Event, EventStore};

i_rs_core::create_store!(EventStore, "event");

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
