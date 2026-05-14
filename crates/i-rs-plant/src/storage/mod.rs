use crate::models::{Plant, PlantStore};


i_rs_core::create_store!(PlantStore, "plant");


pub fn add_plant(store: &mut PlantStore, plant: Plant) {
    store.plants.push(plant);
}

pub fn find_plant<'a>(store: &'a PlantStore, name: &str) -> Option<&'a Plant> {
    store.plants.iter().find(|p| p.name == name)
}

pub fn find_plant_mut<'a>(store: &'a mut PlantStore, name: &str) -> Option<&'a mut Plant> {
    store.plants.iter_mut().find(|p| p.name == name)
}

pub fn remove_plant(store: &mut PlantStore, name: &str) -> bool {
    let len_before = store.plants.len();
    store.plants.retain(|p| p.name != name);
    store.plants.len() < len_before
}
