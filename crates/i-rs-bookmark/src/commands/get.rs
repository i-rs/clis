use crate::presentation::{OutputFormat, output_error, output_item, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_password: bool, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let bookmark = match crate::service::get_bookmark(&store, &name) {
        Ok(b) => b,
        Err(e) => {
            if format.is_json() {
                println!("{}", output_error(&e.to_string(), "NOT_FOUND", format));
            }
            return Err(e);
        }
    };

    if format.is_json() {
        let password = if show_password {
            storage::get_password(&name).ok().flatten()
        } else {
            None
        };

        #[derive(serde::Serialize)]
        struct GetOutput {
            name: String,
            url: String,
            account: Option<String>,
            password: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            name: bookmark.name.clone(),
            url: bookmark.url.clone(),
            account: bookmark.account.clone(),
            password,
            tags: bookmark.tags.clone(),
            remark: bookmark.remark.clone(),
            created_at: bookmark.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: bookmark.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

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
                println!(
                    "{:16} {}",
                    "Password:".style(style),
                    "(stored in keychain)".dimmed()
                );
            }
        }
        Ok(None) => {
            println!("{:16} {}", "Password:".style(style), "(not set)".dimmed());
        }
        Err(e) => {
            println!(
                "{:16} {}",
                "Password:".style(style),
                format!("(error: {e})").red()
            );
        }
    }

    if !bookmark.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            bookmark
                .tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !bookmark.remark.is_empty() {
        println!(
            "{:16} {}",
            "Remark:".style(style),
            bookmark.remark.join("; ").dimmed()
        );
    }

    println!(
        "{:16} {}",
        "Created:".style(style),
        bookmark
            .created_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        bookmark
            .updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    Ok(())
}
