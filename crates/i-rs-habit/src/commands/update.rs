use crate::presentation::print_success;

pub fn handle_update(name: String, description: Option<String>, frequency: Option<String>, tags: Option<Vec<String>>, remark: Option<Vec<String>>) -> anyhow::Result<()> {
    crate::service::update_habit(name.clone(), description, frequency, tags, remark)?;
    print_success(&format!("Habit '{name}' updated successfully"));
    Ok(())
}
