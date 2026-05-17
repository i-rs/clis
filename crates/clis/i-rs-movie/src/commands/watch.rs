use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_watch(
    name: String,
    rating: Option<f32>,
    review: Option<Vec<String>>,
    output_format: OutputFormat,
) -> Result<()> {
    if let Some(r) = rating
        && !(0.0..=10.0).contains(&r)
    {
        anyhow::bail!("Rating must be between 0 and 10");
    }

    let mut store = storage::load_store()?;

    let movie = match store.movies.get_mut(&name) {
        Some(m) => m,
        None => {
            anyhow::bail!("Movie '{name}' not found");
        }
    };

    movie.watched = true;
    movie.updated_at = Utc::now();

    if let Some(r) = rating {
        movie.rating = Some(r);
    }
    if let Some(r) = review {
        movie.review = r;
    }

    storage::save_store(&store)?;

    if matches!(output_format, OutputFormat::Json) {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "message": format!("Movie '{}' marked as watched", name)
            })
        );
    } else {
        let rating_str = if let Some(r) = rating {
            format!(" (Rating: {r:.1}/10)")
        } else {
            String::new()
        };
        print_success(&format!(
            "✓ Movie '{}' marked as watched{}",
            name.green(),
            rating_str
        ));
    }

    Ok(())
}
