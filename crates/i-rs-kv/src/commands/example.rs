use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-kv Examples".bold().cyan());
    println!();

    println!("{}", "Add Key-Value:".bold().green());
    println!("  i-rs-kv add api_key \"sk-xxx\" --tag production");
    println!("  i-rs-kv add config 'JSON' --tag config");
    println!();

    println!("{}", "List Entries:".bold().green());
    println!("  i-rs-kv list");
    println!("  i-rs-kv list --tag production");
    println!();

    println!("{}", "Get Value:".bold().green());
    println!("  i-rs-kv get api_key");
    println!();

    println!("{}", "Update Value:".bold().green());
    println!("  i-rs-kv update api_key --value \"sk-new-xxx\"");
    println!();

    println!("{}", "Delete Entry:".bold().green());
    println!("  i-rs-kv delete old_key");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-kv list --json");
    println!("  i-rs-kv get api_key --json");
    println!();
}
