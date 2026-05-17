use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_date;

#[allow(clippy::too_many_arguments)]
pub fn handle_update(
    name: String,
    year: Option<i32>,
    director: Option<String>,
    rating: Option<f32>,
    review: Option<Vec<String>>,
    release_date: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
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

    let movie = match store.movies.get_mut(&name) {
        Some(m) => m,
        None => {
            anyhow::bail!("Movie '{name}' not found");
        }
    };

    if let Some(y) = year {
        movie.year = Some(y);
    }
    if let Some(d) = director {
        movie.director = Some(d);
    }
    if let Some(r) = rating {
        movie.rating = Some(r);
    }
    i_rs_core::update_field!(movie.review, review);
    if let Some(d) = release_date {
        movie.release_date = Some(d);
    }
    i_rs_core::update_field!(movie.tags, tag);
    i_rs_core::update_field!(movie.remark, remark);

    movie.updated_at = Utc::now();

    storage::save_store(&store)?;

    if matches!(output_format, OutputFormat::Json) {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "message": format!("Movie '{}' updated successfully", name)
            })
        );
    } else {
        print_success(&format!("✓ Movie '{name}' updated"));
    }

    Ok(())
}
