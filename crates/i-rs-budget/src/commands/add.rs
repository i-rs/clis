use crate::models::{Budget, BudgetPeriod};
use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn handle_add(
    category: String,
    amount: f64,
    period: Option<String>,
    tags: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.budgets.contains_key(&category) {
        if format.is_json() {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": { "code": "ALREADY_EXISTS", "message": format!("Budget for category '{}' already exists", category) }
                })
            );
        }
        anyhow::bail!("Budget for category '{category}' already exists");
    }

    let budget_period = match period.as_deref() {
        Some("daily" | "d") => BudgetPeriod::Daily,
        Some("weekly" | "w") => BudgetPeriod::Weekly,
        Some("yearly" | "y") => BudgetPeriod::Yearly,
        Some("monthly" | "m") | None => BudgetPeriod::Monthly,
        _ => {
            if format.is_json() {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": { "code": "INVALID_PERIOD", "message": "Invalid period. Use: daily, weekly, monthly, yearly" }
                    })
                );
            }
            anyhow::bail!("Invalid period");
        }
    };

    let mut budget = Budget::new(category.clone(), amount, budget_period);
    budget.tags = tags;
    budget.remark = remark;
    budget.updated_at = Utc::now();

    let period_str = format!("{budget_period:?}").to_lowercase();
    let tags_to_use = budget.tags.clone();

    store.add_budget(budget);
    storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct BudgetData {
            category: String,
            amount: f64,
            period: String,
            tags: Vec<String>,
        }
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "data": BudgetData {
                    category,
                    amount,
                    period: period_str,
                    tags: tags_to_use,
                }
            })
        );
    } else {
        print_success(&format!("Added budget for '{category}': {amount:.2}"));
    }

    Ok(())
}
