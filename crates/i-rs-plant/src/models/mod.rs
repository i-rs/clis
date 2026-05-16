use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plant {
    pub name: String,
    pub species: String,
    pub location: String,
    pub watering_interval_days: u32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_watered: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Plant {
    pub fn new(
        name: String,
        species: String,
        location: String,
        watering_interval_days: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            name,
            species,
            location,
            watering_interval_days,
            last_watered: now,
            tags: Vec::new(),
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn days_until_next_watering(&self) -> i64 {
        let next_watering =
            self.last_watered + chrono::Duration::days(i64::from(self.watering_interval_days));
        (next_watering - Utc::now()).num_days()
    }

    pub fn needs_water(&self) -> bool {
        self.days_until_next_watering() <= 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlantStore {
    pub plants: Vec<Plant>,
}

#[derive(Debug, Clone, Tabled)]
pub struct PlantRow {
    pub name: String,
    pub species: String,
    pub location: String,
    pub interval: String,
    pub last_watered: String,
    pub days_until: String,
}

impl PlantRow {
    pub fn from_plant(plant: &Plant) -> Self {
        let days = plant.days_until_next_watering();
        let days_str = if days <= 0 {
            format!("{} days overdue", -days)
        } else {
            format!("{days} days")
        };

        Self {
            name: plant.name.clone(),
            species: plant.species.clone(),
            location: plant.location.clone(),
            interval: format!("{} days", plant.watering_interval_days),
            last_watered: plant.last_watered.format("%Y-%m-%d").to_string(),
            days_until: days_str,
        }
    }
}

impl PlantStore {
    pub fn add_entry(&mut self, entry: Plant) {
        self.plants.push(entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<Plant> {
        let idx = self.plants.iter().position(|p| p.name == name)?;
        Some(self.plants.remove(idx))
    }

    pub fn get_entry(&self, name: &str) -> Option<&Plant> {
        self.plants.iter().find(|p| p.name == name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut Plant> {
        self.plants.iter_mut().find(|p| p.name == name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantStats {
    pub total_plants: usize,
    pub needs_water: usize,
    pub healthy: usize,
    pub total_waterings: usize,
}
