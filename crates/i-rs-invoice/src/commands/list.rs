use crate::presentation::{OutputFormat, format_table, print_invoice_count, print_warning};
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    #[arg(short, long, help = "Filter by tag")]
    pub tag: Option<String>,

    #[arg(short, long, action, help = "Show only reimbursed invoices")]
    pub reimbursed: bool,

    #[arg(short, long, action, help = "Show only unreimbursed invoices")]
    pub unreimbursed: bool,

    #[arg(long, help = "Output format")]
    pub format: Option<OutputFormat>,
}

pub fn run_list(args: ListArgs) -> Result<()> {
    let store = storage::load_store()?;

    let filtered: Vec<&crate::models::Invoice> = if let Some(tag) = &args.tag {
        crate::models::filter_by_tag(&store, tag)
    } else if args.reimbursed {
        storage::filter_by_reimbursed(&store, true)
    } else if args.unreimbursed {
        storage::filter_by_reimbursed(&store, false)
    } else {
        storage::get_all_entries(&store)
    };

    let format = args.format.unwrap_or(OutputFormat::Default);

    if filtered.is_empty() {
        print_warning("No invoices found");
        return Ok(());
    }

    let mut sorted = filtered;
    sorted.sort_by_key(|e| std::cmp::Reverse(e.date));

    if format == OutputFormat::Json {
        println!(
            "{}",
            crate::presentation::output_list(
                &sorted,
                sorted.len(),
                args.tag.as_deref(),
                OutputFormat::Json
            )
        );
    } else {
        println!("{}", format_table(&sorted));
        print_invoice_count(sorted.len());
    }

    Ok(())
}
