use crate::models::ProjectStore;

i_rs_core::create_store!(ProjectStore, "project");

pub fn project_exists(store: &ProjectStore, name: &str) -> bool {
    store.get_entry(name).is_some()
}
