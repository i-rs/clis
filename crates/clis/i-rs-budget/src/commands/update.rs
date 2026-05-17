use crate::models::BudgetPeriod;
use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn handle_update(
    category: String,
    amount: Option<f64>,
    period: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let budget = if let Some(b) = store.budgets.get_mut(&category) {
        b
    } else {
        if format.is_json() {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": { "code": "NOT_FOUND", "message": format!("Budget for category '{}' not found", category) }
                })
            );
        }
        anyhow::bail!("Budget not found");
    };

    i_rs_core::update_field!(budget.amount, amount);

    if let Some(p) = period {
        budget.period = match p.as_str() {
            "daily" | "d" => BudgetPeriod::Daily,
            "weekly" | "w" => BudgetPeriod::Weekly,
            "yearly" | "y" => BudgetPeriod::Yearly,
            "monthly" | "m" => BudgetPeriod::Monthly,
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
    }

    i_rs_core::update_field!(budget.tags, tags);

    i_rs_core::update_field!(budget.remark, remark);

    budget.updated_at = Utc::now();

    let updated_data = (
        budget.category.clone(),
        budget.amount,
        format!("{:?}", budget.period).to_lowercase(),
        budget.tags.clone(),
        budget.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    );

    storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct BudgetData {
            category: String,
            amount: f64,
            period: String,
            tags: Vec<String>,
            updated_at: String,
        }
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "data": BudgetData {
                    category: updated_data.0,
                    amount: updated_data.1,
                    period: updated_data.2,
                    tags: updated_data.3,
                    updated_at: updated_data.4,
                }
            })
        );
    } else {
        print_success(&format!("Updated budget for '{category}'"));
    }

    Ok(())
}
