use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-weight Examples".bold().cyan());
    println!();

    println!("{}", "Add Weight Record:".bold().green());
    println!("  i-rs-weight add 70.5");
    println!("  i-rs-weight add 70.5 --date 2025-06-14 --remark \"After workout\"");
    println!("  i-rs-weight add 70.3 --date 2025-06-15");
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

    println!("{}", "Get Record by ID:".bold().green());
    println!("  i-rs-weight get abc12345");
    println!();

    println!("{}", "Update Record:".bold().green());
    println!("  i-rs-weight update abc12345 --weight 70.2");
    println!("  i-rs-weight update abc12345 --tag morning --remark \"Morning weight\"");
    println!();

    println!("{}", "Delete Record:".bold().green());
    println!("  i-rs-weight delete abc12345");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-weight list --json");
    println!();
}
