use crate::presentation::{print_error, output_error, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let movie = match store.movies.get(&name) {
        Some(m) => m,
        None => {
            if matches!(output_format, OutputFormat::Json) {
                println!("{}", output_error(&format!("Movie '{}' not found", name), "NOT_FOUND", output_format));
            } else {
                print_error(&format!("Movie '{}' not found", name));
            }
            anyhow::bail!("Movie '{}' not found", name);
        }
    };

    if matches!(output_format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct MovieOutput<'a> {
            name: &'a str,
            year: Option<i32>,
            director: Option<&'a str>,
            watched: bool,
            rating: Option<f32>,
            review: Vec<&'a str>,
            release_date: Option<String>,
            tags: Vec<&'a str>,
            remark: Vec<&'a str>,
            created_at: i64,
            updated_at: i64,
        }

        let output = MovieOutput {
            name: &movie.name,
            year: movie.year,
            director: movie.director.as_deref(),
            watched: movie.watched,
            rating: movie.rating,
            review: movie.review.iter().map(|s| s.as_str()).collect(),
            release_date: movie.release_date.map(|d| d.format("%Y-%m-%d").to_string()),
            tags: movie.tags.iter().map(|s| s.as_str()).collect(),
            remark: movie.remark.iter().map(|s| s.as_str()).collect(),
            created_at: movie.created_at.timestamp(),
            updated_at: movie.updated_at.timestamp(),
        };

        println!("{}", output_item(&output, output_format));
        return Ok(());
    }

    println!();
    println!("{}", "┌──────────────────────────────────────────────".dimmed());
    println!("{} {}", "│".dimmed(), movie.name.bold().cyan());
    println!("{}", "├──────────────────────────────────────────────".dimmed());

    if let Some(year) = movie.year {
        println!("{} {:12} {}", "│".dimmed(), "Year:".dimmed(), year);
    }

    if let Some(ref director) = movie.director {
        println!("{} {:12} {}", "│".dimmed(), "Director:".dimmed(), director);
    }

    println!(
        "{} {:12} {}",
        "│".dimmed(),
        "Watched:".dimmed(),
        if movie.watched { "✓" } else { "✗" }
    );

    if let Some(rating) = movie.rating {
        println!(
            "{} {:12} {:.1}/10",
            "│".dimmed(),
            "Rating:".dimmed(),
            rating
        );
    }

    if !movie.tags.is_empty() {
        println!(
            "{} {:12} {}",
            "│".dimmed(),
            "Tags:".dimmed(),
            movie.tags.join(", ")
        );
    }

    if !movie.review.is_empty() {
        println!("{} {:12}", "│".dimmed(), "Review:".dimmed());
        for line in &movie.review {
            println!("{}  {}", "│".dimmed(), line);
        }
    }

    if !movie.remark.is_empty() {
        println!("{} {:12}", "│".dimmed(), "Remark:".dimmed());
        for line in &movie.remark {
            println!("{}  {}", "│".dimmed(), line);
        }
    }

    if let Some(date) = movie.release_date {
        println!(
            "{} {:12} {}",
            "│".dimmed(),
            "Release:".dimmed(),
            date.format("%Y-%m-%d")
        );
    }

    println!(
        "{} {:12} {}",
        "│".dimmed(),
        "Added:".dimmed(),
        movie.created_at.format("%Y-%m-%d %H:%M")
    );

    println!("{}", "└──────────────────────────────────────────────".dimmed());

    Ok(())
}
