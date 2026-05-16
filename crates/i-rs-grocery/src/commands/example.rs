use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-grocery Examples".bold().cyan());
    println!();

    println!("{}", "Add items to grocery list:".bold().green());
    println!("  i-rs-grocery add milk 2 bottles --tag dairy");
    println!("  i-rs-grocery add eggs 1 dozen --tag dairy");
    println!("  i-rs-grocery add bread 1 loaf --tag bakery");
    println!();

    println!("{}", "List items:".bold().green());
    println!("  i-rs-grocery list");
    println!("  i-rs-grocery list --tag dairy");
    println!("  i-rs-grocery list --purchased");
    println!("  i-rs-grocery list --needed");
    println!();

    println!("{}", "Mark as purchased:".bold().green());
    println!("  i-rs-grocery purchase milk");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-grocery get milk");
    println!();

    println!("{}", "Update item:".bold().green());
    println!("  i-rs-grocery update milk --quantity 3");
    println!();

    println!("{}", "Clear purchased items:".bold().green());
    println!("  i-rs-grocery clear");
    println!();

    println!("{}", "Delete item:".bold().green());
    println!("  i-rs-grocery delete bread");
    println!();
}
