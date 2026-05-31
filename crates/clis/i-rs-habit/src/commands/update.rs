use crate::presentation::print_success;

pub fn handle_update(
    name: String,
    description: Option<String>,
    frequency: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::update_habit(
        &mut store,
        name.clone(),
        description,
        frequency,
        tags,
        remark,
    )?;
    crate::storage::save_store(&store)?;
    print_success(&format!("Habit '{name}' updated successfully"));
    Ok(())
}
