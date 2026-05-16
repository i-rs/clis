use crate::presentation::{print_header, print_success};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_purchase(name: String) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    let item = storage::toggle_purchased(&mut store, &name)?;
    storage::save_store(&store)?;

    print_header("Purchase Status Updated");
    println!(
        "{} {}",
        "Name:".style(owo_colors::Style::new().bold()),
        name
    );
    let status = if item.purchased {
        "✅ Purchased"
    } else {
        "🔄 Needed"
    };
    println!(
        "{} {}",
        "Status:".style(owo_colors::Style::new().bold()),
        status
    );

    if item.purchased {
        print_success(&format!("Item '{name}' marked as purchased"));
    } else {
        print_success(&format!("Item '{name}' marked as needed"));
    }

    Ok(())
}
