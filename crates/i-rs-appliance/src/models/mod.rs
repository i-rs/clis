use chrono::{DateTime, Duration, NaiveDate, Utc};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRecord {
    pub date: NaiveDate,
    pub description: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appliance {
    pub id: String,
    pub name: String,
    pub brand: String,
    pub model: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub purchase_date: DateTime<Utc>,
    pub lifespan_years: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(default)]
    pub maintenance_records: Vec<MaintenanceRecord>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Appliance {
    pub fn expiry_date(&self) -> NaiveDate {
        self.purchase_date.date_naive() + Duration::days(365 * i64::from(self.lifespan_years))
    }

    pub fn days_until_expiry(&self) -> i64 {
        (self.expiry_date() - Utc::now().date_naive()).num_days()
    }

    pub fn is_expired(&self) -> bool {
        self.days_until_expiry() < 0
    }

    pub fn needs_replacement_soon(&self) -> bool {
        let days = self.days_until_expiry();
        (0..=90).contains(&days)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ApplianceStore {
    pub appliances: BTreeMap<String, Appliance>,
}


#[allow(dead_code)]
impl ApplianceStore {
    pub fn add_entry(&mut self, appliance: Appliance) {
        self.appliances.insert(appliance.id.clone(), appliance);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<Appliance> {
        self.appliances.remove(id)
    }

    pub fn remove_appliance_by_name(&mut self, name: &str) -> Option<Appliance> {
        let key = self.appliances.iter().find(|(_, a)| a.name.eq_ignore_ascii_case(name)).map(|(k, _)| k.clone());
        key.and_then(|k| self.appliances.remove(&k))
    }

    pub fn get_entry(&self, id: &str) -> Option<&Appliance> {
        self.appliances.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut Appliance> {
        self.appliances.get_mut(id)
    }

    pub fn get_all_appliances(&self) -> Vec<&Appliance> {
        self.appliances.values().collect()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Appliance> {
        self.appliances.values().find(|a| a.name.eq_ignore_ascii_case(name))
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Appliance> {
        self.appliances.values_mut().find(|a| a.name.eq_ignore_ascii_case(name))
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&Appliance> {
        self.appliances
            .values()
            .filter(|a| a.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }

    pub fn appliances_count(&self) -> usize {
        self.appliances.len()
    }

    pub fn expired_count(&self) -> usize {
        self.appliances.values().filter(|a| a.is_expired()).count()
    }

    pub fn needs_replacement_count(&self) -> usize {
        self.appliances.values().filter(|a| a.needs_replacement_soon()).count()
    }
}

#[derive(Tabled)]
pub struct ApplianceRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "BRAND")]
    brand: String,
    #[tabled(rename = "MODEL")]
    model: String,
    #[tabled(rename = "EXPIRY")]
    expiry: String,
    #[tabled(rename = "DAYS")]
    days: String,
    #[tabled(rename = "STATUS")]
    status: String,
}

impl ApplianceRow {
    pub fn from_appliance(appliance: &Appliance) -> Self {
        let days = appliance.days_until_expiry();
        let status = if appliance.is_expired() {
            format!("{}", "EXPIRED".red())
        } else if appliance.needs_replacement_soon() {
            format!("{}", "SOON".yellow())
        } else {
            format!("{}", "OK".green())
        };

        Self {
            name: appliance.name.clone(),
            brand: appliance.brand.clone(),
            model: appliance.model.clone(),
            expiry: appliance.expiry_date().format("%Y-%m-%d").to_string(),
            days: days.to_string(),
            status,
        }
    }
}

#[derive(Tabled)]
#[allow(dead_code)]
pub struct MaintenanceRow {
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "DESCRIPTION")]
    description: String,
}

impl MaintenanceRow {
    #[allow(dead_code)]
pub fn from_record(record: &MaintenanceRecord) -> Self {
        Self {
            date: record.date.format("%Y-%m-%d").to_string(),
            description: record.description.clone(),
        }
    }
}
