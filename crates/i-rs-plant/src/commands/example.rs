use owo_colors::OwoColorize;

pub fn example() {
    println!("{} {}", "i-rs-plant".cyan().bold(), "Examples".dimmed());
    println!();

    println!("{}", "Add a new plant:".green());
    println!("  i-rs-plant add --name \"Monstera\" --species \"Monstera deliciosa\" --location \"Living room\" --interval 7");
    println!();

    println!("{}", "List all plants:".green());
    println!("  i-rs-plant list");
    println!();

    println!("{}", "List plants with tag filter:".green());
    println!("  i-rs-plant list --tag indoor");
    println!();

    println!("{}", "Get plant details:".green());
    println!("  i-rs-plant get Monstera");
    println!();

    println!("{}", "Water a plant:".green());
    println!("  i-rs-plant water Monstera");
    println!();

    println!("{}", "Update plant info:".green());
    println!("  i-rs-plant update Monstera --location \"Bedroom\" --interval 10");
    println!();

    println!("{}", "View statistics:".green());
    println!("  i-rs-plant stats");
    println!();

    println!("{}", "Delete a plant:".green());
    println!("  i-rs-plant delete Monstera");
    println!();

    println!("{}", "JSON output (all commands):".green());
    println!("  i-rs-plant list --json");
    println!("  i-rs-plant get Monstera --json");
}
