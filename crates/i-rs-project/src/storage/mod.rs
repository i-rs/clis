use crate::models::{Project, ProjectStore};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("i-rs");
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
    }
    
    Ok(config_dir)
}

fn get_data_file_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("project.json"))
}

pub fn load_store() -> Result<ProjectStore> {
    let path = get_data_file_path()?;
    
    if !path.exists() {
        return Ok(ProjectStore::default());
    }
    
    let content = fs::read_to_string(&path)
        .context("Failed to read project data file")?;
    
    let store: ProjectStore = serde_json::from_str(&content)
        .context("Failed to parse project data")?;
    
    Ok(store)
}

pub fn save_store(store: &ProjectStore) -> Result<()> {
    let path = get_data_file_path()?;
    
    let content = serde_json::to_string_pretty(store)
        .context("Failed to serialize project data")?;
    
    fs::write(&path, content)
        .context("Failed to write project data file")?;
    
    Ok(())
}

pub fn find_project<'a>(store: &'a ProjectStore, name: &str) -> Option<&'a Project> {
    store.projects.iter().find(|p| p.name.eq_ignore_ascii_case(name))
}

pub fn find_project_mut<'a>(store: &'a mut ProjectStore, name: &str) -> Option<&'a mut Project> {
    store.projects.iter_mut().find(|p| p.name.eq_ignore_ascii_case(name))
}

pub fn project_exists(store: &ProjectStore, name: &str) -> bool {
    find_project(store, name).is_some()
}

pub fn add_project(store: &mut ProjectStore, project: Project) {
    store.projects.push(project);
}

pub fn remove_project<'a>(store: &'a mut ProjectStore, name: &str) -> Option<Project> {
    let index = store.projects.iter().position(|p| p.name.eq_ignore_ascii_case(name))?;
    Some(store.projects.remove(index))
}
