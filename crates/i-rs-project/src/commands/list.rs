use crate::models::ProjectListItem;
use crate::presentation::{
    OutputFormat, format_project_table, output_list, print_project_count, print_warning,
};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    tag: Option<String>,
    status: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let projects: Vec<&crate::models::Project> = store
        .projects
        .iter()
        .filter(|p| {
            let tag_match = tag
                .as_ref()
                .is_none_or(|t| p.tags.iter().any(|pt| pt.eq_ignore_ascii_case(t)));
            let status_match = status.as_ref().is_none_or(|s| {
                let status_str = format!("{:?}", p.status).to_lowercase();
                status_str.eq_ignore_ascii_case(s)
                    || (s == "onhold" && status_str == "onhold")
                    || (s == "on_hold" && status_str == "onhold")
            });
            tag_match && status_match
        })
        .collect();

    if projects.is_empty() {
        if format.is_json() {
            let filter = tag.clone().or(status.clone());
            println!(
                "{}",
                output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format)
            );
        } else {
            print_warning("No projects found.");
        }
        return Ok(());
    }

    if format.is_json() {
        let items: Vec<ProjectListItem> =
            projects.iter().map(|p| ProjectListItem::from(*p)).collect();
        let filter = tag.clone().or(status.clone());
        println!(
            "{}",
            output_list(&items, items.len(), filter.as_deref(), format)
        );
        return Ok(());
    }

    let table = format_project_table(&projects);
    println!("\n{table}");

    print_project_count(projects.len());

    Ok(())
}
