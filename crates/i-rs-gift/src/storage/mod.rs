use crate::models::{Gift, GiftStore};

i_rs_core::create_store!(GiftStore, "gift");

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
