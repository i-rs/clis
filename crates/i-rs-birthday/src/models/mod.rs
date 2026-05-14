use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Birthday {
    pub name: String,
    pub birth_date: String,
    pub year: Option<i32>,
    pub relationship: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Birthday {
    pub fn days_until_birthday(&self) -> i64 {
        let today = Local::now().date_naive();
        let (month, day) = self.parse_birth_date();
        
        let this_year_birthday = NaiveDate::from_ymd_opt(today.year(), month, day)
            .unwrap_or_else(|| today);
        
        if this_year_birthday >= today {
            (this_year_birthday - today).num_days()
        } else {
            let next_year_birthday = NaiveDate::from_ymd_opt(today.year() + 1, month, day)
                .unwrap_or_else(|| this_year_birthday);
            (next_year_birthday - today).num_days()
        }
    }

    pub fn is_today(&self) -> bool {
        self.days_until_birthday() == 0
    }

    pub fn is_upcoming(&self, days: i64) -> bool {
        let days_left = self.days_until_birthday();
        days_left > 0 && days_left <= days
    }

    pub fn age(&self) -> Option<i32> {
        self.year.map(|birth_year| {
            let today = Local::now().date_naive();
            let mut age = today.year() - birth_year;
            let (month, day) = self.parse_birth_date();
            let birth_date_this_year = NaiveDate::from_ymd_opt(today.year(), month, day)
                .unwrap_or_else(|| today);
            if today < birth_date_this_year {
                age -= 1;
            }
            age
        })
    }

    fn parse_birth_date(&self) -> (u32, u32) {
        let parts: Vec<&str> = self.birth_date.split('-').collect();
        if parts.len() == 2 {
            let month: u32 = parts[0].parse().unwrap_or(1);
            let day: u32 = parts[1].parse().unwrap_or(1);
            (month, day)
        } else {
            (1, 1)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthdayStore {
    pub birthdays: std::collections::HashMap<String, Birthday>,
}

impl Default for BirthdayStore {
    fn default() -> Self {
        Self {
            birthdays: std::collections::HashMap::new(),
        }
    }
}

#[derive(Tabled)]
pub struct BirthdayRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "BIRTHDAY")]
    birth_date: String,
    #[tabled(rename = "AGE")]
    age: String,
    #[tabled(rename = "DAYS_LEFT")]
    days_left: String,
    #[tabled(rename = "RELATIONSHIP")]
    relationship: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl BirthdayRow {
    pub fn from_birthday(birthday: &Birthday) -> Self {
        let days = birthday.days_until_birthday();
        let days_str = if birthday.is_today() {
            "TODAY!".red().bold().to_string()
        } else if days <= 7 {
            format!("{} days", days).yellow().to_string()
        } else if days <= 30 {
            format!("{} days", days).cyan().to_string()
        } else {
            format!("{} days", days)
        };

        let age_str = birthday.age()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "-".to_string());

        Self {
            name: birthday.name.clone(),
            birth_date: birthday.birth_date.clone(),
            age: age_str,
            days_left: days_str,
            relationship: birthday.relationship.clone(),
            tags: if birthday.tags.is_empty() {
                "-".to_string()
            } else {
                birthday.tags.join(", ")
            },
        }
    }
}
