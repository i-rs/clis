use crate::models::{Gift, GiftStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<GiftStore> {
    let mut storage = Storage::<GiftStore>::new("gift");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &GiftStore) -> anyhow::Result<()> {
    let storage = Storage::<GiftStore>::new("gift");
    storage.save_data(store)
}


pub fn add_gift(store: &mut GiftStore, gift: Gift) {
    store.gifts.insert(gift.name.clone(), gift);
}

pub fn remove_gift(store: &mut GiftStore, name: &str) -> Option<Gift> {
    store.gifts.remove(name)
}

pub fn get_gift<'a>(store: &'a GiftStore, name: &str) -> Option<&'a Gift> {
    store.gifts.get(name)
}

#[allow(dead_code)]
pub fn get_gift_mut<'a>(store: &'a mut GiftStore, name: &str) -> Option<&'a mut Gift> {
    store.gifts.get_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a GiftStore, tag: Option<&str>) -> Vec<&'a Gift> {
    if let Some(tag) = tag {
        store
            .gifts
            .values()
            .filter(|g| g.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.gifts.values().collect()
    }
}

pub fn filter_by_type<'a>(store: &'a GiftStore, gift_type: Option<&str>) -> Vec<&'a Gift> {
    if let Some(gt) = gift_type {
        store
            .gifts
            .values()
            .filter(|g| g.gift_type.to_string() == gt)
            .collect()
    } else {
        store.gifts.values().collect()
    }
}
