use crate::models::{AssetType, Investment, InvestmentStore};


i_rs_core::create_store!(InvestmentStore, "invest");


pub fn add_entry(store: &mut InvestmentStore, investment: Investment) {
    store.investments.insert(investment.name.clone(), investment);
}

pub fn remove_entry(store: &mut InvestmentStore, name: &str) -> Option<Investment> {
    store.investments.remove(name)
}

pub fn get_entry<'a>(store: &'a InvestmentStore, name: &str) -> Option<&'a Investment> {
    store.investments.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut InvestmentStore, name: &str) -> Option<&'a mut Investment> {
    store.investments.get_mut(name)
}

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
