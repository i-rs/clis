use crate::models::BestByStore;

i_rs_core::create_store!(BestByStore, "bestby");

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
