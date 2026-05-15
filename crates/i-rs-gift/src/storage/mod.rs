use crate::models::{Gift, GiftStore};


i_rs_core::create_store!(GiftStore, "gift");


pub fn add_entry(store: &mut GiftStore, gift: Gift) {
    store.gifts.insert(gift.name.clone(), gift);
}

pub fn remove_entry(store: &mut GiftStore, name: &str) -> Option<Gift> {
    store.gifts.remove(name)
}

pub fn get_entry<'a>(store: &'a GiftStore, name: &str) -> Option<&'a Gift> {
    store.gifts.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut GiftStore, name: &str) -> Option<&'a mut Gift> {
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
