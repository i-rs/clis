use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum BudgetPeriod {
    Daily,
    Weekly,
    #[default]
    Monthly,
    Yearly,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub category: String,
    pub amount: f64,
    #[serde(default)]
    pub period: BudgetPeriod,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Budget {
    pub fn new(category: String, amount: f64, period: BudgetPeriod) -> Self {
        let now = Utc::now();
        Self {
            category,
            amount,
            period,
            tags: Vec::new(),
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: String,
    pub category: String,
    pub amount: f64,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub date: NaiveDate,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Expense {
    pub fn new(category: String, amount: f64, description: String, date: NaiveDate) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            amount,
            description,
            tags: Vec::new(),
            date,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct BudgetStore {
    pub budgets: BTreeMap<String, Budget>,
    pub expenses: BTreeMap<String, Expense>,
}


#[allow(dead_code)]
impl BudgetStore {
    pub fn add_budget(&mut self, budget: Budget) {
        self.budgets.insert(budget.category.clone(), budget);
    }

    pub fn remove_budget(&mut self, category: &str) -> Option<Budget> {
        self.budgets.remove(category)
    }

    pub fn get_budget_mut(&mut self, category: &str) -> Option<&mut Budget> {
        self.budgets.get_mut(category)
    }

    pub fn add_expense(&mut self, expense: Expense) {
        self.expenses.insert(expense.id.clone(), expense);
    }

    pub fn remove_expense(&mut self, id: &str) -> Option<Expense> {
        self.expenses.remove(id)
    }

    pub fn get_expense(&self, id: &str) -> Option<&Expense> {
        self.expenses.get(id)
    }

    pub fn get_expenses_by_category(&self, category: &str) -> Vec<&Expense> {
        self.expenses
            .values()
            .filter(|e| e.category == category)
            .collect()
    }

    pub fn get_expenses_in_period(&self, start: NaiveDate, end: NaiveDate) -> Vec<&Expense> {
        self.expenses
            .values()
            .filter(|e| e.date >= start && e.date <= end)
            .collect()
    }

    pub fn get_total_spent_for_category(&self, category: &str) -> f64 {
        self.get_expenses_by_category(category)
            .iter()
            .map(|e| e.amount)
            .sum()
    }

    pub fn get_total_spent_in_period(&self, start: NaiveDate, end: NaiveDate) -> f64 {
        self.get_expenses_in_period(start, end)
            .iter()
            .map(|e| e.amount)
            .sum()
    }

    pub fn budgets_count(&self) -> usize {
        self.budgets.len()
    }

    pub fn expenses_count(&self) -> usize {
        self.expenses.len()
    }
}

#[derive(Tabled)]
pub struct BudgetRow {
    #[tabled(rename = "CATEGORY")]
    category: String,
    #[tabled(rename = "AMOUNT")]
    amount: String,
    #[tabled(rename = "PERIOD")]
    period: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl BudgetRow {
    pub fn from_budget(budget: &Budget) -> Self {
        Self {
            category: budget.category.clone(),
            amount: format!("{:.2}", budget.amount),
            period: format!("{:?}", budget.period).to_lowercase(),
            tags: if budget.tags.is_empty() {
                "-".to_string()
            } else {
                budget.tags.join(", ")
            },
        }
    }
}

#[derive(Tabled)]
pub struct ExpenseRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "CATEGORY")]
    category: String,
    #[tabled(rename = "AMOUNT")]
    amount: String,
    #[tabled(rename = "DESCRIPTION")]
    description: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl ExpenseRow {
    pub fn from_expense(expense: &Expense) -> Self {
        Self {
            id: expense.id[..8].to_string(),
            category: expense.category.clone(),
            amount: format!("{:.2}", expense.amount),
            description: if expense.description.is_empty() {
                "-".to_string()
            } else {
                expense.description.clone()
            },
            date: expense.date.format("%Y-%m-%d").to_string(),
            tags: if expense.tags.is_empty() {
                "-".to_string()
            } else {
                expense.tags.join(", ")
            },
        }
    }
}

#[derive(Tabled)]
pub struct BudgetStatsRow {
    #[tabled(rename = "CATEGORY")]
    category: String,
    #[tabled(rename = "BUDGET")]
    budget: String,
    #[tabled(rename = "SPENT")]
    spent: String,
    #[tabled(rename = "REMAINING")]
    remaining: String,
    #[tabled(rename = "PERCENTAGE")]
    percentage: String,
}

impl BudgetStatsRow {
    pub fn from_budget(budget: &Budget, spent: f64) -> Self {
        let remaining = budget.amount - spent;
        let percentage = if budget.amount > 0.0 {
            (spent / budget.amount * 100.0).min(100.0)
        } else {
            0.0
        };

        Self {
            category: budget.category.clone(),
            budget: format!("{:.2}", budget.amount),
            spent: format!("{spent:.2}"),
            remaining: format!("{remaining:.2}"),
            percentage: format!("{percentage:.1}%"),
        }
    }
}
