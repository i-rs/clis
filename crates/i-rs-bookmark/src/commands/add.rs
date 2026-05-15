use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    url: String,
    account: Option<String>,
    password: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let has_password = password.is_some();
    crate::service::add_bookmark(name.clone(), url, account, password, tag, remark)?;
    print_success(&format!("✓ Bookmark '{}' added successfully", name.green()));
    if has_password {
        println!("  {}", "Password stored securely in keychain".dimmed());
    }
    Ok(())
}
