use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    url: Option<String>,
    account: Option<String>,
    password: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let bookmark = match store.get_entry_mut(&name) {
        Some(b) => b,
        None => {
            anyhow::bail!("Bookmark '{name}' not found");
        }
    };

    if let Some(url) = url {
        bookmark.url = url;
    }
    if let Some(account) = account {
        bookmark.account = Some(account);
    }
    if let Some(password) = password {
        storage::store_password(&name, &password)?;
        println!("{}", "Password updated and stored securely in keychain".green());
    }
    if let Some(tag) = tag {
        bookmark.tags = tag;
    }
    if let Some(remark) = remark {
        bookmark.remark = remark;
    }

    bookmark.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Bookmark '{}' updated successfully", name.green()));

    Ok(())
}
