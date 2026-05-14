use crate::presentation::{print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_stats(format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if store.records.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", serde_json::json!({
                "success": true,
                "data": {
                    "total_records": 0,
                    "message": "No vision records found"
                }
            }));
        } else {
            print_warning("No vision records found.");
        }
        return Ok(());
    }

    let records: Vec<_> = store.records.values().collect();

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct StatsData {
            total_records: usize,
            earliest_date: Option<String>,
            latest_date: Option<String>,
            left_sphere_latest: Option<f64>,
            right_sphere_latest: Option<f64>,
            left_sphere_change: Option<f64>,
            right_sphere_change: Option<f64>,
            left_cylinder_latest: Option<f64>,
            right_cylinder_latest: Option<f64>,
        }

        let earliest = records.first();
        let latest = records.last();

        let left_change = if records.len() >= 2 {
            let mut earliest_with_left = None;
            let mut latest_with_left = None;
            for r in records.iter() {
                if r.left_sphere.is_some() {
                    if earliest_with_left.is_none() {
                        earliest_with_left = Some(r);
                    }
                    latest_with_left = Some(r);
                }
            }
            match (earliest_with_left, latest_with_left) {
                (Some(first), Some(last)) => {
                    if let (Some(f), Some(l)) = (first.left_sphere, last.left_sphere) {
                        Some(l - f)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        };

        let right_change = if records.len() >= 2 {
            let mut earliest_with_right = None;
            let mut latest_with_right = None;
            for r in records.iter() {
                if r.right_sphere.is_some() {
                    if earliest_with_right.is_none() {
                        earliest_with_right = Some(r);
                    }
                    latest_with_right = Some(r);
                }
            }
            match (earliest_with_right, latest_with_right) {
                (Some(first), Some(last)) => {
                    if let (Some(f), Some(l)) = (first.right_sphere, last.right_sphere) {
                        Some(l - f)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        };

        let stats = StatsData {
            total_records: records.len(),
            earliest_date: earliest.map(|r| r.date.format("%Y-%m-%d").to_string()),
            latest_date: latest.map(|r| r.date.format("%Y-%m-%d").to_string()),
            left_sphere_latest: latest.and_then(|r| r.left_sphere),
            right_sphere_latest: latest.and_then(|r| r.right_sphere),
            left_sphere_change: left_change,
            right_sphere_change: right_change,
            left_cylinder_latest: latest.and_then(|r| r.left_cylinder),
            right_cylinder_latest: latest.and_then(|r| r.right_cylinder),
        };

        println!("{}", serde_json::json!({
            "success": true,
            "data": stats
        }));
        return Ok(());
    }

    println!("\n{}", "Vision Statistics:".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());

    println!("  {:16} {}", "Total Records:".dimmed(), records.len().to_string().green());

    if let Some(earliest) = records.first() {
        println!("  {:16} {}", "Earliest Record:".dimmed(), earliest.date.format("%Y-%m-%d").green());
    }

    if let Some(latest) = records.last() {
        println!("  {:16} {}", "Latest Record:".dimmed(), latest.date.format("%Y-%m-%d").green());
    }

    println!();

    if let Some(latest) = records.last() {
        println!("{}", "Latest Vision:".bold().cyan());

        if let Some(ls) = latest.left_sphere {
            let sign = if ls >= 0.0 { "+" } else { "" };
            println!("  {:16} {}{:.2}", "Left Sphere:".dimmed(), sign, ls);
        }

        if let Some(rs) = latest.right_sphere {
            let sign = if rs >= 0.0 { "+" } else { "" };
            println!("  {:16} {}{:.2}", "Right Sphere:".dimmed(), sign, rs);
        }

        if let Some(lc) = latest.left_cylinder {
            let sign = if lc >= 0.0 { "+" } else { "" };
            println!("  {:16} {}{:.2}", "Left Cylinder:".dimmed(), sign, lc);
        }

        if let Some(rc) = latest.right_cylinder {
            let sign = if rc >= 0.0 { "+" } else { "" };
            println!("  {:16} {}{:.2}", "Right Cylinder:".dimmed(), sign, rc);
        }

        if let Some(la) = latest.left_axis {
            println!("  {:16} {}", "Left Axis:".dimmed(), la);
        }

        if let Some(ra) = latest.right_axis {
            println!("  {:16} {}", "Right Axis:".dimmed(), ra);
        }
    }

    if records.len() >= 2 {
        let mut earliest_with_left = None;
        let mut latest_with_left = None;
        let mut earliest_with_right = None;
        let mut latest_with_right = None;

        for r in records.iter() {
            if r.left_sphere.is_some() {
                if earliest_with_left.is_none() {
                    earliest_with_left = Some(r);
                }
                latest_with_left = Some(r);
            }
            if r.right_sphere.is_some() {
                if earliest_with_right.is_none() {
                    earliest_with_right = Some(r);
                }
                latest_with_right = Some(r);
            }
        }

        if let (Some(first), Some(last)) = (earliest_with_left, latest_with_left) {
            if let (Some(f), Some(l)) = (first.left_sphere, last.left_sphere) {
                let change = l - f;
                let sign = if change >= 0.0 { "+" } else { "" };
                let direction = if change.abs() < 0.25 {
                    "→".dimmed().to_string()
                } else if change > 0.0 {
                    "↓".red().to_string()
                } else {
                    "↑".green().to_string()
                };
                println!();
                println!("{}", "Left Eye Change:".bold().cyan());
                println!("  {:16} {}{:.2} ({}{:.2} {})", "".dimmed(), sign, change, sign, change.abs(), direction);
            }
        }

        if let (Some(first), Some(last)) = (earliest_with_right, latest_with_right) {
            if let (Some(f), Some(l)) = (first.right_sphere, last.right_sphere) {
                let change = l - f;
                let sign = if change >= 0.0 { "+" } else { "" };
                let direction = if change.abs() < 0.25 {
                    "→".dimmed().to_string()
                } else if change > 0.0 {
                    "↓".red().to_string()
                } else {
                    "↑".green().to_string()
                };
                println!();
                println!("{}", "Right Eye Change:".bold().cyan());
                println!("  {:16} {}{:.2} ({}{:.2} {})", "".dimmed(), sign, change, sign, change.abs(), direction);
            }
        }
    }

    println!();

    Ok(())
}
