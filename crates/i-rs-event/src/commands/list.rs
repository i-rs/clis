use crate::models::Event;
use crate::presentation::{format_events_table, print_event_count, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;
use i_rs_core::presentation::output::output_list;

#[derive(Parser, Debug, Clone)]
pub struct ListArgs {
    #[arg(short, long, help = "Filter by tag")]
    pub tag: Option<String>,

    #[arg(short, long, help = "Filter by event type")]
    pub event_type: Option<String>,
}

pub fn run(args: &ListArgs, json: bool) -> Result<()> {
    let store = storage::load_store()?;

    let mut events: Vec<&Event> = store.events.values().collect();

    if let Some(ref tag) = args.tag {
        events.retain(|e| e.tags.iter().any(|t| t == tag));
    }

    if let Some(ref et) = args.event_type {
        events.retain(|e| e.event_type.to_string() == *et);
    }

    events.sort_by_key(|b| std::cmp::Reverse(b.date));

    let event_refs: Vec<&Event> = events;
    let count = event_refs.len();
    let filter = args.tag.clone();
    let format = if json { OutputFormat::Json } else { OutputFormat::Table };

    if json {
        println!("{}", output_list(&event_refs, count, filter.as_deref(), format));
    } else {
        print_header("Events");
        if !event_refs.is_empty() {
            println!("{}", format_events_table(&event_refs));
        }
        print_event_count(count);
    }

    Ok(())
}
