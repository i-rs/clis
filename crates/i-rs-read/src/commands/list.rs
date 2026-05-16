use crate::presentation::{
    OutputFormat, format_table, output_list, print_book_count, print_header,
};
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(short, long, help = "Filter by tag")]
    pub tag: Option<String>,

    #[arg(short, long, help = "Filter by status")]
    pub status: Option<String>,
}

pub fn list(args: ListArgs, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let books: Vec<&crate::models::Book> = if let Some(ref tag) = args.tag {
        storage::filter_by_tag(tag, &store)
    } else {
        storage::list_entries(&store)
    };

    let filtered_books: Vec<&crate::models::Book> = if let Some(ref status) = args.status {
        let status_lower = status.to_lowercase();
        books
            .into_iter()
            .filter(|b| format!("{:?}", b.status).to_lowercase() == status_lower)
            .collect()
    } else {
        books
    };

    match output_format {
        OutputFormat::Json => {
            let data: Vec<&crate::models::Book> = filtered_books.clone();
            println!(
                "{}",
                output_list(&data, data.len(), args.tag.as_deref(), output_format)
            );
        }
        OutputFormat::Table | OutputFormat::Default => {
            if filtered_books.is_empty() {
                print_header("No Books Found");
                print_book_count(0);
            } else {
                print_header("Books");
                let table = format_table(&filtered_books);
                println!("{table}");
                print_book_count(filtered_books.len());
            }
        }
    }

    Ok(())
}
