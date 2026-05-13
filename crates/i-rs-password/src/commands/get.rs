use crate::presentation::{print_error, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_password: bool) -> Result<()> {
    let store = storage::load_store()?;

    let entry = match storage::get_entry(&store, &name) {
        Some(e) => e,
        None => {
            print_error(&format!("Entry '{}' not found", name));
            anyhow::bail!("Entry '{}' not found", name);
        }
    };

    print_header(&format!("Entry: {}", entry.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    println!("{:16} {}", "URL:".style(style), entry.url.cyan());

    if let Some(ref account) = entry.account {
        println!("{:16} {}", "Account:".style(style), account.yellow());
    }

    match storage::get_password(&name) {
        Ok(Some(pwd)) => {
            if show_password {
                println!("{:16} {}", "Password:".style(style), pwd.red());
            } else {
                println!("{:16} {}", "Password:".style(style), "(stored in keychain)".dimmed().to_string());
            }
        }
        Ok(None) => {
            println!("{:16} {}", "Password:".style(style), "(not set)".dimmed().to_string());
        }
        Err(e) => {
            println!("{:16} {}", "Password:".style(style), format!("(error: {})", e).red());
        }
    }

    if !entry.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !entry.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), entry.remark.join("; ").dimmed());
    }

    println!("{:16} {}", "Created:".style(style), entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
