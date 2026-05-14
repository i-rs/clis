use crate::models::{VocabRow, VocabStats, VocabWord};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};
pub fn format_table(words: &[&VocabWord]) -> String {
    let rows: Vec<VocabRow> = words
        .iter()
        .map(|w| VocabRow::from_word(w))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_word_count(count: usize) {
    println!("\n{} {} words", "Total:".dimmed(), count.to_string().cyan());
}
pub fn print_stats(stats: &VocabStats) {
    println!("\n{}", "Statistics:".bold().cyan());
    println!("  {:12} {}", "Total:".dimmed(), stats.total.to_string().green());
    println!("  {:12} {} (🆕)", "New:".dimmed(), stats.new_count.to_string().yellow());
    println!("  {:12} {} (📖)", "Learning:".dimmed(), stats.learning_count.to_string().blue());
    println!("  {:12} {} (✅)", "Mastered:".dimmed(), stats.mastered_count.to_string().green());
    println!("  {:12} {}", "Reviews:".dimmed(), stats.total_reviews.to_string().magenta());
}
