use crate::models::ArticleDetail;
use crate::presentation::{output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let article = match storage::get_article(&store, &name) {
        Some(a) => a,
        None => {
            anyhow::bail!("Article '{}' not found", name);
        }
    };

    if matches!(format, OutputFormat::Json) {
        let detail = ArticleDetail::from(article);
        println!("{}", output_item(&detail, format));
        return Ok(());
    }

    println!();
    println!("{}", "📄 Article Details".bold().cyan());
    println!();
    println!("  {}: {}", "Name".dimmed(), article.name.cyan());
    println!("  {}: {}", "Title".dimmed(), article.title.cyan());
    println!("  {}: {}", "URL".dimmed(), article.url.green());
    println!("  {}: {}", "Source".dimmed(), article.source.green());
    println!("  {}: {}", "Status".dimmed(), article.status.to_string().cyan());
    
    if !article.tags.is_empty() {
        println!("  {}: {}", "Tags".dimmed(), article.tags.join(", ").cyan());
    }
    
    if !article.remark.is_empty() {
        println!("  {}:", "Remarks".dimmed());
        for remark in &article.remark {
            println!("    - {}", remark);
        }
    }
    
    if !article.notes.is_empty() {
        println!("  {}:", "Notes".dimmed());
        for note in &article.notes {
            println!("    - {}", note);
        }
    }
    
    if let Some(read_at) = article.read_at {
        println!("  {}: {}", "Read At".dimmed(), read_at.format("%Y-%m-%d %H:%M:%S").to_string().cyan());
    }
    
    println!();
    println!("  {}: {}", "Created".dimmed(), article.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("  {}: {}", "Updated".dimmed(), article.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!();

    Ok(())
}
