use crate::presentation::print_success;

pub fn handle_delete(name: String) -> anyhow::Result<()> {
    let mut store = crate::storage::load_store()?;
    crate::service::delete_habit(&mut store, &name)?;
    crate::storage::save_store(&store)?;
    print_success(&format!("Habit '{name}' deleted successfully"));
    Ok(())
}
