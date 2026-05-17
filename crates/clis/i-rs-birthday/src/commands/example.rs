use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-birthday Examples".bold().cyan());
    println!();

    println!("{}", "Add Birthday:".bold().green());
    println!("  i-rs-birthday add John 06-15 --year 1990 --relationship friend --tag personal");
    println!("  i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag important");
    println!("  i-rs-birthday add Colleague 03-10 --relationship colleague");
    println!();

    println!("{}", "List Birthdays:".bold().green());
    println!("  i-rs-birthday list");
    println!("  i-rs-birthday list --tag family");
    println!();

    println!("{}", "Get Birthday Details:".bold().green());
    println!("  i-rs-birthday get John");
    println!();

    println!("{}", "Update Birthday:".bold().green());
    println!("  i-rs-birthday update John --birth-date 06-20 --tag work");
    println!("  i-rs-birthday update Mom --relationship \"close family\"");
    println!();

    println!("{}", "Delete Birthday:".bold().green());
    println!("  i-rs-birthday delete John");
    println!();

    println!("{}", "View Statistics:".bold().green());
    println!("  i-rs-birthday stats");
    println!();

    println!("{}", "View Upcoming Birthdays:".bold().green());
    println!("  i-rs-birthday upcoming");
    println!("  i-rs-birthday upcoming --days 7");
    println!("  i-rs-birthday upcoming --days 60");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-birthday list --json");
    println!("  i-rs-birthday get John --json");
    println!("  i-rs-birthday stats --json");
    println!("  i-rs-birthday upcoming --json");
    println!();
}
