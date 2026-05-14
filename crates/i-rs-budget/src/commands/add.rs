use crate::models::{Budget, BudgetPeriod};
use crate::presentation::{print_error, print_success, OutputFormat};
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
        if matches!(format, OutputFormat::Json) {
            println!("{}", serde_json::json!({
                "success": false,
                "error": { "code": "ALREADY_EXISTS", "message": format!("Budget for category '{}' already exists", category) }
            }));
        } else {
            print_error(&format!("Budget for category '{}' already exists", category));
        }
        anyhow::bail!("Budget for category '{}' already exists", category);
    }

    let budget_period = match period.as_deref() {
        Some("daily") | Some("d") => BudgetPeriod::Daily,
        Some("weekly") | Some("w") => BudgetPeriod::Weekly,
        Some("yearly") | Some("y") => BudgetPeriod::Yearly,
        Some("monthly") | Some("m") | None => BudgetPeriod::Monthly,
        _ => {
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": false,
                    "error": { "code": "INVALID_PERIOD", "message": "Invalid period. Use: daily, weekly, monthly, yearly" }
                }));
            } else {
                print_error("Invalid period. Use: daily, weekly, monthly, yearly");
            }
            anyhow::bail!("Invalid period");
        }
    };

    let mut budget = Budget::new(category.clone(), amount, budget_period);
    budget.tags = tags.clone();
    budget.remark = remark;
    budget.updated_at = Utc::now();

    let period_str = format!("{:?}", budget_period).to_lowercase();
    let tags_to_use = budget.tags.clone();

    store.add_budget(budget);
    storage::save_store(&store)?;

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct BudgetData {
            category: String,
            amount: f64,
            period: String,
            tags: Vec<String>,
        }
        println!("{}", serde_json::json!({
            "success": true,
            "data": BudgetData {
                category,
                amount,
                period: period_str,
                tags: tags_to_use,
            }
        }));
    } else {
        print_success(&format!("Added budget for '{}': {:.2}", category, amount));
    }

    Ok(())
}
