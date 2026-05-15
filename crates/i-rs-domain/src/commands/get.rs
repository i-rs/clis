use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_password: bool, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let domain = if let Some(d) = store.get_entry(&name) { d } else {
        let msg = format!("Domain '{name}' not found");
        if format.is_json() {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        } 
        anyhow::bail!("{msg}");
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
            expiry_date: String,
            days_until_expiry: i64,
            is_expired: bool,
            registrar: Option<String>,
            password: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            name: domain.name.clone(),
            expiry_date: domain.expiry_date.format("%Y-%m-%d").to_string(),
            days_until_expiry: domain.days_until_expiry(),
            is_expired: domain.is_expired(),
            registrar: domain.registrar.clone(),
            password,
            tags: domain.tags.clone(),
            remark: domain.remark.clone(),
            created_at: domain.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: domain.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Domain: {}", domain.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    let days = domain.days_until_expiry();

    println!("{:16} {}", "Domain:".style(style), domain.name.cyan());
    println!("{:16} {}", "Expiry Date:".style(style), domain.expiry_date.format("%Y-%m-%d").to_string().cyan());

    let status = if domain.is_expired() {
        format!("{} (expired {} days ago)", "EXPIRED".red(), days.abs())
    } else if days <= 30 {
        format!("{} ({} days left)", "EXPIRING SOON".yellow().bold(), days)
    } else {
        format!("{days} days left")
    };
    println!("{:16} {}", "Status:".style(style), status);

    if let Some(ref registrar) = domain.registrar {
        println!("{:16} {}", "Registrar:".style(style), registrar.yellow());
    }

    match storage::get_password(&name) {
        Ok(Some(pwd)) => {
            if show_password {
                println!("{:16} {}", "Password:".style(style), pwd.red());
            } else {
                println!("{:16} {}", "Password:".style(style), "(stored in keychain)".dimmed());
            }
        }
        Ok(None) => {
            println!("{:16} {}", "Password:".style(style), "(not set)".dimmed());
        }
        Err(e) => {
            println!("{:16} {}", "Password:".style(style), format!("(error: {e})").red());
        }
    }

    if !domain.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), domain.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !domain.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), domain.remark.join("; ").dimmed());
    }

    println!("{:16} {}", "Created:".style(style), domain.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), domain.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}