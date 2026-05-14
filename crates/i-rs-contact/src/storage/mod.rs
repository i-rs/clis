use crate::models::{Contact, ContactStore};
use anyhow::{Context, Result};
use chrono::Utc;
use dirs::config_dir;
use serde_json;
use std::fs;
use std::path::PathBuf;

fn get_store_path() -> Result<PathBuf> {
    let config_dir = if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir)
    } else {
        config_dir().context("Failed to get config directory")?
    };
    Ok(config_dir.join("i-rs").join("contacts.json"))
}

pub fn load_store() -> Result<ContactStore> {
    let path = get_store_path()?;
    
    if !path.exists() {
        return Ok(ContactStore::default());
    }
    
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path.display()))
}

pub fn save_store(store: &ContactStore) -> Result<()> {
    let path = get_store_path()?;
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    
    let content = serde_json::to_string_pretty(store)
        .context("Failed to serialize store")?;
    
    fs::write(&path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}

pub fn add_contact(
    store: &mut ContactStore,
    name: String,
    phone: String,
    email: String,
    relationship: String,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<Contact> {
    let now = Utc::now();
    let contact = Contact {
        name: name.clone(),
        phone,
        email,
        relationship,
        tags,
        remark,
        last_contact: None,
        contact_count: 0,
        created_at: now,
        updated_at: now,
    };
    
    store.add_entry(contact.clone());
    Ok(contact)
}

pub fn delete_contact(store: &mut ContactStore, name: &str) -> Result<Contact> {
    store.remove_entry(name)
        .ok_or_else(|| anyhow::anyhow!("Contact '{}' not found", name))
}

pub fn update_contact(
    store: &mut ContactStore,
    name: &str,
    phone: Option<String>,
    email: Option<String>,
    relationship: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<Contact> {
    let contact = store.get_entry_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Contact '{}' not found", name))?;
    
    if let Some(p) = phone {
        contact.phone = p;
    }
    if let Some(e) = email {
        contact.email = e;
    }
    if let Some(r) = relationship {
        contact.relationship = r;
    }
    if let Some(t) = tags {
        contact.tags = t;
    }
    if let Some(rm) = remark {
        contact.remark = rm;
    }
    contact.updated_at = Utc::now();
    
    Ok(contact.clone())
}

#[allow(dead_code)]
pub fn record_contact(store: &mut ContactStore, name: &str) -> Result<Contact> {
    let contact = store.get_entry_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Contact '{}' not found", name))?;
    
    let now = Utc::now();
    contact.last_contact = Some(now);
    contact.contact_count += 1;
    contact.updated_at = now;
    
    Ok(contact.clone())
}
