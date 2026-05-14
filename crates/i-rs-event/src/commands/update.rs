use crate::models::EventType;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use i_rs_core::parse_datetime;

#[derive(Parser, Debug, Clone)]
pub struct UpdateArgs {
    #[arg(help = "Event name")]
    pub name: String,
    #[arg(short, long, help = "New event date (YYYY-MM-DD or YYYY-MM-DD HH:MM)")]
    pub date: Option<String>,
    #[arg(short = 'y', long, help = "New event type (meeting/gathering/course/other)")]
    pub event_type: Option<String>,
    #[arg(short = 'l', long, help = "New location")]
    pub location: Option<String>,
    #[arg(short = 'p', long, help = "New participants (comma separated)")]
    pub participants: Option<String>,
    #[arg(short = 'T', long, help = "New tags")]
    pub tag: Option<Vec<String>>,
    #[arg(short, long, help = "New remarks")]
    pub remark: Option<Vec<String>>,
}

pub fn run(args: &UpdateArgs, json: bool) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.events.contains_key(&args.name) {
        anyhow::bail!("Event '{}' not found", args.name);
    }

    let event_name = args.name.clone();
    {
        let event = store.events.get_mut(&event_name).expect("existence checked above");

        if let Some(ref date) = args.date {
            event.date = parse_datetime(date)?;
        }
        if let Some(ref et) = args.event_type {
            event.event_type = match et.to_lowercase().as_str() {
                "meeting" => EventType::Meeting,
                "gathering" => EventType::Gathering,
                "course" => EventType::Course,
                "other" => EventType::Other,
                _ => anyhow::bail!("Invalid event type: {et}"),
            };
        }
        if let Some(ref loc) = args.location {
            event.location = loc.clone();
        }
        if let Some(ref parts) = args.participants {
            event.participants = parts.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        }
        if let Some(ref t) = args.tag {
            event.tags = t.clone();
        }
        if let Some(ref r) = args.remark {
            event.remark = r.clone();
        }

        event.updated_at = Utc::now();
    }

    storage::save_store(&store)?;

    if json {
        println!("{{\"success\":true,\"data\":{{\"name\":\"{event_name}\"}}}}");
    } else {
        println!("✓ Event '{event_name}' updated successfully");
    }

    Ok(())
}
