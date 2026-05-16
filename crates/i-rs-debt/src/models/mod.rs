use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DebtType {
    CreditCard,
    Loan,
    Borrowed,
}

impl std::fmt::Display for DebtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreditCard => write!(f, "credit_card"),
            Self::Loan => write!(f, "loan"),
            Self::Borrowed => write!(f, "borrowed"),
        }
    }
}

impl std::str::FromStr for DebtType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "credit_card" | "creditcard" | "cc" => Ok(Self::CreditCard),
            "loan" | "l" => Ok(Self::Loan),
            "borrowed" | "b" => Ok(Self::Borrowed),
            _ => Err(format!("Invalid debt type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub amount: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub paid_at: DateTime<Utc>,
    pub note: Option<String>,
}

impl Payment {
    pub fn new(amount: f64, note: Option<String>) -> Self {
        Self {
            amount,
            paid_at: Utc::now(),
            note,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Debt {
    pub name: String,
    pub debt_type: DebtType,
    pub total_amount: f64,
    pub remaining: f64,
    pub interest_rate: Option<f64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub payments: Vec<Payment>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Debt {
    pub fn new(name: String, debt_type: DebtType, total_amount: f64) -> Self {
        let now = Utc::now();
        Self {
            name,
            debt_type,
            total_amount,
            remaining: total_amount,
            interest_rate: None,
            due_date: None,
            payments: Vec::new(),
            tags: Vec::new(),
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_payment(&mut self, amount: f64, note: Option<String>) {
        let payment = Payment::new(amount, note);
        self.remaining = (self.remaining - amount).max(0.0);
        self.updated_at = Utc::now();
        self.payments.push(payment);
    }

    pub fn paid_amount(&self) -> f64 {
        self.total_amount - self.remaining
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total_amount == 0.0 {
            100.0
        } else {
            (self.paid_amount() / self.total_amount * 100.0).min(100.0)
        }
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            self.remaining > 0.0 && due < Utc::now()
        } else {
            false
        }
    }

    pub fn days_overdue(&self) -> Option<i64> {
        if self.is_overdue()
            && let Some(due) = self.due_date
        {
            return Some((Utc::now() - due).num_days());
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DebtStore {
    #[serde(default)]
    pub debts: std::collections::BTreeMap<String, Debt>,
}

impl DebtStore {
    pub fn add_entry(&mut self, entry: Debt) {
        self.debts.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<Debt> {
        self.debts.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&Debt> {
        self.debts.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut Debt> {
        self.debts.get_mut(key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct DebtRow {
    pub name: String,
    pub debt_type: String,
    pub total: String,
    pub paid: String,
    pub remaining: String,
    pub progress: String,
    pub due_date: String,
    pub tags: String,
}

impl DebtRow {
    pub fn from_debt(debt: &Debt) -> Self {
        use owo_colors::OwoColorize;

        let progress = debt.progress_percentage();
        let progress_str = format!("{progress:.1}%");

        let progress_colored = if debt.is_overdue() {
            progress_str.red().to_string()
        } else if progress >= 100.0 {
            progress_str.green().to_string()
        } else {
            progress_str.yellow().to_string()
        };

        let due_str = if let Some(due) = debt.due_date {
            let days = (due - Utc::now()).num_days();
            if days < 0 {
                format!("{} ({}d)", due.format("%Y-%m-%d"), days.abs())
            } else {
                due.format("%Y-%m-%d").to_string()
            }
        } else {
            "-".to_string()
        };

        Self {
            name: debt.name.clone(),
            debt_type: format!("{:?}", debt.debt_type),
            total: format!("{:.2}", debt.total_amount),
            paid: format!("{:.2}", debt.paid_amount()),
            remaining: format!("{:.2}", debt.remaining),
            progress: progress_colored,
            due_date: due_str,
            tags: if debt.tags.is_empty() {
                "-".to_string()
            } else {
                debt.tags.join(", ")
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtDetail {
    pub name: String,
    pub debt_type: String,
    pub total_amount: f64,
    pub paid_amount: f64,
    pub remaining: f64,
    pub interest_rate: Option<f64>,
    pub due_date: Option<String>,
    pub progress: f64,
    pub is_overdue: bool,
    pub days_overdue: Option<i64>,
    pub payment_count: usize,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Debt> for DebtDetail {
    fn from(debt: &Debt) -> Self {
        Self {
            name: debt.name.clone(),
            debt_type: format!("{:?}", debt.debt_type),
            total_amount: debt.total_amount,
            paid_amount: debt.paid_amount(),
            remaining: debt.remaining,
            interest_rate: debt.interest_rate,
            due_date: debt
                .due_date
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            progress: debt.progress_percentage(),
            is_overdue: debt.is_overdue(),
            days_overdue: debt.days_overdue(),
            payment_count: debt.payments.len(),
            tags: debt.tags.clone(),
            remark: debt.remark.clone(),
            created_at: debt.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: debt.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_debts: usize,
    pub total_amount: f64,
    pub total_paid: f64,
    pub total_remaining: f64,
    pub overdue_count: usize,
    pub by_type: std::collections::BTreeMap<String, TypeStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStats {
    pub count: usize,
    pub total: f64,
    pub paid: f64,
    pub remaining: f64,
}
