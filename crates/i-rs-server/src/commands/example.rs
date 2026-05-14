use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-server Examples".bold().cyan());
    println!();

    println!("{}", "Add Server:".bold().green());
    println!("  i-rs-server add web1 192.168.1.100 22 --user admin --tag production");
    println!();

    println!("{}", "List Servers:".bold().green());
    println!("  i-rs-server list");
    println!("  i-rs-server list --tag production");
    println!();

    println!("{}", "Get Server Details:".bold().green());
    println!("  i-rs-server get web1");
    println!("  i-rs-server get web1 --show-password");
    println!();

    println!("{}", "Update Server:".bold().green());
    println!("  i-rs-server update web1 --user newadmin");
    println!("  i-rs-server update web1 --tag web --tag production");
    println!();

    println!("{}", "Delete Server:".bold().green());
    println!("  i-rs-server delete web1");
    println!();

    println!("{}", "SSH Command Suggestions:".bold().green());
    println!("  i-rs-server suggest web1");
    println!("  i-rs-server suggest web1 --command docker");
    println!();

    println!("{}", "Tip:".dimmed());
    println!("  Use --json flag for JSON output: i-rs-server list --json");
}
