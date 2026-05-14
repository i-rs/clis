use crate::presentation::print_header;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use std::io::{self, Write};

pub fn handle_quiz(count: Option<usize>) -> Result<()> {
    let mut store = storage::load_store()?;
    let quiz_count = count.unwrap_or(5);

    let word_keys: Vec<String> = {
        let words = store.get_words_for_quiz(quiz_count);
        words.iter().map(|w| w.word.clone()).collect()
    };

    if word_keys.is_empty() {
        anyhow::bail!("No words available for quiz");
    }

    println!();
    print_header("Vocabulary Quiz");
    println!("\n{}", format!("Total words to review: {}", word_keys.len()).cyan());
    println!("Type 'q' to quit quiz\n");

    let mut correct = 0;
    let mut total = 0;

    for word_key in word_keys {
        let word = match store.get_entry(&word_key) {
            Some(w) => w,
            None => continue,
        };

        total += 1;
        println!("{}", "─".repeat(50).dimmed());
        println!("\n{}: {}", "Word".bold(), word.word.green());
        println!("\nEnter your answer (definition):");

        print!("  > ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input == "q" {
            println!("\n{}", "Quiz ended.".yellow());
            break;
        }

        let answer = input;
        let correct_answer = word.definition.to_lowercase();

        if answer == correct_answer {
            println!("  {}", "✓ Correct!".green());
            correct += 1;
        } else {
            println!("  {}", "✗ Incorrect".red());
            println!("  {}: {}", "Correct answer".dimmed(), word.definition.cyan());
        }

        if !word.example.is_empty() {
            println!("\n  {}:", "Example".dimmed());
            for ex in &word.example {
                println!("    • {ex}");
            }
        }

        if let Some(updated_word) = store.get_entry_mut(&word_key) {
            updated_word.review_count += 1;
            updated_word.updated_at = Utc::now();
            if updated_word.review_count >= 5 && updated_word.status == crate::models::VocabStatus::Learning {
                updated_word.status = crate::models::VocabStatus::Mastered;
                println!("  {}", "🎉 Status changed to Mastered!".yellow());
            } else if updated_word.review_count >= 2 && updated_word.status == crate::models::VocabStatus::New {
                updated_word.status = crate::models::VocabStatus::Learning;
                println!("  {}", "📖 Status changed to Learning!".yellow());
            }
        }
    }

    storage::save_store(&store)?;

    println!();
    println!("{}", "═".repeat(50).dimmed());
    print_header("Quiz Complete!");
    println!("  {} {}", "Correct:".dimmed(), format!("{correct}/{total}").green());
    if total > 0 {
        let percentage = (f64::from(correct) / f64::from(total)) * 100.0;
        println!("  {} {:.1}%", "Score:".dimmed(), percentage);
    }

    Ok(())
}
