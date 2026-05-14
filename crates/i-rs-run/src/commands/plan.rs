use crate::models::RunPlan;
use crate::presentation::{
    format_plan_table, output_list, print_error, print_plan_count, print_success, print_warning,
    OutputFormat,
};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_plan_add(
    name: String,
    target_distance: f64,
    target_pace: String,
    schedule: Vec<u8>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let plan = RunPlan {
        id: Uuid::new_v4().to_string(),
        name,
        target_distance_km: target_distance,
        target_pace,
        schedule_days: schedule,
        tags,
        remark,
        created_at: Utc::now(),
    };

    let mut store = storage::load_store()?;
    store.add_plan(plan);
    storage::save_store(&store)?;

    print_success("✓ Run plan added");

    Ok(())
}

pub fn handle_plan_list(format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let plans: Vec<RunPlan> = store.plans.values().cloned().collect();

    if plans.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, None, format));
        } else {
            print_warning("No run plans found.");
        }
        return Ok(());
    }

    let plans_ref: Vec<&RunPlan> = plans.iter().collect();

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct PlanItem {
            id: String,
            name: String,
            target_distance_km: f64,
            target_pace: String,
            schedule_days: Vec<u8>,
            tags: Vec<String>,
        }

        let items: Vec<PlanItem> = plans
            .iter()
            .map(|p| PlanItem {
                id: p.id.clone(),
                name: p.name.clone(),
                target_distance_km: p.target_distance_km,
                target_pace: p.target_pace.clone(),
                schedule_days: p.schedule_days.clone(),
                tags: p.tags.clone(),
            })
            .collect();

        println!("{}", output_list(&items, items.len(), None, format));
        return Ok(());
    }

    let table = format_plan_table(&plans_ref);
    println!("\n{}", table);

    print_plan_count(plans_ref.len());

    Ok(())
}

pub fn handle_plan_get(id: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(plan) = store.get_plan(&id) {
        if matches!(format, OutputFormat::Json) {
            #[derive(serde::Serialize, Clone)]
            struct PlanItem {
                id: String,
                name: String,
                target_distance_km: f64,
                target_pace: String,
                schedule_days: Vec<u8>,
                tags: Vec<String>,
                remark: Vec<String>,
                created_at: String,
            }

            let item = PlanItem {
                id: plan.id.clone(),
                name: plan.name.clone(),
                target_distance_km: plan.target_distance_km,
                target_pace: plan.target_pace.clone(),
                schedule_days: plan.schedule_days.clone(),
                tags: plan.tags.clone(),
                remark: plan.remark.clone(),
                created_at: plan.created_at.to_rfc3339(),
            };

            println!("{}", output_list(&[item], 1, None, format));
        } else {
            println!("\n{}", "Run Plan Details:".bold().cyan());
            println!("  {:12} {}", "ID:".dimmed(), plan.id);
            println!("  {:12} {}", "Name:".dimmed(), plan.name);
            println!(
                "  {:12} {} km",
                "Target:".dimmed(),
                format!("{:.2}", plan.target_distance_km)
            );
            println!("  {:12} {}/km", "Pace:".dimmed(), plan.target_pace);
            println!("  {:12} {:?}", "Schedule:".dimmed(), plan.schedule_days);
            if !plan.tags.is_empty() {
                println!("  {:12} {}", "Tags:".dimmed(), plan.tags.join(", "));
            }
            if !plan.remark.is_empty() {
                println!("  {:12} {}", "Remark:".dimmed(), plan.remark.join(", "));
            }
        }
    } else {
        if matches!(format, OutputFormat::Json) {
            println!(
                "{}",
                output_list::<serde_json::Value>(&[], 0, Some(&format!("ID: {}", id)), format)
            );
        } else {
            print_error(&format!("No plan found with ID: {}", id));
        }
        anyhow::bail!("No plan found with ID: {}", id);
    }

    Ok(())
}

pub fn handle_plan_delete(id: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.remove_plan(&id).is_none() {
        print_error(&format!("No plan found with ID: {}", id));
        anyhow::bail!("No plan found with ID: {}", id);
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Plan {} deleted", id.green()));

    Ok(())
}
