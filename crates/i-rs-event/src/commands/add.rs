use crate::models::{Event, EventType};
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use i_rs_core::utils::validation::validate_name;

#[derive(Parser, Debug, Clone)]
pub struct AddArgs {
    #[arg(help = "Event name")]
    pub name: String,

    #[arg(short, long, help = "Event date (YYYY-MM-DD or YYYY-MM-DD HH:MM)")]
    pub date: String,

    #[arg(short, long, default_value = "other", help = "Event type (meeting, gathering, course, other)")]
    pub event_type: String,

    #[arg(short, long, help = "Event location")]
    pub location: Option<String>,

    #[arg(short = 'p', long, help = "Participants (comma-separated)")]
    pub participants: Option<String>,

    #[arg(short, long, help = "Tags (comma-separated)")]
    pub tags: Option<String>,

    #[arg(short, long, help = "Remarks (can be specified multiple times)")]
    pub remark: Vec<String>,
}

pub fn run(args: &AddArgs, json: bool) -> Result<()> {
    if let Err(e) = validate_name(&args.name) {
        anyhow::bail!("{}", e.message);
    }

    let event_type: EventType = args.event_type.parse().map_err(|e: String| {
        print_error(&e);
        anyhow::anyhow!("{}", e)
    })?;

    let date = parse_date(&args.date)?;

    let mut event = Event::new(args.name.clone(), date);
    event.event_type = event_type;

    if let Some(ref loc) = args.location {
        event.location = loc.clone();
    }

    if let Some(ref participants_str) = args.participants {
        event.participants = participants_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    if let Some(ref tags_str) = args.tags {
        event.tags = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    if !args.remark.is_empty() {
        event.remark = args.remark.clone();
    }

    let mut store = storage::load_store()?;

    if store.events.contains_key(&args.name) {
        anyhow::bail!("Event '{}' already exists", args.name);
    }

    storage::add_event(&mut store, event);
    storage::save_store(&store)?;

    if json {
        println!(
            r#"{{"success": true, "data": {{"name": "{}"}}}}"#,
            args.name
        );
    } else {
        print_success(&format!("Event '{}' added successfully", args.name));
    }

    Ok(())
}

fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    let formats = [
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
        "%Y/%m/%d %H:%M",
        "%Y/%m/%d",
    ];

    for fmt in &formats {
        if let Ok(dt) = DateTime::parse_from_str(date_str, fmt) {
            return Ok(dt.with_timezone(&Utc));
        }
    }

    Err(anyhow::anyhow!("Invalid date format: {}", date_str))
}
