use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gift {
    pub name: String,
    pub gift_type: GiftType,
    pub recipient: String,
    pub occasion: String,
    pub value: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GiftType {
    Sent,
    Received,
}

impl std::fmt::Display for GiftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sent => write!(f, "sent"),
            Self::Received => write!(f, "received"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct GiftStore {
    pub gifts: std::collections::BTreeMap<String, Gift>,
}


#[derive(Tabled)]
pub struct GiftRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "TYPE")]
    gift_type: String,
    #[tabled(rename = "RECIPIENT")]
    recipient: String,
    #[tabled(rename = "OCCASION")]
    occasion: String,
    #[tabled(rename = "VALUE")]
    value: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "REMARK")]
    remark: String,
}

#[allow(dead_code)]
impl GiftStore {
    pub fn add_entry(&mut self, entry: Gift) {
        self.gifts.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<Gift> {
        self.gifts.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&Gift> {
        self.gifts.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut Gift> {
        self.gifts.get_mut(key)
    }
}

impl GiftRow {
    pub fn from_gift(gift: &Gift) -> Self {
        Self {
            name: gift.name.clone(),
            gift_type: gift.gift_type.to_string(),
            recipient: gift.recipient.clone(),
            occasion: gift.occasion.clone(),
            value: format!("{:.2}", gift.value),
            date: gift.date.format("%Y-%m-%d").to_string(),
            tags: if gift.tags.is_empty() {
                "-".to_string()
            } else {
                gift.tags.join(", ")
            },
            remark: if gift.remark.is_empty() {
                "-".to_string()
            } else {
                gift.remark.join(", ")
            },
        }
    }
}
