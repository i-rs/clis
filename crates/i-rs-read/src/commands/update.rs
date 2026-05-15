use crate::models::BookStatus;
use crate::presentation::{output_item, print_header, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct UpdateArgs {
    #[arg(help = "Book name")]
    pub name: String,

    #[arg(short, long, help = "Current page")]
    pub current_page: Option<u32>,

    #[arg(short, long, help = "Status (reading, completed, paused, dropped, to_read)")]
    pub status: Option<String>,

    #[arg(short, long, help = "Rating (0-5)")]
    pub rating: Option<f32>,

    #[arg(short, long, help = "Review")]
    pub review: Option<String>,

    #[arg(short = 't', long, help = "Tags (comma-separated)")]
    pub tags: Option<String>,

    #[arg(short, long, help = "Add remark")]
    pub add_remark: Option<String>,

    #[arg(long, help = "Remove remark by index (1-based)")]
    pub remove_remark: Option<usize>,
}

pub fn update(args: UpdateArgs, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    let book = if let Some(b) = storage::get_entry_mut(&args.name, &mut store) { b } else {
        let msg = format!("Book '{}' not found", args.name);
        anyhow::bail!(msg);
    };

    if let Some(current_page) = args.current_page {
        if current_page > book.total_pages {
            let msg = format!(
                "Current page ({}) cannot exceed total pages ({})",
                current_page, book.total_pages
            );
            anyhow::bail!(msg);
        }
        book.current_page = current_page;
    }

    if let Some(status_str) = args.status {
        let status_lower = status_str.to_lowercase();
        book.status = match status_lower.as_str() {
            "reading" => BookStatus::Reading,
            "completed" => {
                book.current_page = book.total_pages;
                BookStatus::Completed
            }
            "paused" => BookStatus::Paused,
            "dropped" => BookStatus::Dropped,
            "to_read" => BookStatus::ToRead,
            _ => {
                let msg = format!(
                    "Invalid status '{status_str}'. Valid options: reading, completed, paused, dropped, to_read"
                );
                anyhow::bail!(msg);
            }
        };
    }

    if let Some(rating) = args.rating {
        if !(0.0..=5.0).contains(&rating) {
            let msg = "Rating must be between 0 and 5";
            anyhow::bail!(msg);
        }
        book.rating = Some(rating);
    }

    if let Some(review) = args.review {
        book.review = review;
    }

    if let Some(tags_str) = args.tags {
        book.tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Some(remark) = args.add_remark {
        book.remark.push(remark);
    }

    if let Some(idx) = args.remove_remark {
        if idx == 0 || idx > book.remark.len() {
            let msg = format!(
                "Invalid remark index {}. Valid range: 1-{}",
                idx,
                book.remark.len()
            );
            anyhow::bail!(msg);
        }
        book.remark.remove(idx - 1);
    }

    book.updated_at = chrono::Utc::now();

    let book_name = book.name.clone();
    let current_page = book.current_page;
    let total_pages = book.total_pages;
    let status = book.status.clone();
    let book_json = output_item(book, output_format);

    storage::save_store(&store)?;

    match output_format {
        OutputFormat::Json => {
            println!("{book_json}");
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Book Updated");
            print_success(&format!("Book '{book_name}' has been updated"));
            println!("Current page: {current_page}/{total_pages}");
            println!("Status: {status:?}");
            println!("Progress: {:.1}%", (current_page as f32 / total_pages as f32) * 100.0);
        }
    }

    Ok(())
}
