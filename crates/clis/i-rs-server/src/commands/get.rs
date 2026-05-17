use crate::presentation::{OutputFormat, output_error, output_item, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_password: bool, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let server = if let Some(s) = store.get_entry(&name) {
        s
    } else {
        let msg = format!("Server '{name}' not found");
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
            host: String,
            port: u16,
            user: Option<String>,
            password: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            name: server.name.clone(),
            host: server.host.clone(),
            port: server.port,
            user: server.user.clone(),
            password,
            tags: server.tags.clone(),
            remark: server.remark.clone(),
            created_at: server.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: server.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Server: {}", server.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    println!("{:16} {}", "Host:".style(style), server.host.cyan());
    println!(
        "{:16} {}",
        "Port:".style(style),
        server.port.to_string().cyan()
    );

    if let Some(ref user) = server.user {
        println!("{:16} {}", "User:".style(style), user.yellow());
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

    if !server.remark.is_empty() {
        println!(
            "{:16} {}",
            "Remark:".style(style),
            server.remark.join("; ").dimmed()
        );
    }

    println!(
        "{:16} {}",
        "Created:".style(style),
        server
            .created_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        server
            .updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    println!(
        "\n{} Use '{}' for SSH command suggestions",
        "Tip:".dimmed(),
        "i-rs-server suggest".cyan()
    );

    Ok(())
}
