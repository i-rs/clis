use crate::presentation::{print_error, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_password: bool) -> Result<()> {
    let store = storage::load_store()?;

    let bookmark = match storage::get_bookmark(&store, &name) {
        Some(b) => b,
        None => {
            print_error(&format!("Bookmark '{}' not found", name));
            anyhow::bail!("Bookmark '{}' not found", name);
        }
    };

    print_header(&format!("Bookmark: {}", bookmark.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    println!("{:16} {}", "URL:".style(style), bookmark.url.cyan());

    if let Some(ref account) = bookmark.account {
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

    if !bookmark.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), bookmark.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !bookmark.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), bookmark.remark.join("; ").dimmed());
    }

    println!("{:16} {}", "Created:".style(style), bookmark.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), bookmark.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
