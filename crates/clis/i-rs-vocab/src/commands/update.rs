use crate::models::VocabStatus;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    word_key: String,
    definition: Option<String>,
    example: Option<Vec<String>>,
    status: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    review: bool,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let word_lower = word_key.to_lowercase();

    let word_name = {
        match store.get_entry(&word_lower) {
            Some(v) => v.word.clone(),
            None => {
                anyhow::bail!("Word '{word_key}' not found");
            }
        }
    };

    let (updated_word_name, updated_review_count) = {
        let vocab = match store.get_entry_mut(&word_lower) {
            Some(v) => v,
            None => {
                anyhow::bail!("Word '{word_key}' not found");
            }
        };

        i_rs_core::update_field!(vocab.definition, definition);
        i_rs_core::update_field!(vocab.example, example);
        if let Some(status_str) = status {
            match VocabStatus::parse_str(&status_str) {
                Some(s) => vocab.status = s,
                None => {
                    anyhow::bail!("Invalid status '{status_str}'");
                }
            }
        }
        i_rs_core::update_field!(vocab.tags, tag);
        i_rs_core::update_field!(vocab.remark, remark);
        if review {
            vocab.review_count += 1;
            if vocab.review_count >= 5 && vocab.status == VocabStatus::Learning {
                vocab.status = VocabStatus::Mastered;
            } else if vocab.review_count >= 2 && vocab.status == VocabStatus::New {
                vocab.status = VocabStatus::Learning;
            }
        }

        vocab.updated_at = Utc::now();
        (vocab.word.clone(), vocab.review_count)
    };

    storage::save_store(&store)?;

    if review {
        print_success(&format!(
            "✓ Review recorded for '{}' (count: {})",
            updated_word_name.green(),
            updated_review_count
        ));
    } else {
        print_success(&format!("✓ Word '{}' updated", word_name.green()));
    }

    Ok(())
}
