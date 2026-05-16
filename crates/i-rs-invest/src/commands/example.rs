use crate::presentation::print_header;
use owo_colors::OwoColorize;

pub fn handle_example() {
    print_header("i-rs-invest Examples");
    println!();
    println!("{}", "Add a stock investment:".cyan());
    println!("  i-rs-invest add Apple --symbol AAPL --type stock --quantity 10 --price 150.00");
    println!();
    println!("{}", "Add a fund investment:".cyan());
    println!(
        "  i-rs-invest add Index Fund --symbol VTI --type fund --quantity 50 --price 200.00 --date 2024-01-15"
    );
    println!();
    println!("{}", "Add a crypto investment:".cyan());
    println!(
        "  i-rs-invest add Bitcoin --symbol BTC --type crypto --quantity 0.5 --price 40000.00"
    );
    println!();
    println!("{}", "Add with tags and remarks:".cyan());
    println!(
        "  i-rs-invest add Tesla --symbol TSLA --type stock --qty 15 --price 250.00 --tag tech --tag growth --remark Long term hold"
    );
    println!();
    println!("{}", "Update current price:".cyan());
    println!("  i-rs-invest update Apple --current-price 175.50");
    println!();
    println!("{}", "List all investments:".cyan());
    println!("  i-rs-invest list");
    println!();
    println!("{}", "Filter by type:".cyan());
    println!("  i-rs-invest list --type stock");
    println!("  i-rs-invest list --type fund");
    println!("  i-rs-invest list --type crypto");
    println!();
    println!("{}", "Filter by tag:".cyan());
    println!("  i-rs-invest list --tag tech");
    println!();
    println!("{}", "Get investment details:".cyan());
    println!("  i-rs-invest get Apple");
    println!();
    println!("{}", "View statistics:".cyan());
    println!("  i-rs-invest stats");
    println!();
    println!("{}", "Delete an investment:".cyan());
    println!("  i-rs-invest delete Apple");
    println!();
    println!("{}", "JSON output:".cyan());
    println!("  i-rs-invest list --json");
    println!("  i-rs-invest get Apple --json");
}
