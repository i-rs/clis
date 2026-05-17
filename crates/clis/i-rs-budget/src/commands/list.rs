use crate::models::{Budget, Expense};
use crate::presentation::{
    OutputFormat, format_budget_table, format_expense_table, output_list, print_budget_count,
    print_expense_count, print_warning,
};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    list_type: Option<String>,
    category: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let list_type = list_type.unwrap_or_else(|| "budgets".to_string());

    match list_type.as_str() {
        "budgets" | "budget" | "b" => {
            let budgets: Vec<&Budget> = if let Some(ref cat) = category {
                store.budgets.get(cat).into_iter().collect()
            } else {
                store.budgets.values().collect()
            };

            if budgets.is_empty() {
                if format.is_json() {
                    let filter = category.as_deref();
                    println!(
                        "{}",
                        output_list::<serde_json::Value>(&[], 0, filter, format)
                    );
                } else {
                    print_warning("No budgets found.");
                }
                return Ok(());
            }

            let budgets_ref: Vec<&Budget> = budgets;

            if format.is_json() {
                #[derive(serde::Serialize, Clone)]
                struct BudgetListItem {
                    category: String,
                    amount: f64,
                    period: String,
                    tags: Vec<String>,
                }

                let items: Vec<BudgetListItem> = budgets_ref
                    .iter()
                    .map(|b| BudgetListItem {
                        category: b.category.clone(),
                        amount: b.amount,
                        period: format!("{:?}", b.period).to_lowercase(),
                        tags: b.tags.clone(),
                    })
                    .collect();

                let filter = category.as_deref();
                println!("{}", output_list(&items, items.len(), filter, format));
            } else {
                let table = format_budget_table(&budgets_ref);
                println!("\n{table}");
                print_budget_count(budgets_ref.len());
            }
        }
        "expenses" | "expense" | "e" => {
            let expenses: Vec<&Expense> = if let Some(ref cat) = category {
                store.get_expenses_by_category(cat)
            } else {
                store.expenses.values().collect()
            };

            if expenses.is_empty() {
                if format.is_json() {
                    let filter = category.as_deref();
                    println!(
                        "{}",
                        output_list::<serde_json::Value>(&[], 0, filter, format)
                    );
                } else {
                    print_warning("No expenses found.");
                }
                return Ok(());
            }

            let expenses_ref: Vec<&Expense> = expenses;

            if format.is_json() {
                #[derive(serde::Serialize, Clone)]
                struct ExpenseListItem {
                    id: String,
                    category: String,
                    amount: f64,
                    description: String,
                    date: String,
                    tags: Vec<String>,
                }

                let items: Vec<ExpenseListItem> = expenses_ref
                    .iter()
                    .map(|e| ExpenseListItem {
                        id: e.id[..8].to_string(),
                        category: e.category.clone(),
                        amount: e.amount,
                        description: e.description.clone(),
                        date: e.date.format("%Y-%m-%d").to_string(),
                        tags: e.tags.clone(),
                    })
                    .collect();

                let filter = category.as_deref();
                println!("{}", output_list(&items, items.len(), filter, format));
            } else {
                let table = format_expense_table(&expenses_ref);
                println!("\n{table}");
                print_expense_count(expenses_ref.len());
            }
        }
        _ => {
            if format.is_json() {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": { "code": "INVALID_TYPE", "message": "Invalid list type. Use: budgets, expenses" }
                    })
                );
            }
            anyhow::bail!("Invalid list type");
        }
    }

    Ok(())
}
