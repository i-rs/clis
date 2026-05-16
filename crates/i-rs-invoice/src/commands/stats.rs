use crate::presentation::print_stats;
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct StatsArgs {
    #[arg(short, long, help = "Filter by tag")]
    pub tag: Option<String>,

    #[arg(short, long, action, help = "Show only reimbursed statistics")]
    pub reimbursed: bool,

    #[arg(short, long, action, help = "Show only unreimbursed statistics")]
    pub unreimbursed: bool,
}

pub fn run_stats(args: StatsArgs) -> Result<()> {
    let store = storage::load_store()?;

    let invoices: Vec<&crate::models::Invoice> = if let Some(tag) = &args.tag {
        crate::models::filter_by_tag(&store, tag)
    } else if args.reimbursed {
        storage::filter_by_reimbursed(&store, true)
    } else if args.unreimbursed {
        storage::filter_by_reimbursed(&store, false)
    } else {
        storage::get_all_entries(&store)
    };

    let total: f64 = invoices.iter().map(|inv| inv.amount).sum();
    let reimbursed: f64 = invoices
        .iter()
        .filter(|inv| inv.reimbursed)
        .map(|inv| inv.amount)
        .sum();
    let unreimbursed = total - reimbursed;
    let count = invoices.len();
    let reimbursed_count = invoices.iter().filter(|inv| inv.reimbursed).count();

    print_stats(total, reimbursed, unreimbursed, count, reimbursed_count);

    Ok(())
}
