use crate::models::Expense;
use crate::presentation::{OutputFormat, output_item, print_error, print_success, print_warning};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_expense(
    category: String,
    amount: f64,
    description: String,
    date: Option<String>,
    tags: Vec<String>,
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.budgets.contains_key(&category) {
        if format.is_json() {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": { "code": "NOT_FOUND", "message": format!("Budget for category '{}' not found. Create it first with 'add' command.", category) }
                })
            );
        } else {
            print_error(&format!("Budget for category '{category}' not found"));
            print_warning("Create it first with: i-rs-budget add <category> <amount>");
        }
        anyhow::bail!("Budget not found");
    }

    let expense_date = if let Some(d) = date {
        chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").unwrap_or_else(|_| {
            chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().date_naive())
        })
    } else {
        Utc::now().date_naive()
    };

    let mut expense = Expense::new(category.clone(), amount, description, expense_date);
    expense.tags = tags;

    store.add_expense(expense.clone());
    storage::save_store(&store)?;

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct ExpenseData {
            id: String,
            category: String,
            amount: f64,
            description: String,
            date: String,
            tags: Vec<String>,
        }
        println!(
            "{}",
            output_item(
                &ExpenseData {
                    id: expense.id,
                    category: expense.category,
                    amount: expense.amount,
                    description: expense.description,
                    date: expense.date.format("%Y-%m-%d").to_string(),
                    tags: expense.tags,
                },
                format
            )
        );
    } else {
        print_success(&format!("Added expense {amount:.2} to '{category}'"));
        println!("  ID: {}", expense.id[..8].to_string().cyan());
    }

    Ok(())
}
