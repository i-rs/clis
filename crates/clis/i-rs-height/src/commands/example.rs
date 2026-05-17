use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-height Examples".bold().cyan());
    println!();

    println!("{}", "Add Height Record:".bold().green());
    println!("  i-rs-height add 2025-06-14 175.5");
    println!("  i-rs-height add 2025-06-14 175.5 --weight 68.5");
    println!("  i-rs-height add 2025-06-15 175.8 --tag \"morning\" --remark \"After breakfast\"");
    println!();

    println!("{}", "List Records:".bold().green());
    println!("  i-rs-height list");
    println!("  i-rs-height list --days 30");
    println!();

    println!("{}", "List with Chart:".bold().green());
    println!("  i-rs-height list --chart");
    println!("  i-rs-height list --days 7 --chart");
    println!();

    println!("{}", "List with Stats:".bold().green());
    println!("  i-rs-height list --stats");
    println!("  i-rs-height list --days 30 --stats");
    println!();

    println!("{}", "Get Record:".bold().green());
    println!("  i-rs-height get 2025-06-14");
    println!();

    println!("{}", "Delete Record:".bold().green());
    println!("  i-rs-height delete 2025-06-15");
    println!();

    println!("{}", "Set Target:".bold().green());
    println!("  i-rs-height set 180.0");
    println!();

    println!("{}", "Show Target:".bold().green());
    println!("  i-rs-height target");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-height list --json");
    println!("  i-rs-height get 2025-06-14 --json");
    println!();
}
