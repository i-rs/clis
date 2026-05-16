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

    println!("{}", "Search Entries:".bold().green());
    println!("  i-rs-kv search xxx");
    println!("  i-rs-kv search config --json");
    println!();

    println!("{}", "List with Pattern:".bold().green());
    println!("  i-rs-kv list --pattern api");
    println!("  i-rs-kv list --pattern config --json");
    println!();

    println!("{}", "Copy & Rename:".bold().green());
    println!("  i-rs-kv copy api_key api_key_backup");
    println!("  i-rs-kv rename old_key new_key");
    println!();

    println!("{}", "Statistics:".bold().green());
    println!("  i-rs-kv stats");
    println!("  i-rs-kv stats --json");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-kv list --json");
    println!("  i-rs-kv get api_key --json");
    println!();

    println!("{}", "Data Management:".bold().green());
    println!("  i-rs-kv data export");
    println!("  i-rs-kv data import backup.json");
    println!("  i-rs-kv data clear");
    println!();

    println!("{}", "Skill Info:".bold().green());
    println!("  i-rs-kv skill");
    println!("  i-rs-kv skill summary");
    println!();
}
