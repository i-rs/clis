use crate::presentation::{print_error, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_password: bool) -> Result<()> {
    let store = storage::load_store()?;

    let domain = match storage::get_domain(&store, &name) {
        Some(d) => d,
        None => {
            print_error(&format!("Domain '{}' not found", name));
            anyhow::bail!("Domain '{}' not found", name);
        }
    };

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
        format!("{} days left", days)
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
