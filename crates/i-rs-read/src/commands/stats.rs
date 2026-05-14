use crate::presentation::{print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct StatsArgs {
    #[arg(short, long, help = "Filter by tag")]
    pub tag: Option<String>,
}

pub fn stats(_args: StatsArgs, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let books: Vec<&crate::models::Book> = storage::list_books(&store);

    let total_books = books.len();
    let total_pages: u32 = books.iter().map(|b| b.total_pages).sum();
    let total_read_pages: u32 = books.iter().map(|b| b.current_page).sum();

    let reading_count = books
        .iter()
        .filter(|b| b.status == crate::models::BookStatus::Reading)
        .count();
    let completed_count = books
        .iter()
        .filter(|b| b.status == crate::models::BookStatus::Completed)
        .count();
    let to_read_count = books
        .iter()
        .filter(|b| b.status == crate::models::BookStatus::ToRead)
        .count();
    let paused_count = books
        .iter()
        .filter(|b| b.status == crate::models::BookStatus::Paused)
        .count();
    let dropped_count = books
        .iter()
        .filter(|b| b.status == crate::models::BookStatus::Dropped)
        .count();

    let rated_books: Vec<&&crate::models::Book> =
        books.iter().filter(|b| b.rating.is_some()).collect();
    let avg_rating: f32 = if !rated_books.is_empty() {
        rated_books.iter().map(|b| b.rating.expect("filtered by rating.is_some() above")).sum::<f32>()
            / rated_books.len() as f32
    } else {
        0.0
    };

    match output_format {
        OutputFormat::Json => {
            let stats_json = serde_json::json!({
                "total_books": total_books,
                "total_pages": total_pages,
                "pages_read": total_read_pages,
                "reading": reading_count,
                "completed": completed_count,
                "to_read": to_read_count,
                "paused": paused_count,
                "dropped": dropped_count,
                "average_rating": avg_rating,
                "rated_books": rated_books.len()
            });
            println!("{}", serde_json::to_string_pretty(&stats_json)?);
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Reading Statistics");

            println!("\n📚 Overall:");
            println!("  Total books: {}", total_books);
            println!("  Total pages: {}", total_pages);
            println!("  Pages read: {}/{} ({:.1}%)",
                total_read_pages,
                total_pages,
                if total_pages > 0 { (total_read_pages as f32 / total_pages as f32) * 100.0 } else { 0.0 }
            );

            println!("\n📖 Status:");
            println!("  Reading: {}", reading_count);
            println!("  Completed: {}", completed_count);
            println!("  To Read: {}", to_read_count);
            println!("  Paused: {}", paused_count);
            println!("  Dropped: {}", dropped_count);

            println!("\n⭐ Rating:");
            println!("  Rated books: {}", rated_books.len());
            if rated_books.len() > 0 {
                println!("  Average rating: {:.2}", avg_rating);
            }
        }
    }

    Ok(())
}
