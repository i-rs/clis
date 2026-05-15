use crate::models::Book;
use crate::presentation::{output_error, output_item, print_error, print_header, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct AddArgs {
    #[arg(help = "Book name")]
    pub name: String,

    #[arg(help = "Author name")]
    pub author: String,

    #[arg(help = "Total pages")]
    pub total_pages: u32,

    #[arg(short, long, help = "Tags (can be specified multiple times)")]
    pub tags: Vec<String>,

    #[arg(short, long, help = "Remarks (can be specified multiple times)")]
    pub remark: Vec<String>,
}

pub fn add(args: AddArgs, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.books.contains_key(&args.name) {
        let msg = format!("Book '{}' already exists", args.name);
        print_error(&msg);
        if matches!(output_format, OutputFormat::Json) {
            println!("{}", output_error(&msg, "ALREADY_EXISTS", output_format));
        }
        anyhow::bail!(msg);
    }

    let mut book = Book::new(args.name, args.author, args.total_pages);
    book.tags = args.tags;
    book.remark = args.remark;

    store.add_entry(book.clone());
    storage::save_store(&store)?;

    match output_format {
        OutputFormat::Json => {
            println!("{}", output_item(&book, output_format));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Book Added");
            print_success(&format!("Name: {}", book.name));
            println!("Author: {}", book.author);
            println!("Total Pages: {}", book.total_pages);
            println!("Status: {:?}", book.status);
        }
    }

    Ok(())
}
