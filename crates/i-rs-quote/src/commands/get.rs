use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(id: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let quote = if let Some(q) = store.get_entry(&id) { q } else {
        let msg = format!("Quote '{id}' not found");
        if format.is_json() {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        } 
        anyhow::bail!("{msg}");
    };

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct GetOutput {
            id: String,
            content: String,
            author: Option<String>,
            source: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
        }

        let output = GetOutput {
            id: quote.id.clone(),
            content: quote.content.clone(),
            author: quote.author.clone(),
            source: quote.source.clone(),
            tags: quote.tags.clone(),
            remark: quote.remark.clone(),
            created_at: quote.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Quote: {}", id.green().bold()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Content:".style(style), quote.content.cyan());
    println!();

    if let Some(ref author) = quote.author {
        println!("{:16} {}", "Author:".style(style), author.green());
    }

    if let Some(ref source) = quote.source {
        println!("{:16} {}", "Source:".style(style), source.yellow());
    }

    if !quote.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), quote.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !quote.remark.is_empty() {
        println!("\n{}:", "Remarks".bold());
        for line in &quote.remark {
            println!("  {}", line.dimmed());
        }
    }

    println!("\n{:16} {}", "Created:".style(style), quote.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
