use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-domain Examples".bold().cyan());
    println!();

    println!("{}", "Add Domain:".bold().green());
    println!("  i-rs-domain add example.com 2025-12-31 --registrar \"GoDaddy\" --tag production");
    println!("  i-rs-domain add test.org 2026-06-30 --registrar \"Namecheap\" --tag testing");
    println!();

    println!("{}", "List Domains:".bold().green());
    println!("  i-rs-domain list");
    println!("  i-rs-domain list --tag production");
    println!();

    println!("{}", "Get Domain:".bold().green());
    println!("  i-rs-domain get example.com");
    println!("  i-rs-domain get example.com --show-password");
    println!();

    println!("{}", "Update Domain:".bold().green());
    println!("  i-rs-domain update example.com --expiry-date 2026-12-31 --remark \"Renewed for 2 years\"");
    println!();

    println!("{}", "Delete Domain:".bold().green());
    println!("  i-rs-domain delete test.org");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-domain list --json");
    println!("  i-rs-domain get example.com --json");
    println!();
}