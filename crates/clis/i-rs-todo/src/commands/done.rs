use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use anyhow::Result;

pub fn handle_done(name: String, format: OutputFormat) -> Result<()> {
    let mut store = crate::storage::load_store()?;
    let todo = crate::service::toggle_todo_done(&mut store, &name)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&todo);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    if todo.is_done {
        print_success(&format!("✓ Todo '{}' marked as done", name));
    } else {
        print_success(&format!("✓ Todo '{}' marked as pending", name));
    }

    Ok(())
}
