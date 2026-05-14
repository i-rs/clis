use crate::presentation::{print_error, print_success, print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_delete(category: Option<String>, expense_id: Option<String>, format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Some(cat) = category {
        if store.budgets.remove(&cat).is_none() {
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": false,
                    "error": { "code": "NOT_FOUND", "message": format!("Budget for category '{}' not found", cat) }
                }));
            } 
            anyhow::bail!("Budget not found");
        }
        storage::save_store(&store)?;

        if matches!(format, OutputFormat::Json) {
            println!("{}", serde_json::json!({
                "success": true,
                "data": { "message": format!("Budget for '{}' deleted", cat) }
            }));
        } else {
            print_success(&format!("Deleted budget for '{cat}'"));
        }
    } else if let Some(id) = expense_id {
        if store.expenses.remove(&id).is_none() {
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": false,
                    "error": { "code": "NOT_FOUND", "message": format!("Expense '{}' not found", id) }
                }));
            } 
            anyhow::bail!("Expense not found");
        }
        storage::save_store(&store)?;

        if matches!(format, OutputFormat::Json) {
            println!("{}", serde_json::json!({
                "success": true,
                "data": { "message": format!("Expense '{}' deleted", id) }
            }));
        } else {
            print_success(&format!("Deleted expense '{id}'"));
        }
    } else {
        if matches!(format, OutputFormat::Json) {
            println!("{}", serde_json::json!({
                "success": false,
                "error": { "code": "MISSING_ARGUMENT", "message": "Provide either --category or --expense-id" }
            }));
        } else {
            print_error("Provide either --category or --expense-id");
            print_warning("Usage: i-rs-budget delete --category <category>");
            print_warning("       i-rs-budget delete --expense-id <id>");
        }
        anyhow::bail!("Missing argument");
    }

    Ok(())
}
