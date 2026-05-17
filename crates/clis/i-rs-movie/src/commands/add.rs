use crate::models::Movie;
use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

#[allow(clippy::too_many_arguments)]
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

    if let Some(r) = rating
        && !(0.0..=10.0).contains(&r)
    {
        anyhow::bail!("Rating must be between 0 and 10");
    }

    let mut store = storage::load_store()?;

    if store.movies.contains_key(&name) {
        anyhow::bail!("Movie '{name}' already exists");
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

    store.add_entry(movie);
    storage::save_store(&store)?;

    if matches!(output_format, OutputFormat::Json) {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "message": format!("Movie '{}' added successfully", name)
            })
        );
    } else {
        print_success(&format!("✓ Movie added: {}", name.green()));
    }

    Ok(())
}
