use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub name: String,
    pub amount: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: DateTime<Utc>,
    pub invoice_type: InvoiceType,
    #[serde(default)]
    pub reimbursed: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceType {
    Electronic,
    Paper,
}

impl std::fmt::Display for InvoiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Electronic => write!(f, "electronic"),
            Self::Paper => write!(f, "paper"),
        }
    }
}

impl std::str::FromStr for InvoiceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "electronic" | "e" => Ok(Self::Electronic),
            "paper" | "p" => Ok(Self::Paper),
            _ => Err(format!(
                "Invalid invoice type: {s}. Use 'electronic' or 'paper'"
            )),
        }
    }
}

#[derive(Tabled)]
pub struct InvoiceRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Amount")]
    pub amount: String,
    #[tabled(rename = "Date")]
    pub date: String,
    #[tabled(rename = "Type")]
    pub invoice_type: String,
    #[tabled(rename = "Reimbursed")]
    pub reimbursed: String,
    #[tabled(rename = "Tags")]
    pub tags: String,
}

impl InvoiceRow {
    pub fn from_invoice(invoice: &Invoice) -> Self {
        Self {
            name: invoice.name.clone(),
            amount: format!("{:.2}", invoice.amount),
            date: invoice.date.format("%Y-%m-%d").to_string(),
            invoice_type: invoice.invoice_type.to_string(),
            reimbursed: if invoice.reimbursed { "✓" } else { "✗" }.to_string(),
            tags: invoice.tags.join(", "),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InvoiceStore {
    pub entries: std::collections::BTreeMap<String, Invoice>,
}

impl InvoiceStore {
    pub fn add_entry(&mut self, entry: Invoice) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<Invoice> {
        self.entries.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&Invoice> {
        self.entries.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut Invoice> {
        self.entries.get_mut(key)
    }
}

pub fn filter_by_tag<'a>(store: &'a InvoiceStore, tag: &str) -> Vec<&'a Invoice> {
    store
        .entries
        .values()
        .filter(|inv| inv.tags.iter().any(|t| t == tag))
        .collect()
}

pub fn parse_date(date_str: &str) -> anyhow::Result<DateTime<Utc>> {
    i_rs_core::parse_datetime(date_str)
}
