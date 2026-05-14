use crate::models::{AssetType, Investment, InvestmentStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<InvestmentStore> {
    let mut storage = Storage::<InvestmentStore>::new("invest");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &InvestmentStore) -> anyhow::Result<()> {
    let storage = Storage::<InvestmentStore>::new("invest");
    storage.save_data(store)
}


pub fn add_investment(store: &mut InvestmentStore, investment: Investment) {
    store.investments.insert(investment.name.clone(), investment);
}

pub fn remove_investment(store: &mut InvestmentStore, name: &str) -> Option<Investment> {
    store.investments.remove(name)
}

#[allow(dead_code)]
pub fn get_investment<'a>(store: &'a InvestmentStore, name: &str) -> Option<&'a Investment> {
    store.investments.get(name)
}

pub fn get_investment_mut<'a>(store: &'a mut InvestmentStore, name: &str) -> Option<&'a mut Investment> {
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
