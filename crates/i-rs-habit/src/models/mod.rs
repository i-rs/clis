use chrono::{DateTime, Utc};
use i_rs_core::storage::HasTags;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub frequency: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(default)]
    pub checkins: Vec<Checkin>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl HasTags for Habit {
    fn tags(&self) -> &[String] {
        &self.tags
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkin {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HabitStore {
    pub entries: BTreeMap<String, Habit>,
}

impl HabitStore {
    pub fn add_entry(&mut self, entry: Habit) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<Habit> {
        self.entries.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&Habit> {
        self.entries.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut Habit> {
        self.entries.get_mut(name)
    }
}

#[derive(Tabled)]
pub struct HabitRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "DESCRIPTION")]
    description: String,
    #[tabled(rename = "FREQUENCY")]
    frequency: String,
    #[tabled(rename = "STREAK")]
    streak: i32,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl HabitRow {
    pub fn from_habit(habit: &Habit) -> Self {
        Self {
            name: habit.name.clone(),
            description: if habit.description.is_empty() {
                "-".to_string()
            } else {
                habit.description.clone()
            },
            frequency: habit.frequency.clone(),
            streak: Self::calculate_streak(habit),
            tags: if habit.tags.is_empty() {
                "-".to_string()
            } else {
                habit.tags.join(", ")
            },
            updated_at: habit.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }

    fn calculate_streak(habit: &Habit) -> i32 {
        if habit.checkins.is_empty() {
            return 0;
        }

        let mut checkin_dates: Vec<DateTime<Utc>> = habit
            .checkins
            .iter()
            .map(|c| {
                c.date
                    .date_naive()
                    .and_hms_opt(12, 0, 0)
                    .expect("12:00:00 is always valid")
                    .and_local_timezone(chrono::Local)
                    .single()
                    .expect("12:00 is DST-safe")
                    .with_timezone(&Utc)
            })
            .collect();
        checkin_dates.sort_by(|a, b| b.cmp(a));

        let today = Utc::now()
            .date_naive()
            .and_hms_opt(12, 0, 0)
            .expect("12:00:00 is always valid")
            .and_local_timezone(chrono::Local)
            .single()
            .expect("12:00 is DST-safe")
            .with_timezone(&Utc);
        let mut streak = 0;
        let mut current_day = today;

        for checkin in checkin_dates {
            let diff_days = (current_day - checkin).num_days().abs();
            if diff_days <= 1 {
                streak += 1;
                current_day = checkin - chrono::Duration::days(1);
            } else {
                break;
            }
        }

        streak
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub name: String,
    pub description: String,
    pub frequency: String,
    pub streak: i32,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub checkin_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Habit> for ListItem {
    fn from(habit: &Habit) -> Self {
        let mut checkin_dates: Vec<DateTime<Utc>> = habit
            .checkins
            .iter()
            .map(|c| {
                c.date
                    .date_naive()
                    .and_hms_opt(12, 0, 0)
                    .expect("12:00:00 is always valid")
                    .and_local_timezone(chrono::Local)
                    .single()
                    .expect("12:00 is DST-safe")
                    .with_timezone(&Utc)
            })
            .collect();
        checkin_dates.sort_by(|a, b| b.cmp(a));

        let today = Utc::now()
            .date_naive()
            .and_hms_opt(12, 0, 0)
            .expect("12:00:00 is always valid")
            .and_local_timezone(chrono::Local)
            .single()
            .expect("12:00 is DST-safe")
            .with_timezone(&Utc);
        let mut streak = 0;
        let mut current_day = today;

        for checkin in checkin_dates {
            let diff_days = (current_day - checkin).num_days().abs();
            if diff_days <= 1 {
                streak += 1;
                current_day = checkin - chrono::Duration::days(1);
            } else {
                break;
            }
        }

        Self {
            name: habit.name.clone(),
            description: habit.description.clone(),
            frequency: habit.frequency.clone(),
            streak,
            tags: habit.tags.clone(),
            remark: habit.remark.clone(),
            checkin_count: habit.checkins.len(),
            created_at: habit.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: habit.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
