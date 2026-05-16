use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-run Examples".bold().cyan());
    println!();

    println!("{}", "Add Run Record:".bold().green());
    println!("  i-rs-run add 2025-06-14 5.0 30 --heart-rate 145 --weather sunny");
    println!("  i-rs-run add 2025-06-15 10.0 60 --tags marathon --remark \"Long run\"");
    println!();

    println!("{}", "List Records:".bold().green());
    println!("  i-rs-run list");
    println!("  i-rs-run list --json");
    println!();

    println!("{}", "Get Record Details:".bold().green());
    println!("  i-rs-run get <ID>");
    println!("  i-rs-run get <ID> --json");
    println!();

    println!("{}", "Delete Record:".bold().green());
    println!("  i-rs-run delete <ID>");
    println!();

    println!("{}", "View Statistics:".bold().green());
    println!("  i-rs-run stats");
    println!();

    println!("{}", "Add Run Plan:".bold().green());
    println!("  i-rs-run plan-add \"5K Training\" 5.0 6:00 --schedule 1 3 5 --tags beginner");
    println!();

    println!("{}", "List Plans:".bold().green());
    println!("  i-rs-run plan-list");
    println!("  i-rs-run plan-list --json");
    println!();

    println!("{}", "Get Plan Details:".bold().green());
    println!("  i-rs-run plan-get <ID>");
    println!();

    println!("{}", "Delete Plan:".bold().green());
    println!("  i-rs-run plan-delete <ID>");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-run list --json");
    println!("  i-rs-run get <ID> --json");
    println!();
}
