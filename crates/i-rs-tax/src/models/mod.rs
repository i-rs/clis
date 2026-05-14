use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRecord {
    pub name: String,
    pub tax_type: TaxType,
    pub amount: f64,
    pub date: NaiveDate,
    pub year: i32,
    pub status: TaxStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaxType {
    #[serde(rename = "个人所得税")]
    Personal,
    #[serde(rename = "增值税")]
    Vat,
}

impl std::fmt::Display for TaxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Personal => write!(f, "个人所得税"),
            Self::Vat => write!(f, "增值税"),
        }
    }
}

impl TaxType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "个人所得税" | "personal" => Some(Self::Personal),
            "增值税" | "vat" => Some(Self::Vat),
            _ => None,
        }
    }

    pub fn variants() -> Vec<&'static str> {
        vec!["个人所得税", "增值税"]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaxStatus {
    #[serde(rename = "未申报")]
    Unreported,
    #[serde(rename = "申报中")]
    Filing,
    #[serde(rename = "已申报")]
    Filed,
    #[serde(rename = "已缴纳")]
    Paid,
}

impl std::fmt::Display for TaxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreported => write!(f, "未申报"),
            Self::Filing => write!(f, "申报中"),
            Self::Filed => write!(f, "已申报"),
            Self::Paid => write!(f, "已缴纳"),
        }
    }
}

impl TaxStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "未申报" | "unreported" => Some(Self::Unreported),
            "申报中" | "filing" => Some(Self::Filing),
            "已申报" | "filed" => Some(Self::Filed),
            "已缴纳" | "paid" => Some(Self::Paid),
            _ => None,
        }
    }

    pub fn variants() -> Vec<&'static str> {
        vec!["未申报", "申报中", "已申报", "已缴纳"]
    }
}

#[derive(Tabled, Debug, Clone, Serialize, Deserialize)]
pub struct TaxRecordRow {
    #[tabled(rename = "名称")]
    pub name: String,
    #[tabled(rename = "税种")]
    pub tax_type: String,
    #[tabled(rename = "金额")]
    pub amount: String,
    #[tabled(rename = "日期")]
    pub date: String,
    #[tabled(rename = "年度")]
    pub year: String,
    #[tabled(rename = "状态")]
    pub status: String,
    #[tabled(rename = "标签")]
    pub tags: String,
}

impl TaxRecordRow {
    pub fn from_entity(entity: &TaxRecord) -> Self {
        Self {
            name: entity.name.clone(),
            tax_type: entity.tax_type.to_string(),
            amount: format!("{:.2}", entity.amount),
            date: entity.date.format("%Y-%m-%d").to_string(),
            year: entity.year.to_string(),
            status: entity.status.to_string(),
            tags: entity.tags.join(", "),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct TaxStore {
    pub entries: std::collections::BTreeMap<String, TaxRecord>,
}

