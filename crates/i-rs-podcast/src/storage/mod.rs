use crate::models::PodcastStore;


i_rs_core::create_store!(PodcastStore, "podcast");


pub fn add_podcast(store: &mut PodcastStore, podcast: crate::models::Podcast) {
    store.add_podcast(podcast);
}

pub fn remove_podcast(store: &mut PodcastStore, name: &str) -> Option<crate::models::Podcast> {
    store.remove_podcast(name)
}
