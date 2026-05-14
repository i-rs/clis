use crate::models::PodcastStore;


i_rs_core::create_store!(PodcastStore, "podcast");


pub fn add_entry(store: &mut PodcastStore, podcast: crate::models::Podcast) {
    store.add_entry(podcast);
}

pub fn remove_entry(store: &mut PodcastStore, name: &str) -> Option<crate::models::Podcast> {
    store.remove_entry(name)
}
