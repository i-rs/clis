use crate::presentation::{print_header, print_success};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    quantity: i32,
    unit: String,
    tags: Vec<String>,
    remark: Vec<String>,
) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    if store.get_entry(&name).is_some() {
        anyhow::bail!("Item '{name}' already exists");
    }

    let item = storage::add_item(&mut store, name.clone(), quantity, unit, tags, remark)?;
    storage::save_store(&store)?;

    print_header("Grocery Item Added");
    println!(
        "{} {}",
        "Name:".style(owo_colors::Style::new().bold()),
        name
    );
    println!(
        "{} {} {}",
        "Quantity:".style(owo_colors::Style::new().bold()),
        item.quantity,
        item.unit
    );
    print_success(&format!("Item '{name}' added to grocery list"));

    Ok(())
}
