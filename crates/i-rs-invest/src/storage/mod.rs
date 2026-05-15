use crate::models::{AssetType, Investment, InvestmentStore};

i_rs_core::create_store!(InvestmentStore, "invest");

pub fn filter_by_type<'a>(store: &'a InvestmentStore, asset_type: Option<&AssetType>) -> Vec<&'a Investment> {
    if let Some(asset_type) = asset_type {
        store
            .investments
            .values()
            .filter(|i| &i.asset_type == asset_type)
            .collect()
    } else {
        store.investments.values().collect()
    }
}

pub fn filter_by_tag<'a>(store: &'a InvestmentStore, tag: Option<&str>) -> Vec<&'a Investment> {
    if let Some(tag) = tag {
        store
            .investments
            .values()
            .filter(|i| i.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.investments.values().collect()
    }
}
