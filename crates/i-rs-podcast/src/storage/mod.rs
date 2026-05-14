use crate::models::PodcastStore;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<PodcastStore> {
    let mut storage = Storage::<PodcastStore>::new("podcast");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &PodcastStore) -> anyhow::Result<()> {
    let storage = Storage::<PodcastStore>::new("podcast");
    storage.save_data(store)
}


pub fn add_podcast(store: &mut PodcastStore, podcast: crate::models::Podcast) {
    store.add_podcast(podcast);
}

pub fn remove_podcast(store: &mut PodcastStore, name: &str) -> Option<crate::models::Podcast> {
    store.remove_podcast(name)
}
