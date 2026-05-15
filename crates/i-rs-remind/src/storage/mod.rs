use crate::models::{Remind, RemindStore};

i_rs_core::create_store!(RemindStore, "remind");

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
