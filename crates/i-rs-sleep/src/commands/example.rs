use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-sleep Examples".bold().cyan());
    println!();

    println!("{}", "Record sleep (bedtime, wake time, quality 1-5):".bold().green());
    println!("  i-rs-sleep add 22:30 06:45 4 --tag workday");
    println!("  i-rs-sleep add 23:00 07:00 5 --tag weekend --remark \"Great sleep!\"");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-sleep list");
    println!("  i-rs-sleep list --tag workday");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-sleep get abc12345");
    println!();

    println!("{}", "Show statistics:".bold().green());
    println!("  i-rs-sleep stats");
    println!();

    println!("{}", "Update record:".bold().green());
    println!("  i-rs-sleep update abc12345 --quality 5");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-sleep delete abc12345");
    println!();

    println!("{}", "Quality Scale:".bold().yellow());
    println!("  1 - 😴 Awful");
    println!("  2 - 😪 Poor");
    println!("  3 - 😌 Fair");
    println!("  4 - 😊 Good");
    println!("  5 - 😁 Excellent");
    println!();
}