use crate::presentation::print_success;

pub fn handle_delete(name: String) -> anyhow::Result<()> {
    crate::service::delete_habit(&name)?;
    print_success(&format!("Habit '{name}' deleted successfully"));
    Ok(())
}
