use crate::models::{VocabStatus, VocabWord};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(
    word: String,
    definition: String,
    example: Vec<String>,
    status: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let word_lower = word.to_lowercase();
    if store.words.contains_key(&word_lower) {
        anyhow::bail!("Word '{}' already exists", word);
    }

    let vocab_status = match status.as_ref().map(|s| s.as_str()) {
        Some(s) => VocabStatus::from_str(s).unwrap_or(VocabStatus::New),
        None => VocabStatus::New,
    };

    let now = Utc::now();
    let vocab_word = VocabWord {
        word: word.clone(),
        definition,
        example,
        status: vocab_status,
        review_count: 0,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_word(vocab_word);
    storage::save_store(&store)?;

    print_success(&format!("✓ Word added: {}", word.green()));

    Ok(())
}
