use crate::presentation::{print_success, OutputFormat};

pub fn handle_update(
    id: String,
    description: Option<String>,
    frequency: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::update_habit(
        &mut store,
        id.clone(),
        description,
        frequency,
        tag,
        remark,
    )?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        println!("{}", serde_json::json!({"success": true, "message": format!("Habit '{}' updated", id)}));
        return Ok(());
    }
    print_success(&format!("Habit '{}' updated successfully", id));
    Ok(())
}
