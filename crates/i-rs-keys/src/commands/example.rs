use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-keys Examples".bold().cyan());
    println!();

    println!("{}", "Add Key (value stored in OS keychain):".bold().green());
    println!("  i-rs-keys add \"github_token\" \"token_xxx\" --tag coding --remark \"GitHub personal access token\"");
    println!("  i-rs-keys add \"openai_api\" \"sk-xxx\" --tag ai --remark \"OpenAI API key\"");
    println!("  i-rs-keys add \"vercel_token\" \"xxx\" --tag hosting --type \"api_key\"");
    println!();

    println!("{}", "List Keys:".bold().green());
    println!("  i-rs-keys list");
    println!("  i-rs-keys list --tag coding");
    println!();

    println!("{}", "Get Key:".bold().green());
    println!("  i-rs-keys get github_token");
    println!("  i-rs-keys get github_token --show-value");
    println!();

    println!("{}", "Update Key:".bold().green());
    println!("  i-rs-keys update github_token --key-value \"new_token_xxx\"");
    println!();

    println!("{}", "Delete Key:".bold().green());
    println!("  i-rs-keys delete old_key");
    println!();

    println!("{}", "Key Types:".bold().yellow());
    println!("  api_key, token, password, certificate, ssh_key, etc.");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-keys list --json");
    println!("  i-rs-keys get github_token --json");
    println!();

    println!("{}", "Security:".bold().red());
    println!("  Values are stored in OS keychain (Keychain/macOS, Credential Manager/Windows)");
    println!();
}
