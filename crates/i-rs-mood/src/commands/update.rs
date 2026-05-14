use crate::models::Mood;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    date: String,
    mood: Option<String>,
    tag: Option<Vec<String>>,
    content: Option<Vec<String>>,
) -> Result<()> {
    let date = parse_date(&date)?;

    let mut store = storage::load_store()?;

    let record = match store.records.get_mut(&date) {
        Some(r) => r,
        None => {
            anyhow::bail!("No record found for {}", date);
        }
    };

    if let Some(mood_str) = mood {
        let mood_level = parse_mood(&mood_str)?;
        record.mood = Mood::from_level(mood_level).unwrap();
    }
    if let Some(tags) = tag {
        record.tags = tags;
    }
    if let Some(c) = content {
        record.content = c;
    }

    record.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Record for {} updated", date.green()));

    Ok(())
}


fn parse_mood(mood_str: &str) -> Result<u8> {
    let mood_lower = mood_str.to_lowercase();

    match mood_lower.as_str() {
        "great" | "5" | "😊" => Ok(5),
        "good" | "4" | "🙂" => Ok(4),
        "okay" | "ok" | "3" | "😐" => Ok(3),
        "bad" | "2" | "😔" => Ok(2),
        "terrible" | "1" | "😢" => Ok(1),
        _ => {
            if let Ok(num) = mood_str.parse::<u8>() {
                if num >= 1 && num <= 5 {
                    return Ok(num);
                }
            }
            Err(anyhow::anyhow!("Invalid mood: {}. Use 1-5, great/good/okay/bad/terrible, or emoji", mood_str))
        }
    }
}
