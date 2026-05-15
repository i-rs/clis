use crate::presentation::print_success;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    url: Option<String>,
    account: Option<String>,
    password: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    crate::service::update_bookmark(name.clone(), url, account, password, tag, remark)?;
    print_success(&format!("✓ Bookmark '{}' updated successfully", name.green()));
    Ok(())
}
