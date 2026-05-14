use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use chrono::Utc;

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

    if let Some(r) = rating {
        if !(0.0..=10.0).contains(&r) {
            anyhow::bail!("Rating must be between 0 and 10");
        }
    }

    let mut store = storage::load_store()?;

    let movie = match store.movies.get_mut(&name) {
        Some(m) => m,
        None => {
            anyhow::bail!("Movie '{}' not found", name);
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
    if let Some(r) = review {
        movie.review = r;
    }
    if let Some(d) = release_date {
        movie.release_date = Some(d);
    }
    if let Some(t) = tag {
        movie.tags = t;
    }
    if let Some(r) = remark {
        movie.remark = r;
    }

    movie.updated_at = Utc::now();

    storage::save_store(&store)?;

    if matches!(output_format, OutputFormat::Json) {
        println!("{}", serde_json::json!({
            "success": true,
            "message": format!("Movie '{}' updated successfully", name)
        }));
    } else {
        print_success(&format!("✓ Movie '{}' updated", name));
    }

    Ok(())
}

