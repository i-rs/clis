use crate::models::{PetbathEntry, PetbathStore};

i_rs_core::create_store!(PetbathStore, "petbath");

pub fn filter_by_tag<'a>(store: &'a PetbathStore, tag: Option<&'a str>) -> Vec<&'a PetbathEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
