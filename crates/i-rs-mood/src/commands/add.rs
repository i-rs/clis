use crate::models::{Mood, MoodRecord};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_add(
    date: String,
    mood: String,
    tag: Vec<String>,
    content: Vec<String>,
) -> Result<()> {
    let date = parse_date(&date)?;

    let mood_level = parse_mood(&mood)?;
    let mood_obj = Mood::from_level(mood_level).expect("parse_mood validates level is 1-5");

    let mut store = storage::load_store()?;

    if store.records.contains_key(&date) {
        anyhow::bail!("Record for {} already exists", date);
    }

    let now = chrono::Utc::now();
    let record = MoodRecord {
        date,
        mood: mood_obj,
        tags: tag,
        content,
        created_at: now,
        updated_at: now,
    };

    store.add_record(record);
    storage::save_store(&store)?;

    print_success(&format!("✓ Mood record added: {} {}", mood_obj, mood_obj.label().green()));

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
