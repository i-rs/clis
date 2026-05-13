use crate::presentation::{print_error, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_password: bool) -> Result<()> {
    let store = storage::load_store()?;

    let server = match storage::get_server(&store, &name) {
        Some(s) => s,
        None => {
            print_error(&format!("Server '{}' not found", name));
            anyhow::bail!("Server '{}' not found", name);
        }
    };

    print_header(&format!("Server: {}", server.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    println!(
        "{:16} {}",
        "Host:".style(style),
        server.host.cyan()
    );
    println!(
        "{:16} {}",
        "Port:".style(style),
        server.port.to_string().cyan()
    );

    if let Some(ref user) = server.user {
        println!(
            "{:16} {}",
            "User:".style(style),
            user.yellow()
        );
    }

    match storage::get_password(&name) {
        Ok(Some(pwd)) => {
            if show_password {
                println!(
                    "{:16} {}",
                    "Password:".style(style),
                    pwd.red()
                );
            } else {
                println!(
                    "{:16} {}",
                    "Password:".style(style),
                    "(stored in keychain)".dimmed().to_string()
                );
            }
        }
        Ok(None) => {
            println!(
                "{:16} {}",
                "Password:".style(style),
                "(not set)".dimmed().to_string()
            );
        }
        Err(e) => {
            println!(
                "{:16} {}",
                "Password:".style(style),
                format!("(error: {})", e).red()
            );
        }
    }

    if !server.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            server
                .tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !server.notes.is_empty() {
        println!(
            "{:16} {}",
            "Notes:".style(style),
            server.notes.join("; ").dimmed()
        );
    }

    println!(
        "\n{} Use '{}' for SSH command suggestions",
        "Tip:".dimmed(),
        "i-rs-server suggest".cyan()
    );

    Ok(())
}
