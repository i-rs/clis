use crate::presentation::{print_success, OutputFormat};

pub fn handle_delete(id: String, format: OutputFormat) -> anyhow::Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::delete_habit(&mut store, &id)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        println!("{}", serde_json::json!({"success": true, "message": format!("Habit '{}' deleted", id)}));
        return Ok(());
    }
    print_success(&format!("Habit '{}' deleted successfully", id));
    Ok(())
}
