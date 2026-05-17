use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub date: NaiveDate,
    pub distance_km: f64,
    pub duration_minutes: f64,
    pub pace: String,
    pub heart_rate: Option<u32>,
    pub weather: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunPlan {
    pub id: String,
    pub name: String,
    pub target_distance_km: f64,
    pub target_pace: String,
    pub schedule_days: Vec<u8>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunStore {
    pub records: BTreeMap<String, RunRecord>,
    pub plans: BTreeMap<String, RunPlan>,
}

#[allow(dead_code)]
impl RunStore {
    pub fn add_entry(&mut self, record: RunRecord) {
        self.records.insert(record.id.clone(), record);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<RunRecord> {
        self.records.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&RunRecord> {
        self.records.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut RunRecord> {
        self.records.get_mut(id)
    }

    pub fn get_all_records(&self) -> Vec<&RunRecord> {
        self.records.values().collect()
    }

    pub fn add_plan(&mut self, plan: RunPlan) {
        self.plans.insert(plan.id.clone(), plan);
    }

    pub fn remove_plan(&mut self, id: &str) -> Option<RunPlan> {
        self.plans.remove(id)
    }

    pub fn get_plan(&self, id: &str) -> Option<&RunPlan> {
        self.plans.get(id)
    }

    pub fn get_all_plans(&self) -> Vec<&RunPlan> {
        self.plans.values().collect()
    }

    pub fn total_distance(&self) -> f64 {
        self.records.values().map(|r| r.distance_km).sum()
    }

    pub fn total_duration(&self) -> f64 {
        self.records.values().map(|r| r.duration_minutes).sum()
    }

    pub fn avg_pace(&self) -> Option<String> {
        if self.records.is_empty() {
            return None;
        }
        let total_dist = self.total_distance();
        let total_time = self.total_duration();
        if total_dist == 0.0 {
            return None;
        }
        let pace_min_per_km = total_time / total_dist;
        let pace_min = pace_min_per_km as u32;
        let pace_sec = ((pace_min_per_km - f64::from(pace_min)) * 60.0) as u32;
        Some(format!("{pace_min}:{pace_sec:02}"))
    }

    pub fn records_count(&self) -> usize {
        self.records.len()
    }

    pub fn plans_count(&self) -> usize {
        self.plans.len()
    }
}

#[derive(Tabled)]
pub struct RunRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "DISTANCE")]
    distance: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "PACE")]
    pace: String,
    #[tabled(rename = "HR")]
    heart_rate: String,
    #[tabled(rename = "WEATHER")]
    weather: String,
}

impl RunRow {
    pub fn from_record(record: &RunRecord) -> Self {
        Self {
            id: record.id[..8].to_string(),
            date: record.date.format("%Y-%m-%d").to_string(),
            distance: format!("{:.2}", record.distance_km),
            duration: format_duration(record.duration_minutes),
            pace: record.pace.clone(),
            heart_rate: record
                .heart_rate
                .map_or_else(|| "-".to_string(), |hr| hr.to_string()),
            weather: record.weather.clone().unwrap_or_else(|| "-".to_string()),
        }
    }
}

#[derive(Tabled)]
pub struct PlanRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "TARGET KM")]
    target: String,
    #[tabled(rename = "PACE")]
    pace: String,
    #[tabled(rename = "SCHEDULE")]
    schedule: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl PlanRow {
    pub fn from_plan(plan: &RunPlan) -> Self {
        Self {
            id: plan.id[..8].to_string(),
            name: plan.name.clone(),
            target: format!("{:.2}", plan.target_distance_km),
            pace: plan.target_pace.clone(),
            schedule: format_schedule(&plan.schedule_days),
            tags: if plan.tags.is_empty() {
                "-".to_string()
            } else {
                plan.tags.join(", ")
            },
        }
    }
}

pub fn format_duration(minutes: f64) -> String {
    let hours = (minutes / 60.0) as u32;
    let mins = (minutes % 60.0) as u32;
    let secs = ((minutes * 60.0) % 60.0) as u32;
    if hours > 0 {
        format!("{hours}:{mins:02}:{secs:02}")
    } else {
        format!("{mins}:{secs:02}")
    }
}

pub fn format_pace(distance_km: f64, duration_minutes: f64) -> String {
    if distance_km == 0.0 {
        return "0:00".to_string();
    }
    let pace_min_per_km = duration_minutes / distance_km;
    let pace_min = pace_min_per_km as u32;
    let pace_sec = ((pace_min_per_km - f64::from(pace_min)) * 60.0) as u32;
    format!("{pace_min}:{pace_sec:02}")
}

fn format_schedule(days: &[u8]) -> String {
    if days.is_empty() {
        return "-".to_string();
    }
    let day_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let names: Vec<String> = days
        .iter()
        .filter_map(|&d| {
            if d < 7 {
                Some(day_names[d as usize].to_string())
            } else {
                None
            }
        })
        .collect();
    names.join(", ")
}
