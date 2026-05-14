use crate::models::{Project, ProjectStore};


i_rs_core::create_store!(ProjectStore, "project");

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
