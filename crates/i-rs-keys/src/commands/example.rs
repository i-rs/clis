use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-keys Examples".bold().cyan());
    println!();

    println!("{}", "Add Key (value stored in OS keychain):".bold().green());
    println!("  i-rs-keys add \"github_token\" \"ghp_xxx\" api_key --tag coding --remark \"GitHub PAT\"");
    println!("  i-rs-keys add \"openai_api\" \"sk-xxx\" api_key --tag ai --remark \"OpenAI API key\"");
    println!("  i-rs-keys add \"aws_access\" \"AKIAXXX\" aws_key --tag cloud --remark \"AWS credentials\"");
    println!("  i-rs-keys add \"ssh_work\" \"~/.ssh/id_rsa\" ssh_key --tag work");
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
