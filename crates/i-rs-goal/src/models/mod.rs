use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub amount: f64,
    pub reached: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reached_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsGoal {
    pub id: String,
    pub name: String,
    pub target_amount: f64,
    pub current_amount: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub deadline: DateTime<Utc>,
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl SavingsGoal {
    pub fn progress_percentage(&self) -> f64 {
        if self.target_amount <= 0.0 {
            return 0.0;
        }
        (self.current_amount / self.target_amount * 100.0).min(100.0)
    }

    pub fn remaining_amount(&self) -> f64 {
        (self.target_amount - self.current_amount).max(0.0)
    }

    pub fn days_until_deadline(&self) -> i64 {
        (self.deadline - Utc::now()).num_days()
    }

    pub fn check_milestones(&mut self) {
        for milestone in &mut self.milestones {
            if !milestone.reached && self.current_amount >= milestone.amount {
                milestone.reached = true;
                milestone.reached_at = Some(Utc::now().format("%Y-%m-%d %H:%M").to_string());
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoalStore {
    pub goals: BTreeMap<String, SavingsGoal>,
}

impl GoalStore {
    pub fn add_entry(&mut self, entry: SavingsGoal) {
        self.goals.insert(entry.name.clone(), entry);
    }

    pub fn get_entry(&self, name: &str) -> Option<&SavingsGoal> {
        self.goals.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut SavingsGoal> {
        self.goals.get_mut(name)
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<SavingsGoal> {
        self.goals.remove(name)
    }
}

#[derive(Tabled, Clone)]
pub struct SavingsGoalRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Target")]
    pub target: String,
    #[tabled(rename = "Current")]
    pub current: String,
    #[tabled(rename = "Progress")]
    pub progress: String,
    #[tabled(rename = "Days Left")]
    pub days_left: String,
    #[tabled(rename = "Tags")]
    pub tags: String,
}

impl SavingsGoalRow {
    pub fn from_goal(goal: &SavingsGoal) -> Self {
        let progress = goal.progress_percentage();
        let days_left = goal.days_until_deadline();

        Self {
            name: goal.name.clone(),
            target: format!("{:.2}", goal.target_amount),
            current: format!("{:.2}", goal.current_amount),
            progress: format!("{progress:.1}%"),
            days_left: if days_left < 0 {
                "Overdue".to_string()
            } else {
                days_left.to_string()
            },
            tags: if goal.tags.is_empty() {
                "-".to_string()
            } else {
                goal.tags.join(", ")
            },
        }
    }
}

#[derive(Tabled, Clone)]
pub struct MilestoneRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Amount")]
    pub amount: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Reached At")]
    pub reached_at: String,
}

impl MilestoneRow {
    pub fn from_milestone(milestone: &Milestone) -> Self {
        Self {
            name: milestone.name.clone(),
            amount: format!("{:.2}", milestone.amount),
            status: if milestone.reached {
                "✓ Reached".to_string()
            } else {
                "○ Pending".to_string()
            },
            reached_at: milestone
                .reached_at
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        }
    }
}
