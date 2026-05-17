use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-weight Examples".bold().cyan());
    println!();

    println!("{}", "Add Weight Record:".bold().green());
    println!("  i-rs-weight add 2025-06-14 70.5 --remark \"After workout\"");
    println!("  i-rs-weight add 2025-06-15 70.3");
    println!();

    println!("{}", "List Records:".bold().green());
    println!("  i-rs-weight list");
    println!("  i-rs-weight list --days 30");
    println!();

    println!("{}", "List with Chart:".bold().green());
    println!("  i-rs-weight list --chart");
    println!("  i-rs-weight list --days 7 --chart");
    println!();

    println!("{}", "List with Stats:".bold().green());
    println!("  i-rs-weight list --stats");
    println!("  i-rs-weight list --days 30 --stats");
    println!();

    println!("{}", "Update Record:".bold().green());
    println!("  i-rs-weight update 2025-06-14 --weight 70.2 --remark \"Morning weight\"");
    println!();

    println!("{}", "Delete Record:".bold().green());
    println!("  i-rs-weight delete 2025-06-15");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-weight list --json");
    println!();
}
