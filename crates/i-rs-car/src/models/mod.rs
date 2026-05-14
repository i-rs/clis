use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Car {
    pub name: String,
    pub license_plate: String,
    pub brand: String,
    pub model: String,
    pub mileage: f64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Car {
    pub fn new(name: String, license_plate: String, brand: String, model: String, mileage: f64) -> Self {
        let now = Utc::now();
        Self {
            name,
            license_plate,
            brand,
            model,
            mileage,
            tags: Vec::new(),
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelRecord {
    pub id: String,
    pub car_name: String,
    pub date: NaiveDate,
    pub mileage: f64,
    pub fuel_amount: f64,
    pub price_per_liter: f64,
    pub total_cost: f64,
    pub fuel_type: Option<String>,
    #[serde(default)]
    pub station: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl FuelRecord {
    pub fn new(
        car_name: String,
        date: NaiveDate,
        mileage: f64,
        fuel_amount: f64,
        price_per_liter: f64,
        fuel_type: Option<String>,
        station: Option<String>,
        note: Option<String>,
    ) -> Self {
        let total_cost = fuel_amount * price_per_liter;
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            car_name,
            date,
            mileage,
            fuel_amount,
            price_per_liter,
            total_cost,
            fuel_type,
            station,
            note,
            created_at: Utc::now(),
        }
    }

    pub fn fuel_efficiency(&self, prev_mileage: f64) -> Option<f64> {
        let distance = self.mileage - prev_mileage;
        if distance > 0.0 && self.fuel_amount > 0.0 {
            Some(distance / self.fuel_amount)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRecord {
    pub id: String,
    pub car_name: String,
    pub date: NaiveDate,
    pub mileage: f64,
    pub maintenance_type: String,
    pub cost: f64,
    pub description: Option<String>,
    #[serde(default)]
    pub shop: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl MaintenanceRecord {
    pub fn new(
        car_name: String,
        date: NaiveDate,
        mileage: f64,
        maintenance_type: String,
        cost: f64,
        description: Option<String>,
        shop: Option<String>,
        note: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            car_name,
            date,
            mileage,
            maintenance_type,
            cost,
            description,
            shop,
            note,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Store {
    #[serde(default)]
    pub cars: HashMap<String, Car>,
    #[serde(default)]
    pub fuel_records: Vec<FuelRecord>,
    #[serde(default)]
    pub maintenance_records: Vec<MaintenanceRecord>,
}

impl Store {
    pub fn add_car(&mut self, car: Car) {
        self.cars.insert(car.name.clone(), car);
    }

    pub fn get_car(&self, name: &str) -> Option<&Car> {
        self.cars.get(name)
    }

    pub fn get_car_mut(&mut self, name: &str) -> Option<&mut Car> {
        self.cars.get_mut(name)
    }

    pub fn delete_car(&mut self, name: &str) -> bool {
        self.cars.remove(name).is_some()
    }

    pub fn list_cars(&self) -> Vec<&Car> {
        self.cars.values().collect()
    }

    pub fn add_fuel_record(&mut self, record: FuelRecord) {
        let mileage = record.mileage;
        let car_name = record.car_name.clone();
        self.fuel_records.push(record);
        if let Some(car) = self.cars.get_mut(&car_name) {
            if mileage > car.mileage {
                car.mileage = mileage;
                car.updated_at = Utc::now();
            }
        }
    }

    pub fn get_fuel_records(&self, car_name: Option<&str>) -> Vec<&FuelRecord> {
        match car_name {
            Some(name) => self.fuel_records.iter().filter(|r| r.car_name == name).collect(),
            None => self.fuel_records.iter().collect(),
        }
    }

    pub fn delete_fuel_record(&mut self, id: &str) -> bool {
        let len_before = self.fuel_records.len();
        self.fuel_records.retain(|r| r.id != id);
        self.fuel_records.len() < len_before
    }

    pub fn add_maintenance_record(&mut self, record: MaintenanceRecord) {
        let mileage = record.mileage;
        let car_name = record.car_name.clone();
        self.maintenance_records.push(record);
        if let Some(car) = self.cars.get_mut(&car_name) {
            if mileage > car.mileage {
                car.mileage = mileage;
                car.updated_at = Utc::now();
            }
        }
    }

    pub fn get_maintenance_records(&self, car_name: Option<&str>) -> Vec<&MaintenanceRecord> {
        match car_name {
            Some(name) => self.maintenance_records.iter().filter(|r| r.car_name == name).collect(),
            None => self.maintenance_records.iter().collect(),
        }
    }

    pub fn delete_maintenance_record(&mut self, id: &str) -> bool {
        let len_before = self.maintenance_records.len();
        self.maintenance_records.retain(|r| r.id != id);
        self.maintenance_records.len() < len_before
    }

    pub fn get_last_fuel_record(&self, car_name: &str) -> Option<&FuelRecord> {
        self.fuel_records
            .iter()
            .filter(|r| r.car_name == car_name)
            .max_by_key(|r| r.date)
    }

    pub fn total_fuel_cost(&self, car_name: Option<&str>) -> f64 {
        self.get_fuel_records(car_name)
            .iter()
            .map(|r| r.total_cost)
            .sum()
    }

    pub fn total_maintenance_cost(&self, car_name: Option<&str>) -> f64 {
        self.get_maintenance_records(car_name)
            .iter()
            .map(|r| r.cost)
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct CarRow {
    pub name: String,
    pub license_plate: String,
    pub brand: String,
    pub model: String,
    pub mileage: String,
    pub tags: String,
}

impl From<&Car> for CarRow {
    fn from(car: &Car) -> Self {
        Self {
            name: car.name.clone(),
            license_plate: car.license_plate.clone(),
            brand: car.brand.clone(),
            model: car.model.clone(),
            mileage: format!("{:.0} km", car.mileage),
            tags: if car.tags.is_empty() {
                "-".to_string()
            } else {
                car.tags.join(", ")
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct FuelRow {
    pub date: String,
    pub car: String,
    pub mileage: String,
    pub fuel: String,
    pub price: String,
    pub cost: String,
    pub efficiency: String,
    pub station: String,
}

impl FuelRow {
    pub fn from_record(record: &FuelRecord, prev_mileage: Option<f64>) -> Self {
        let efficiency = if let Some(prev) = prev_mileage {
            if let Some(eff) = record.fuel_efficiency(prev) {
                format!("{:.1} km/L", eff)
            } else {
                "-".to_string()
            }
        } else {
            "-".to_string()
        };

        Self {
            date: record.date.format("%Y-%m-%d").to_string(),
            car: record.car_name.clone(),
            mileage: format!("{:.0}", record.mileage),
            fuel: format!("{:.1} L", record.fuel_amount),
            price: format!("{:.2}", record.price_per_liter),
            cost: format!("{:.2}", record.total_cost),
            efficiency,
            station: record.station.clone().unwrap_or_else(|| "-".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct MaintenanceRow {
    pub date: String,
    pub car: String,
    pub mileage: String,
    pub type_: String,
    pub cost: String,
    pub shop: String,
}

impl MaintenanceRow {
    pub fn from_record(record: &MaintenanceRecord) -> Self {
        Self {
            date: record.date.format("%Y-%m-%d").to_string(),
            car: record.car_name.clone(),
            mileage: format!("{:.0}", record.mileage),
            type_: record.maintenance_type.clone(),
            cost: format!("{:.2}", record.cost),
            shop: record.shop.clone().unwrap_or_else(|| "-".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarDetail {
    pub name: String,
    pub license_plate: String,
    pub brand: String,
    pub model: String,
    pub mileage: f64,
    pub fuel_count: usize,
    pub maintenance_count: usize,
    pub total_fuel_cost: f64,
    pub total_maintenance_cost: f64,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Car> for CarDetail {
    fn from(car: &Car) -> Self {
        Self {
            name: car.name.clone(),
            license_plate: car.license_plate.clone(),
            brand: car.brand.clone(),
            model: car.model.clone(),
            mileage: car.mileage,
            fuel_count: 0,
            maintenance_count: 0,
            total_fuel_cost: 0.0,
            total_maintenance_cost: 0.0,
            tags: car.tags.clone(),
            remark: car.remark.clone(),
            created_at: car.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: car.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_cars: usize,
    pub total_fuel_records: usize,
    pub total_maintenance_records: usize,
    pub total_fuel_cost: f64,
    pub total_maintenance_cost: f64,
    pub by_car: HashMap<String, CarStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarStats {
    pub fuel_count: usize,
    pub maintenance_count: usize,
    pub total_fuel_cost: f64,
    pub total_maintenance_cost: f64,
    pub latest_mileage: f64,
}
