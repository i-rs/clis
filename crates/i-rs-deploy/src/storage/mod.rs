use crate::models::{DeployRecord, DeployStore};

i_rs_core::create_store!(DeployStore, "deploy");

pub fn filter_by_project<'a>(store: &'a DeployStore, project: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match project {
        Some(p) => store
            .entries
            .values()
            .filter(|e| e.project == p)
            .collect(),
        None => store.entries.values().collect(),
    }
}

pub fn filter_by_environment<'a>(store: &'a DeployStore, env: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match env {
        Some(e) => store
            .entries
            .values()
            .filter(|r| r.environment == e)
            .collect(),
        None => store.entries.values().collect(),
    }
}

pub fn filter_by_tag<'a>(store: &'a DeployStore, tag: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
