use crate::presentation::print_header;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_stats() -> Result<()> {
    let store = storage::load_store()?;
    let stats = store.get_stats();

    println!();
    print_header("Vocabulary Statistics");
    println!();

    println!("{}", "Overall:".bold().cyan());
    println!("  {:12} {}", "Total words:".dimmed(), stats.total.to_string().green());
    println!("  {:12} {}", "Total reviews:".dimmed(), stats.total_reviews.to_string().magenta());

    println!();
    println!("{}", "By Status:".bold().cyan());
    println!("  {} {} (🆕)", "New:".dimmed().to_string(), stats.new_count.to_string().yellow());
    println!("  {} {} (📖)", "Learning:".dimmed().to_string(), stats.learning_count.to_string().blue());
    println!("  {} {} (✅)", "Mastered:".dimmed().to_string(), stats.mastered_count.to_string().green());

    if stats.total > 0 {
        println!();
        println!("{}", "Progress:".bold().cyan());
        let mastered_pct = (stats.mastered_count as f64 / stats.total as f64) * 100.0;
        let bar_width = 30;
        let filled = (mastered_pct / 100.0 * bar_width as f64) as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
        println!("  [{}] {:.1}% mastered", bar, mastered_pct);
    }

    Ok(())
}
