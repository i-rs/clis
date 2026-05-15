use crate::presentation::{output_error, output_item, print_error, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct GetArgs {
    #[arg(help = "Book name")]
    pub name: String,
}

pub fn get(args: GetArgs, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(book) = store.get_entry(&args.name) {
        match output_format {
            OutputFormat::Json => {
                println!("{}", output_item(book, output_format));
            }
            OutputFormat::Table | OutputFormat::Default => {
                print_header(&format!("Book: {}", book.name));
                println!("Author: {}", book.author);
                println!("Pages: {}/{}", book.current_page, book.total_pages);
                println!("Status: {:?}", book.status);
                println!("Progress: {:.1}%", book.progress_percentage());
                if let Some(rating) = book.rating {
                    println!("Rating: {rating:.1}");
                }
                if !book.review.is_empty() {
                    println!("Review: {}", book.review);
                }
                if !book.tags.is_empty() {
                    println!("Tags: {}", book.tags.join(", "));
                }
                if !book.remark.is_empty() {
                    println!("Remarks:");
                    for r in &book.remark {
                        println!("  - {r}");
                    }
                }
            }
        }
    } else {
        let msg = format!("Book '{}' not found", args.name);
        print_error(&msg);
        if matches!(output_format, OutputFormat::Json) {
            println!("{}", output_error(&msg, "NOT_FOUND", output_format));
        }
        anyhow::bail!(msg);
    }

    Ok(())
}
