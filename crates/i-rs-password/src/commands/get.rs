use crate::presentation::{output_error, output_item, print_error, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_password: bool, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entry = match storage::get_entry(&store, &name) {
        Some(e) => e,
        None => {
            let msg = format!("Entry '{}' not found", name);
            if matches!(format, OutputFormat::Json) {
                println!("{}", output_error(&msg, "NOT_FOUND", format));
            } else {
                print_error(&msg);
            }
            anyhow::bail!("{}", msg);
        }
    };

    if matches!(format, OutputFormat::Json) {
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
            name: entry.name.clone(),
            url: entry.url.clone(),
            account: entry.account.clone(),
            password,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

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