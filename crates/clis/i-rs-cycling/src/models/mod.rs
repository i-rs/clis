use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclingRecord {
    pub id: Uuid,
    pub date: NaiveDate,
    pub distance_km: f64,
    pub duration_minutes: u32,
    pub avg_speed: f64,
    #[serde(default)]
    pub elevation_gain: Option<f64>,
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl CyclingRecord {
    pub fn new(
        date: NaiveDate,
        distance_km: f64,
        duration_minutes: u32,
        elevation_gain: Option<f64>,
        route: Option<String>,
        tags: Vec<String>,
        remark: Vec<String>,
    ) -> Self {
        let avg_speed = if duration_minutes > 0 {
            distance_km / (f64::from(duration_minutes) / 60.0)
        } else {
            0.0
        };

        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            date,
            distance_km,
            duration_minutes,
            avg_speed,
            elevation_gain,
            route,
            tags,
            remark,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn recalc_avg_speed(&mut self) {
        self.avg_speed = if self.duration_minutes > 0 {
            self.distance_km / (f64::from(self.duration_minutes) / 60.0)
        } else {
            0.0
        };
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CyclingStore {
    pub records: BTreeMap<Uuid, CyclingRecord>,
}

#[allow(dead_code)]
impl CyclingStore {
    pub fn add_entry(&mut self, record: CyclingRecord) {
        self.records.insert(record.id, record);
    }

    pub fn remove_entry(&mut self, id: &Uuid) -> Option<CyclingRecord> {
        self.records.remove(id)
    }

    pub fn get_entry(&self, id: &Uuid) -> Option<&CyclingRecord> {
        self.records.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &Uuid) -> Option<&mut CyclingRecord> {
        self.records.get_mut(id)
    }

    pub fn get_all_records(&self) -> Vec<&CyclingRecord> {
        self.records.values().collect()
    }

    pub fn total_distance(&self) -> f64 {
        self.records.values().map(|r| r.distance_km).sum()
    }

    pub fn total_duration(&self) -> u64 {
        self.records
            .values()
            .map(|r| u64::from(r.duration_minutes))
            .sum()
    }

    pub fn total_elevation(&self) -> f64 {
        self.records.values().filter_map(|r| r.elevation_gain).sum()
    }

    pub fn avg_speed_all(&self) -> Option<f64> {
        let total_dist = self.total_distance();
        let total_time = self.total_duration() as f64 / 60.0;
        if total_time > 0.0 {
            Some(total_dist / total_time)
        } else {
            None
        }
    }

    pub fn records_count(&self) -> usize {
        self.records.len()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&CyclingRecord> {
        self.records
            .values()
            .filter(|r| r.tags.iter().any(|t| t == tag))
            .collect()
    }
}

#[derive(Tabled)]
pub struct CyclingRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "DISTANCE")]
    distance: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "AVG SPEED")]
    avg_speed: String,
    #[tabled(rename = "ELEVATION")]
    elevation: String,
    #[tabled(rename = "ROUTE")]
    route: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl CyclingRow {
    pub fn from_record(record: &CyclingRecord) -> Self {
        Self {
            id: record.id.to_string()[..8].to_string(),
            date: record.date.format("%Y-%m-%d").to_string(),
            distance: format!("{:.1} km", record.distance_km),
            duration: format!("{} min", record.duration_minutes),
            avg_speed: format!("{:.1} km/h", record.avg_speed),
            elevation: match record.elevation_gain {
                Some(e) => format!("{e:.0} m"),
                None => "-".to_string(),
            },
            route: record.route.clone().unwrap_or_else(|| "-".to_string()),
            tags: if record.tags.is_empty() {
                "-".to_string()
            } else {
                record.tags.join(", ")
            },
        }
    }
}

#[derive(Tabled)]
pub struct CyclingDetailRow {
    #[tabled(rename = "KEY")]
    key: String,
    #[tabled(rename = "VALUE")]
    value: String,
}

impl CyclingDetailRow {
    pub fn from_record(record: &CyclingRecord) -> Vec<Self> {
        let mut rows = vec![
            Self {
                key: "ID".to_string(),
                value: record.id.to_string(),
            },
            Self {
                key: "Date".to_string(),
                value: record.date.format("%Y-%m-%d").to_string(),
            },
            Self {
                key: "Distance".to_string(),
                value: format!("{:.2} km", record.distance_km),
            },
            Self {
                key: "Duration".to_string(),
                value: format!("{} minutes", record.duration_minutes),
            },
            Self {
                key: "Avg Speed".to_string(),
                value: format!("{:.2} km/h", record.avg_speed),
            },
        ];

        if let Some(elevation) = record.elevation_gain {
            rows.push(Self {
                key: "Elevation Gain".to_string(),
                value: format!("{elevation:.0} m"),
            });
        }

        if let Some(ref route) = record.route {
            rows.push(Self {
                key: "Route".to_string(),
                value: route.clone(),
            });
        }

        if !record.tags.is_empty() {
            rows.push(Self {
                key: "Tags".to_string(),
                value: record.tags.join(", "),
            });
        }

        if !record.remark.is_empty() {
            rows.push(Self {
                key: "Remark".to_string(),
                value: record.remark.join("; "),
            });
        }

        rows.push(Self {
            key: "Created".to_string(),
            value: record.created_at.format("%Y-%m-%d %H:%M").to_string(),
        });
        rows.push(Self {
            key: "Updated".to_string(),
            value: record.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        });

        rows
    }
}
