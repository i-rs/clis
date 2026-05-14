use crate::models::Movie;
use crate::presentation::{print_error, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::{NaiveDate, Utc};
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    year: Option<i32>,
    director: Option<String>,
    watched: bool,
    rating: Option<f32>,
    review: Vec<String>,
    release_date: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    let release_date = if let Some(date_str) = release_date {
        Some(parse_date(&date_str)?)
    } else {
        None
    };

    if let Some(r) = rating {
        if !(0.0..=10.0).contains(&r) {
            print_error("Rating must be between 0 and 10");
            anyhow::bail!("Rating must be between 0 and 10");
        }
    }

    let mut store = storage::load_store()?;

    if store.movies.contains_key(&name) {
        print_error(&format!("Movie '{}' already exists. Use update command instead.", name));
        anyhow::bail!("Movie '{}' already exists", name);
    }

    let now = Utc::now();
    let movie = Movie {
        name: name.clone(),
        year,
        director,
        watched,
        rating,
        review,
        release_date,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_movie(movie);
    storage::save_store(&store)?;

    if matches!(output_format, OutputFormat::Json) {
        println!("{}", serde_json::json!({
            "success": true,
            "message": format!("Movie '{}' added successfully", name)
        }));
    } else {
        print_success(&format!("✓ Movie added: {}", name.green()));
    }

    Ok(())
}

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
