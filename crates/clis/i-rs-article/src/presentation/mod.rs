use crate::models::{Article, ArticleRow};
use crate::storage::ArticleStats;
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_success};
use owo_colors::OwoColorize;
pub fn format_table(articles: &[&Article]) -> String {
    let rows: Vec<ArticleRow> = articles
        .iter()
        .map(|a| ArticleRow::from_article(a))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_article_count(count: usize) {
    println!(
        "\n{} {} articles",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}
pub fn print_stats(stats: &ArticleStats) {
    println!();
    println!("{}", "📊 Article Statistics".bold().cyan());
    println!();
    println!("  {}: {}", "Total".dimmed(), stats.total.to_string().cyan());
    println!(
        "  {}: {}",
        "Unread".dimmed(),
        stats.unread.to_string().cyan()
    );
    println!(
        "  {}: {}",
        "Reading".dimmed(),
        stats.reading.to_string().cyan()
    );
    println!("  {}: {}", "Read".dimmed(), stats.read.to_string().cyan());
    println!();
}
