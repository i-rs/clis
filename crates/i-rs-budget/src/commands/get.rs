use crate::presentation::{output_item, print_error, print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(category: Option<String>, expense_id: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(cat) = category {
        if let Some(budget) = store.budgets.get(&cat) {
            if matches!(format, OutputFormat::Json) {
                #[derive(serde::Serialize)]
                struct BudgetData {
                    category: String,
                    amount: f64,
                    period: String,
                    tags: Vec<String>,
                    remark: Vec<String>,
                    created_at: String,
                    updated_at: String,
                }
                println!("{}", output_item(&BudgetData {
                    category: budget.category.clone(),
                    amount: budget.amount,
                    period: format!("{:?}", budget.period).to_lowercase(),
                    tags: budget.tags.clone(),
                    remark: budget.remark.clone(),
                    created_at: budget.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_at: budget.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }, format));
            } else {
                println!("\n{}", format!("Budget: {}", cat).cyan().bold());
                println!("  Amount: {:.2}", budget.amount);
                println!("  Period: {:?}", budget.period);
                if !budget.tags.is_empty() {
                    println!("  Tags: {}", budget.tags.join(", "));
                }
                if !budget.remark.is_empty() {
                    println!("  Remark: {}", budget.remark.join(", "));
                }
            }
        } else {
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": false,
                    "error": { "code": "NOT_FOUND", "message": format!("Budget for category '{}' not found", cat) }
                }));
            } else {
            }
            anyhow::bail!("Budget not found");
        }
    } else if let Some(id) = expense_id {
        if let Some(expense) = store.expenses.get(&id) {
            if matches!(format, OutputFormat::Json) {
                #[derive(serde::Serialize)]
                struct ExpenseData {
                    id: String,
                    category: String,
                    amount: f64,
                    description: String,
                    tags: Vec<String>,
                    date: String,
                    created_at: String,
                }
                println!("{}", output_item(&ExpenseData {
                    id: expense.id.clone(),
                    category: expense.category.clone(),
                    amount: expense.amount,
                    description: expense.description.clone(),
                    tags: expense.tags.clone(),
                    date: expense.date.format("%Y-%m-%d").to_string(),
                    created_at: expense.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }, format));
            } else {
                println!("\n{}", format!("Expense: {}", id).cyan().bold());
                println!("  Category: {}", expense.category);
                println!("  Amount: {:.2}", expense.amount);
                println!("  Description: {}", expense.description);
                println!("  Date: {}", expense.date.format("%Y-%m-%d"));
                if !expense.tags.is_empty() {
                    println!("  Tags: {}", expense.tags.join(", "));
                }
            }
        } else {
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": false,
                    "error": { "code": "NOT_FOUND", "message": format!("Expense '{}' not found", id) }
                }));
            } else {
            }
            anyhow::bail!("Expense not found");
        }
    } else {
        if matches!(format, OutputFormat::Json) {
            println!("{}", serde_json::json!({
                "success": false,
                "error": { "code": "MISSING_ARGUMENT", "message": "Provide either --category or --expense-id" }
            }));
        } else {
            print_error("Provide either --category or --expense-id");
            print_warning("Usage: i-rs-budget get --category <category>");
            print_warning("       i-rs-budget get --expense-id <id>");
        }
        anyhow::bail!("Missing argument");
    }

    Ok(())
}
